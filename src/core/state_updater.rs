//! 异步状态更新器
//!
//! 负责定期从服务层拉取最新状态，并通过事件系统通知 UI 更新

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;

use crate::core::error::{AppError, Result};
use crate::core::pomodoro::{PomodoroEvent, PomodoroService};
use crate::core::state::app_state::{AppEvent, AppStateManager};
use crate::storage::database::Database;

/// 状态更新配置
#[derive(Debug, Clone)]
pub struct StateUpdaterConfig {
    /// 状态更新间隔（毫秒）
    pub update_interval_ms: u64,
    /// 缓存清理间隔（毫秒）
    pub cache_cleanup_interval_ms: u64,
    /// 是否启用状态更新
    pub enabled: bool,
}

impl Default for StateUpdaterConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 500,          // 每 500ms 更新一次（优化：减少CPU使用）
            cache_cleanup_interval_ms: 30000, // 每 30s 清理缓存（优化：减少频繁清理）
            enabled: true,
        }
    }
}

/// 状态更新器
pub struct StateUpdater {
    /// 应用状态管理器
    state_manager: Arc<AppStateManager>,

    /// 番茄钟服务
    pomodoro_service: Arc<RwLock<PomodoroService>>,

    /// 数据库连接
    database: Arc<Database>,

    /// 配置
    config: StateUpdaterConfig,

    /// 是否运行中
    is_running: bool,

    /// 关闭通道发送端
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

// 安全实现 Send，因为所有字段都是 Arc 包装的
unsafe impl Send for StateUpdater {}

impl StateUpdater {
    /// 创建新的状态更新器
    pub fn new(
        state_manager: Arc<AppStateManager>,
        pomodoro_service: PomodoroService,
        database: Arc<Database>,
        config: StateUpdaterConfig,
    ) -> Self {
        Self {
            state_manager,
            pomodoro_service: Arc::new(RwLock::new(pomodoro_service)),
            database: Arc::clone(&database),
            config,
            is_running: false,
            shutdown_tx: None,
        }
    }

    /// 创建未初始化的状态更新器
    pub fn new_uninitialized() -> Self {
        // 创建一个未初始化的状态更新器实例
        let state_manager = Arc::new(AppStateManager::new());
        let pomodoro_service =
            PomodoroService::new(crate::core::pomodoro::PomodoroConfig::default());
        let database = Arc::new(Database::init_uninitialized());

        Self {
            state_manager,
            pomodoro_service: Arc::new(RwLock::new(pomodoro_service)),
            database,
            config: StateUpdaterConfig::default(),
            is_running: false,
            shutdown_tx: None,
        }
    }

    /// 启动状态更新循环
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running {
            return Err(AppError::InvalidState("状态更新器已在运行".to_string()));
        }

        self.is_running = true;

        // 创建关闭通道
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let state_manager = self.state_manager.clone();
        let pomodoro_service = self.pomodoro_service.clone();
        let database = Arc::clone(&self.database);
        let config = self.config.clone();

        // 在后台运行状态更新循环
        tokio::spawn(async move {
            Self::run_update_loop(
                state_manager,
                pomodoro_service,
                database,
                config,
                shutdown_rx,
            )
            .await;
        });

        println!("✅ 状态更新器已在后台启动");
        Ok(())
    }

    /// 停止状态更新器
    pub fn stop(&mut self) {
        if self.is_running {
            self.is_running = false;

            // 发送关闭信号
            if let Some(shutdown_tx) = self.shutdown_tx.take() {
                let _ = shutdown_tx.send(());
                println!("🛑 已发送状态更新器关闭信号");
            }
        }
    }

    /// 运行状态更新循环
    async fn run_update_loop(
        state_manager: Arc<AppStateManager>,
        pomodoro_service: Arc<RwLock<PomodoroService>>,
        database: Arc<Database>,
        config: StateUpdaterConfig,
        mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    ) {
        // 创建定时器
        let mut update_interval = interval(Duration::from_millis(config.update_interval_ms));
        let mut cache_cleanup_interval =
            interval(Duration::from_millis(config.cache_cleanup_interval_ms));

        loop {
            tokio::select! {
                // 收到关闭信号
                _ = &mut shutdown_rx => {
                    println!("🛑 状态更新器收到关闭信号，退出循环");
                    break;
                },
                // 状态更新
                _ = update_interval.tick() => {
                    if !config.enabled {
                        continue;
                    }

                    // 更新番茄钟状态（带超时保护）
                    if let Err(_) = tokio::time::timeout(
                        Duration::from_millis(50),
                        Self::update_pomodoro_state(&state_manager, &pomodoro_service)
                    ).await {
                        eprintln!("Pomodoro state update timeout");
                    }

                    // 更新数据库状态（带超时保护）
                    if let Err(_) = tokio::time::timeout(
                        Duration::from_millis(100),
                        Self::update_database_state(&state_manager, &database)
                    ).await {
                        eprintln!("Database state update timeout");
                    }

                    // 发送状态更新事件
                    let _ = state_manager.send_event(AppEvent::SettingsUpdated);
                },
                // 预留：未来在此处执行缓存/状态清理任务
                _ = cache_cleanup_interval.tick() => {}
            }
        }
    }

    /// 更新番茄钟状态
    async fn update_pomodoro_state(
        state_manager: &Arc<AppStateManager>,
        pomodoro_service: &Arc<RwLock<PomodoroService>>,
    ) {
        let mut service = pomodoro_service.write().await;

        // 处理番茄钟 tick
        if let Some(event) = service.tick().await {
            match event {
                PomodoroEvent::Tick {
                    remaining: _,
                    formatted: _,
                    progress: _,
                } => {
                    // 更新 UI 显示
                    let _ = state_manager.send_event(AppEvent::PomodoroEvent(event));
                }
                PomodoroEvent::PhaseCompleted {
                    completed_phase: _,
                    next_phase: _,
                    cycle_count: _,
                } => {
                    // 阶段完成，发送通知
                    let _ = state_manager.send_event(AppEvent::PomodoroStarted);
                }
                PomodoroEvent::StateChanged {
                    is_running,
                    phase: _,
                } => {
                    // 状态变化
                    let _ = state_manager.send_event(match is_running {
                        true => AppEvent::PomodoroStarted,
                        false => AppEvent::PomodoroPaused,
                    });
                }
            }
        }
    }

    /// 更新数据库状态
    async fn update_database_state(state_manager: &Arc<AppStateManager>, database: &Arc<Database>) {
        // 加载最新的任务列表
        if let Ok(todos) = Arc::clone(&database).get_all_todos().await {
            let _ = state_manager.bulk_update_todos(todos).await;
        }
    }

    /// 检查是否运行中
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// 获取配置
    pub fn get_config(&self) -> &StateUpdaterConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: StateUpdaterConfig) {
        self.config = config;
    }
}

impl Drop for StateUpdater {
    fn drop(&mut self) {
        self.stop();
    }
}
