//! 异步任务管理器

use std::collections::HashMap;
use std::future::Future;

use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;

use crate::core::error::{AppError, Result};

/// 任务状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 任务元数据
#[derive(Debug, Clone)]
pub struct TaskMetadata {
    pub name: String,
    pub status: TaskStatus,
    pub created_at: std::time::Instant,
    pub started_at: Option<std::time::Instant>,
    pub error: Option<String>,
}

impl TaskMetadata {
    fn new(name: String) -> Self {
        Self {
            name,
            status: TaskStatus::Pending,
            created_at: std::time::Instant::now(),
            started_at: None,
            error: None,
        }
    }
}

/// 任务管理器
#[derive(Debug)]
pub struct TaskManager {
    tasks: Arc<Mutex<HashMap<String, (JoinHandle<Result<()>>, TaskMetadata)>>>,
    shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

// 安全实现 Send，因为所有字段都是 Arc 包装的
unsafe impl Send for TaskManager {}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    /// 启动后台任务
    pub async fn spawn<F, Fut>(
        &self,
        name: String,
        future: F,
    ) -> Result<()>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let mut tasks = self.tasks.lock().await;

        if tasks.contains_key(&name) {
            return Err(AppError::InvalidState(format!(
                "任务 '{}' 已存在",
                name
            )));
        }

        let handle = tokio::spawn(future());

        let metadata = TaskMetadata::new(name.clone());
        tasks.insert(name, (handle, metadata));

        Ok(())
    }

    /// 取消任务
    pub async fn cancel(&self, name: &str) -> Result<bool> {
        let mut tasks = self.tasks.lock().await;

        if let Some((handle, _metadata)) = tasks.remove(name) {
            handle.abort();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 等待任务完成
    pub async fn wait(&self, name: &str, timeout: Option<std::time::Duration>) -> Result<Option<Result<()>>> {
        let mut tasks = self.tasks.lock().await;

        if let Some((handle, _metadata)) = tasks.get_mut(name) {
            if let Some(timeout) = timeout {
                match tokio::time::timeout(timeout, handle).await {
                    Ok(join_result) => {
                        match join_result {
                            Ok(task_result) => Ok(Some(task_result)),
                            Err(e) => Err(AppError::TaskError(e.to_string())),
                        }
                    },
                    Err(_) => {
                        // 超时
                        Ok(None)
                    }
                }
            } else {
                match handle.await {
                    Ok(result) => Ok(Some(result)),
                    Err(e) => Err(AppError::TaskError(e.to_string())),
                }
            }
        } else {
            Ok(None)
        }
    }

    /// 获取所有任务状态
    pub async fn list_tasks(&self) -> HashMap<String, TaskMetadata> {
        let tasks = self.tasks.lock().await;
        tasks.iter()
            .map(|(name, (_, metadata))| (name.clone(), metadata.clone()))
            .collect()
    }

    /// 检查任务是否存在
    pub async fn exists(&self, name: &str) -> bool {
        let tasks = self.tasks.lock().await;
        tasks.contains_key(name)
    }

    /// 清理已完成的任务
    pub async fn cleanup(&self) -> usize {
        let mut tasks = self.tasks.lock().await;
        let mut to_remove = Vec::new();

        for (name, (handle, _metadata)) in tasks.iter() {
            if handle.is_finished() {
                to_remove.push(name.clone());
            }
        }

        let count = to_remove.len();
        for name in to_remove {
            tasks.remove(&name);
        }

        count
    }

    /// 关闭任务管理器，取消所有任务
    pub async fn shutdown(&self) {
        let mut tasks = self.tasks.lock().await;

        // 取消所有任务
        for (_, (handle, _)) in tasks.drain() {
            handle.abort();
        }
    }

    /// 创建带重试的任务
    pub async fn spawn_with_retry<F, Fut>(
        &self,
        name: String,
        future_factory: F,
        max_retries: u32,
        retry_delay: std::time::Duration,
    ) -> Result<()>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let retry_future = async move {
            let mut attempt = 0;
            loop {
                match future_factory().await {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        attempt += 1;
                        if attempt >= max_retries {
                            return Err(e);
                        }

                        // 指数退避
                        let delay = retry_delay * (2_u32.pow(attempt - 1));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        };

        self.spawn(name, || retry_future).await
    }

    /// 定期执行任务
    pub async fn spawn_interval<F, Fut>(
        &self,
        name: String,
        mut interval: tokio::time::Interval,
        mut task: F,
    ) -> Result<()>
    where
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let task_name = name.clone();
        let task_future = async move {
            loop {
                interval.tick().await;
                if let Err(e) = task().await {
                    // 记录错误但不退出循环
                    eprintln!("Periodic task '{}' error: {}", task_name, e);
                }
            }
        };

        self.spawn(name, move || task_future).await
    }

    /// 限制并发数量的任务执行
    pub async fn spawn_with_limit<F, Fut>(
        &self,
        name: String,
        future: F,
        semaphore: Arc<tokio::sync::Semaphore>,
    ) -> Result<()>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        let task_future = async move {
            let _permit = semaphore.acquire().await
                .map_err(|e| AppError::Other(format!("获取信号量失败: {}", e)))?;

            future().await
        };

        self.spawn(name, || task_future).await
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 预定义的任务名称常量
pub struct TaskNames;

impl TaskNames {
    pub const GITHUB_SYNC: &'static str = "github-sync";
    pub const NETWORK_MONITOR: &'static str = "network-monitor";
    pub const BACKUP: &'static str = "backup";
    pub const CLEANUP: &'static str = "cleanup";
    pub const POMODORO_TICK: &'static str = "pomodoro-tick";
    pub const NOTIFICATION: &'static str = "notification";
}

/// 创建常用信号量
pub fn create_semaphore(max_concurrent: usize) -> Arc<tokio::sync::Semaphore> {
    Arc::new(tokio::sync::Semaphore::new(max_concurrent))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_spawn_task() {
        let manager = TaskManager::new();
        let name = "test-task".to_string();

        assert!(manager.spawn(name.clone(), || async {
            sleep(Duration::from_millis(10)).await;
            Ok(())
        }).await.is_ok());
        assert!(manager.exists(&name).await);
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let manager = TaskManager::new();
        let name = "cancel-test".to_string();

        manager.spawn(name.clone(), || async {
            sleep(Duration::from_secs(10)).await;
            Ok(())
        }).await.unwrap();

        // 立即取消
        assert!(manager.cancel(&name).await.is_ok());
        assert!(!manager.exists(&name).await);
    }

    #[tokio::test]
    async fn test_duplicate_task() {
        let manager = TaskManager::new();
        let name = "duplicate-test".to_string();

        assert!(manager.spawn(name.clone(), || async { Ok(()) }).await.is_ok());
        assert!(manager.spawn(name.clone(), || async { Ok(()) }).await.is_err());
    }

    #[tokio::test]
    async fn test_cleanup() {
        let manager = TaskManager::new();
        let name = "cleanup-test".to_string();

        manager.spawn(name.clone(), || async {
            sleep(Duration::from_millis(10)).await;
            Ok(())
        }).await.unwrap();

        // 等待任务完成
        sleep(Duration::from_millis(50)).await;

        let cleaned = manager.cleanup().await;
        assert_eq!(cleaned, 1);
        assert!(!manager.exists(&name).await);
    }

    #[tokio::test]
    async fn test_spawn_with_retry() {
        let manager = TaskManager::new();
        let name = "retry-test".to_string();

        let mut attempt = 0;
        let future_factory = move || async move {
            attempt += 1;
            if attempt < 3 {
                Err(AppError::Network("模拟网络错误".to_string()))
            } else {
                Ok(())
            }
        };

        assert!(manager
            .spawn_with_retry(name, future_factory, 5, Duration::from_millis(10))
            .await
            .is_ok());
    }
}
