//! PomodoroFlow-Rs 核心库
//!
//! 这个模块提供了应用的核心功能，可被 Tauri 前端调用。

pub mod async_utils;
pub mod core;
pub mod storage;

// 重新导出核心类型
pub use core::{
    error::{AppError, Result},
    pomodoro::{PomodoroConfig, PomodoroPhase, PomodoroService, PomodoroSession},
    state::{AppStateManager, UserConfig},
    todo::{NewTodo, Todo, TodoFilter, TodoService, TodoStatus, TodoUpdate},
};

use async_utils::TaskManager;
use chrono::Utc;
use core::github_sync::{GithubIssue, GithubSyncClient};
use std::sync::Arc;
use storage::database::Database;
use tokio::sync::RwLock;

/// 全局应用实例管理器
pub struct PomodoroAppManager {
    /// 应用状态管理器
    state_manager: Arc<AppStateManager>,

    /// 数据库
    database: Arc<Database>,

    /// 任务管理器
    task_manager: Arc<TaskManager>,

    /// 番茄钟服务
    pomodoro_service: Arc<RwLock<PomodoroService>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GithubSyncTarget {
    pub owner: String,
    pub repo: String,
    pub project_number: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GithubSyncReport {
    pub dry_run: bool,
    pub pending_items: usize,
    pub supported_items: usize,
    pub unsupported_items: usize,
    pub invalid_items: usize,
    pub target: GithubSyncTarget,
    pub errors: Vec<String>,
}

// 安全实现 Send + Sync，因为所有内部字段都是 Arc<...> 包装的
unsafe impl Send for PomodoroAppManager {}
unsafe impl Sync for PomodoroAppManager {}

impl PomodoroAppManager {
    /// 创建新的应用管理器
    pub async fn new() -> Result<Self> {
        println!("🔧 Initializing PomodoroFlow-Rs core library...");

        // 初始化数据库
        let data_dir = dirs::data_dir()
            .ok_or_else(|| AppError::Other("Failed to get data directory".to_string()))?
            .join("pomoflow-rs");

        std::fs::create_dir_all(&data_dir)
            .map_err(|e| AppError::Other(format!("Failed to create data directory: {}", e)))?;

        let db_path = data_dir.join("database.sqlite");
        println!("📊 Initializing database: {:?}", db_path);

        let database = Arc::new(Database::init(&db_path).await?);
        println!("✅ Database initialized successfully");

        // 初始化状态管理器
        println!("🔄 Creating state manager...");
        let state_manager = Arc::new(AppStateManager::new());
        println!("✅ State manager created successfully");

        // 初始化任务管理器
        println!("📋 Creating task manager...");
        let task_manager = Arc::new(TaskManager::new());
        println!("✅ Task manager created successfully");

        // 加载用户配置
        println!("⚙️ Loading user configuration...");
        let user_config = database.load_user_config().await?;
        println!("✅ User configuration loaded successfully");

        // 初始化番茄钟服务
        println!("🍅 Creating Pomodoro service...");
        // 使用用户配置，如果不存在则使用默认值
        let pomodoro_config = if let Some(ref config) = user_config {
            // 将 UserConfig 转换为 PomodoroConfig
            PomodoroConfig {
                work_duration: config.pomodoro_work_duration,
                short_break_duration: config.pomodoro_short_break_duration,
                long_break_duration: config.pomodoro_long_break_duration,
                cycles_until_long_break: config.pomodoro_cycles_until_long_break,
            }
        } else {
            // 如果没有用户配置，使用默认值
            PomodoroConfig::default()
        };
        let pomodoro_service = Arc::new(RwLock::new(PomodoroService::new(pomodoro_config)));
        println!("✅ Pomodoro service created successfully");

        println!("✅ Application core initialized successfully");

        Ok(Self {
            state_manager,
            database,
            task_manager,
            pomodoro_service,
        })
    }

    /// 启动应用
    pub async fn start(&mut self) -> Result<()> {
        println!("🚀 Starting PomodoroFlow-Rs...");

        if let Some(config) = self.database.load_user_config().await? {
            self.state_manager.set_user_config(config).await;
        }

        let todos = self.database.get_all_todos().await?;
        self.state_manager.bulk_update_todos(todos).await?;

        if let Some(session) = self.pomodoro_service.read().await.get_session().cloned() {
            self.state_manager.set_pomodoro_session(session).await;
        }

        self.ensure_pomodoro_tick_task().await?;

        println!("🎉 PomodoroFlow-Rs started successfully!");
        Ok(())
    }

    /// 获取数据库引用（用于标签命令）
    pub fn get_database(&self) -> Arc<Database> {
        Arc::clone(&self.database)
    }

    /// 获取当前番茄钟会话
    pub async fn get_pomodoro_session(&self) -> Result<Option<PomodoroSession>> {
        let service = self.pomodoro_service.read().await;
        Ok(service.get_session().cloned())
    }

    /// 开始番茄钟
    pub async fn start_pomodoro(&mut self) -> Result<()> {
        self.ensure_pomodoro_tick_task().await?;

        let latest_session = {
            let mut service = self.pomodoro_service.write().await;
            service.start()?;
            service.get_session().cloned()
        };

        if let Some(session) = latest_session {
            self.state_manager.set_pomodoro_session(session).await;
        }

        Ok(())
    }

    /// 暂停番茄钟
    pub async fn pause_pomodoro(&mut self) -> Result<()> {
        let session = {
            let mut service = self.pomodoro_service.write().await;
            service.pause()?;
            service.get_session().cloned()
        };
        if let Some(session) = session {
            self.state_manager.set_pomodoro_session(session).await;
        }
        Ok(())
    }

    /// 重置番茄钟
    pub async fn reset_pomodoro(&mut self) -> Result<()> {
        let session = {
            let mut service = self.pomodoro_service.write().await;
            service.reset()?;
            service.get_session().cloned()
        };
        if let Some(session) = session {
            self.state_manager.set_pomodoro_session(session).await;
        }
        Ok(())
    }

    /// 跳过当前阶段
    pub async fn skip_pomodoro_phase(&mut self) -> Result<()> {
        let session = {
            let mut service = self.pomodoro_service.write().await;
            service.skip()?;
            service.get_session().cloned()
        };
        if let Some(session) = session {
            self.state_manager.set_pomodoro_session(session).await;
        }
        Ok(())
    }

    /// 获取所有待办事项
    pub async fn get_todos(&self) -> Result<Vec<Todo>> {
        let todos = self.state_manager.get_all_todos().await;
        Ok(todos)
    }

    /// 创建新任务（带指定状态）
    pub async fn create_todo_with_status(
        &mut self,
        title: String,
        description: Option<String>,
        status: crate::core::todo::TodoStatus,
    ) -> Result<Todo> {
        let new_todo = crate::core::todo::NewTodo {
            title,
            description,
            status,
        };

        // 保存到数据库
        let todo = self.database.create_todo(&new_todo).await?;

        // 添加到状态
        self.state_manager.add_todo(todo.clone()).await?;

        Ok(todo)
    }

    /// 创建新任务（默认状态为 Todo）
    pub async fn create_todo(
        &mut self,
        title: String,
        description: Option<String>,
    ) -> Result<Todo> {
        self.create_todo_with_status(title, description, crate::core::todo::TodoStatus::Todo)
            .await
    }

    /// 更新任务
    pub async fn update_todo(&mut self, id: &str, updates: TodoUpdate) -> Result<Todo> {
        // 更新数据库
        let updated = self.database.update_todo(id, &updates).await?;

        if let Some(todo) = updated.clone() {
            // 更新状态
            self.state_manager.update_todo(id, updates).await?;
            let _ = self.enqueue_todo_issue_sync(&todo, "todo_updated").await;
            Ok(todo)
        } else {
            Err(AppError::NotFound(format!("Todo with id {} not found", id)))
        }
    }

    /// 删除任务
    pub async fn delete_todo(&mut self, id: &str) -> Result<()> {
        // 从数据库删除
        self.database.delete_todo(id).await?;

        // 从状态删除
        self.state_manager.delete_todo(id).await?;

        Ok(())
    }

    /// 切换任务状态
    pub async fn toggle_todo_status(&mut self, id: &str) -> Result<Todo> {
        self.state_manager.toggle_todo_status(id).await?;

        // 同步到数据库并获取更新的任务
        let todos = self.state_manager.get_all_todos().await;
        for todo in todos {
            if todo.id == id {
                let updates = TodoUpdate::new().with_status(todo.status);
                let updated = self.database.update_todo(id, &updates).await?;
                if let Some(updated_todo) = updated {
                    let _ = self
                        .enqueue_todo_issue_sync(&updated_todo, "todo_status_toggled")
                        .await;
                    return Ok(updated_todo);
                }
                break;
            }
        }

        Err(AppError::NotFound(format!("Todo with id {} not found", id)))
    }

    /// 显式设置任务状态
    pub async fn set_todo_status(&mut self, id: &str, status: TodoStatus) -> Result<Todo> {
        self.state_manager
            .set_todo_status(id, status.clone())
            .await?;
        let updates = TodoUpdate::new().with_status(status);
        if let Some(updated_todo) = self.database.update_todo(id, &updates).await? {
            let _ = self
                .enqueue_todo_issue_sync(&updated_todo, "todo_status_set")
                .await;
            Ok(updated_todo)
        } else {
            Err(AppError::NotFound(format!("Todo with id {} not found", id)))
        }
    }

    /// 关联待办与 GitHub Issue / Project
    pub async fn link_todo_github(
        &mut self,
        id: &str,
        issue_id: i64,
        issue_number: i64,
        project_id: i64,
    ) -> Result<Todo> {
        if issue_id <= 0 || issue_number <= 0 || project_id <= 0 {
            return Err(AppError::Validation(
                "issue_id, issue_number, project_id 必须为正整数".to_string(),
            ));
        }

        let updates = TodoUpdate::new()
            .with_github_issue_id(Some(issue_id))
            .with_github_issue_number(Some(issue_number))
            .with_github_project_id(Some(project_id));

        if let Some(updated_todo) = self.database.update_todo(id, &updates).await? {
            self.state_manager.update_todo(id, updates).await?;
            let payload = serde_json::json!({
                "id": id,
                "github_issue_id": issue_id,
                "github_issue_number": issue_number,
                "github_project_id": project_id,
                "action": "link_github",
            });
            let _ = self.database.add_to_sync_queue("update", id, &payload).await;
            let _ = self
                .enqueue_todo_issue_sync(&updated_todo, "todo_github_linked")
                .await;
            Ok(updated_todo)
        } else {
            Err(AppError::NotFound(format!("Todo with id {} not found", id)))
        }
    }

    /// 清除待办的 GitHub 关联
    pub async fn clear_todo_github_link(&mut self, id: &str) -> Result<Todo> {
        let updates = TodoUpdate::new()
            .with_github_issue_id(None)
            .with_github_issue_number(None)
            .with_github_project_id(None);

        if let Some(updated_todo) = self.database.update_todo(id, &updates).await? {
            self.state_manager.update_todo(id, updates).await?;
            let payload = serde_json::json!({
                "id": id,
                "github_issue_id": serde_json::Value::Null,
                "github_issue_number": serde_json::Value::Null,
                "github_project_id": serde_json::Value::Null,
                "action": "clear_github_link",
            });
            let _ = self.database.add_to_sync_queue("update", id, &payload).await;
            Ok(updated_todo)
        } else {
            Err(AppError::NotFound(format!("Todo with id {} not found", id)))
        }
    }

    /// 获取用户配置
    pub async fn get_user_config(&self) -> Result<Option<UserConfig>> {
        let config = self.database.load_user_config().await?;
        Ok(config)
    }

    /// 保存用户配置
    pub async fn save_user_config(&mut self, config: UserConfig) -> Result<()> {
        let config = normalize_user_config(config);
        let previous_config = self.database.load_user_config().await?;
        let next_pomodoro_config = PomodoroConfig {
            work_duration: config.pomodoro_work_duration,
            short_break_duration: config.pomodoro_short_break_duration,
            long_break_duration: config.pomodoro_long_break_duration,
            cycles_until_long_break: config.pomodoro_cycles_until_long_break,
        };
        next_pomodoro_config.validate()?;

        self.database.save_user_config(&config).await?;

        if let Err(err) = self.sync_runtime_config(&config).await {
            if let Some(previous) = previous_config {
                let _ = self.database.save_user_config(&previous).await;
                let _ = self.sync_runtime_config(&previous).await;
            }
            return Err(err);
        }

        self.state_manager.set_user_config(config).await;
        Ok(())
    }

    /// 更新番茄钟配置（仅运行时，不落盘）
    pub async fn update_pomodoro_config(&mut self, config: PomodoroConfig) -> Result<()> {
        self.pomodoro_service
            .write()
            .await
            .update_config(config.clone())?;
        self.state_manager.update_pomodoro_config(config).await?;
        Ok(())
    }

    async fn sync_runtime_config(&mut self, config: &UserConfig) -> Result<()> {
        let pomodoro_config = PomodoroConfig {
            work_duration: config.pomodoro_work_duration,
            short_break_duration: config.pomodoro_short_break_duration,
            long_break_duration: config.pomodoro_long_break_duration,
            cycles_until_long_break: config.pomodoro_cycles_until_long_break,
        };

        self.update_pomodoro_config(pomodoro_config).await
    }

    async fn enqueue_todo_issue_sync(&self, todo: &Todo, reason: &str) -> Result<()> {
        if !has_linked_issue(todo) {
            return Ok(());
        }
        let issue_number = match todo.github_issue_number {
            Some(v) if v > 0 => v,
            _ => return Ok(()),
        };
        let payload = serde_json::json!({
            "id": todo.id,
            "github_issue_number": issue_number,
            "github_project_id": todo.github_project_id,
            "title": todo.title,
            "status": todo.status,
            "project_status": map_todo_status_to_project_status(&todo.status),
            "action": "sync_issue",
            "reason": reason,
        });
        let _ = self
            .database
            .add_to_sync_queue("update", &todo.id, &payload)
            .await?;
        Ok(())
    }

    /// 执行 GitHub 同步流程
    pub async fn run_github_sync(&mut self, dry_run: bool) -> Result<GithubSyncReport> {
        let config = self
            .database
            .load_user_config()
            .await?
            .map(normalize_user_config)
            .ok_or_else(|| AppError::NotFound("用户配置不存在".to_string()))?;

        let target = build_github_sync_target(&config)?;
        let pending = self.database.get_pending_sync_queue().await?;

        let mut report = GithubSyncReport {
            dry_run,
            pending_items: pending.len(),
            supported_items: 0,
            unsupported_items: 0,
            invalid_items: 0,
            target,
            errors: Vec::new(),
        };
        let sync_cursor = config.last_sync_cursor.clone();
        let github_client = if dry_run {
            None
        } else {
            Some(GithubSyncClient::new(
                &config.github_token_encrypted,
                &report.target.owner,
                &report.target.repo,
            )?)
        };

        for (queue_id, operation_type, _record_id, payload_raw) in pending {
            let payload: serde_json::Value = match serde_json::from_str(&payload_raw) {
                Ok(v) => v,
                Err(err) => {
                    report.invalid_items += 1;
                    let message = format!("queue #{queue_id}: payload 解析失败: {err}");
                    report.errors.push(message.clone());
                    if !dry_run {
                        let _ = self
                            .database
                            .mark_sync_queue_failed(queue_id, &message)
                            .await;
                    }
                    continue;
                }
            };

            if is_supported_sync_item(&operation_type, &payload) {
                report.supported_items += 1;
                if !dry_run {
                    match self
                        .execute_supported_sync_item(
                            github_client.as_ref().expect("client initialized"),
                            report.target.project_number,
                            queue_id,
                            &payload,
                        )
                        .await
                    {
                        Ok(_) => {
                            let _ = self.database.mark_sync_queue_synced(queue_id).await;
                        }
                        Err(err) => {
                            let message = err.to_string();
                            report.errors.push(format!("queue #{queue_id}: {message}"));
                            let _ = self
                                .database
                                .mark_sync_queue_failed(queue_id, &message)
                                .await;
                        }
                    }
                }
            } else {
                report.unsupported_items += 1;
                if !dry_run {
                    let message = format!(
                        "queue #{queue_id}: 暂不支持的同步项 (operation_type={operation_type}, action={})",
                        sync_action_from_payload(&payload).unwrap_or("unknown"),
                    );
                    report.errors.push(message.clone());
                    let _ = self
                        .database
                        .mark_sync_queue_failed(queue_id, &message)
                        .await;
                }
            }
        }

        if !dry_run {
            if let Some(client) = github_client.as_ref() {
                self.pull_remote_issue_updates(client, sync_cursor.as_deref(), &mut report)
                    .await;
            }
            let _ = self.database.cleanup_sync_queue().await;
            let _ = self
                .persist_last_sync_cursor(&config, Utc::now().to_rfc3339())
                .await;
        }

        Ok(report)
    }

    async fn execute_supported_sync_item(
        &mut self,
        client: &GithubSyncClient,
        project_number: i64,
        queue_id: i64,
        payload: &serde_json::Value,
    ) -> Result<()> {
        let action = sync_action_from_payload(payload).unwrap_or_default();
        match action {
            "link_github" => {
                let issue_number = sync_issue_number_from_payload(payload).ok_or_else(|| {
                    AppError::Validation(format!(
                        "queue #{queue_id}: link_github 缺少 github_issue_number"
                    ))
                })?;
                let _ = client.get_issue(issue_number).await?;
                Ok(())
            }
            "clear_github_link" => Ok(()),
            "sync_issue" => {
                let issue_number = sync_issue_number_from_payload(payload).ok_or_else(|| {
                    AppError::Validation(format!(
                        "queue #{queue_id}: sync_issue 缺少 github_issue_number"
                    ))
                })?;
                let title = payload
                    .get("title")
                    .and_then(|v| v.as_str())
                    .map(|v| v.trim())
                    .filter(|v| !v.is_empty());
                let status = payload
                    .get("status")
                    .and_then(|v| v.as_str())
                    .and_then(parse_todo_status);
                let _ = client.update_issue(issue_number, title, status.as_ref()).await?;
                if sync_project_id_from_payload(payload).is_some() {
                    if let Some(project_status) = sync_project_status_from_payload(payload) {
                        client
                            .update_project_item_status(
                                project_number,
                                issue_number,
                                project_status,
                            )
                            .await?;
                    }
                }
                Ok(())
            }
            _ => Err(AppError::InvalidState(format!(
                "queue #{queue_id}: 未知 action={action}"
            ))),
        }
    }

    async fn pull_remote_issue_updates(
        &mut self,
        client: &GithubSyncClient,
        since: Option<&str>,
        report: &mut GithubSyncReport,
    ) {
        let todos = match self.database.get_all_todos().await {
            Ok(v) => v,
            Err(err) => {
                report.errors.push(format!("回拉阶段读取本地 todo 失败: {}", err));
                return;
            }
        };
        let recent_issues = match client.list_issues_since(since).await {
            Ok(v) => v,
            Err(err) => {
                report.errors.push(format!("回拉阶段拉取 issue 列表失败: {}", err));
                return;
            }
        };
        let issue_map: std::collections::HashMap<i64, GithubIssue> = recent_issues
            .into_iter()
            .map(|issue| (issue.number, issue))
            .collect();

        for todo in todos {
            if !has_linked_issue(&todo) {
                continue;
            }
            let issue_number = match todo.github_issue_number {
                Some(v) => v,
                None => continue,
            };

            let issue_result = if let Some(issue) = issue_map.get(&issue_number).cloned() {
                Ok(issue)
            } else if since.is_some() {
                // 增量模式下未命中列表说明该 issue 未更新，无需请求详情。
                continue;
            } else {
                client.get_issue(issue_number).await
            };

            match issue_result {
                Ok(issue) => {
                    let has_pending_local = self
                        .database
                        .has_pending_sync_for_record(&todo.id)
                        .await
                        .unwrap_or(false);
                    if has_pending_local {
                        report.errors.push(format!(
                            "conflict: todo {} 存在待推送本地改动，跳过远端覆盖 (issue #{})",
                            todo.id, issue_number
                        ));
                        continue;
                    }
                    if issue.updated_at <= todo.updated_at {
                        continue;
                    }
                    if let Err(err) = self.apply_remote_issue_to_todo(&todo.id, &issue).await {
                        report.errors.push(format!(
                            "pull todo {} from issue #{} 失败: {}",
                            todo.id, issue_number, err
                        ));
                    }
                }
                Err(err) => {
                    report.errors.push(format!(
                        "pull issue #{} 失败: {}",
                        issue_number, err
                    ));
                }
            }
        }
    }

    async fn apply_remote_issue_to_todo(&mut self, todo_id: &str, issue: &GithubIssue) -> Result<()> {
        let mut updates = TodoUpdate::new();
        let mut changed = false;

        let current = self.database.get_todo_by_id(todo_id).await?;
        let current = match current {
            Some(todo) => todo,
            None => return Ok(()),
        };

        if current.title != issue.title {
            updates = updates.with_title(issue.title.clone());
            changed = true;
        }

        let next_status = if issue.state.eq_ignore_ascii_case("closed") {
            Some(TodoStatus::Done)
        } else {
            Some(TodoStatus::Todo)
        };
        if let Some(next_status) = next_status {
            if current.status != next_status {
                updates = updates.with_status(next_status);
                changed = true;
            }
        }

        if !changed {
            return Ok(());
        }

        if let Some(updated_todo) = self.database.update_todo(todo_id, &updates).await? {
            self.state_manager.update_todo(todo_id, updates).await?;
            let _ = self
                .enqueue_todo_issue_sync(&updated_todo, "remote_pull_reconciled")
                .await;
        }

        Ok(())
    }

    async fn persist_last_sync_cursor(&mut self, base_config: &UserConfig, cursor: String) -> Result<()> {
        let mut next = base_config.clone();
        next.last_sync_cursor = Some(cursor);
        self.database.save_user_config(&next).await?;
        self.state_manager.set_user_config(next).await;
        Ok(())
    }

    async fn ensure_pomodoro_tick_task(&self) -> Result<()> {
        let task_name = crate::async_utils::task_manager::TaskNames::POMODORO_TICK.to_string();
        if self.task_manager.exists(&task_name).await {
            return Ok(());
        }

        let state_manager = Arc::clone(&self.state_manager);
        let pomodoro_service = Arc::clone(&self.pomodoro_service);
        self.task_manager
            .spawn(task_name, move || {
                let state_manager = state_manager.clone();
                let pomodoro_service = pomodoro_service.clone();
                async move {
                    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
                    loop {
                        interval.tick().await;

                        let mut service = pomodoro_service.write().await;
                        let event = service.tick().await;
                        let updated_session = service.get_session().cloned();
                        drop(service);

                        if let Some(session) = updated_session {
                            let _ = state_manager.set_pomodoro_session(session).await;
                        }

                        if let Some(event) = event {
                            let _ = state_manager.send_event(
                                crate::core::state::app_state::AppEvent::PomodoroEvent(event),
                            );
                        }
                    }
                }
            })
            .await
    }

    /// 获取应用版本
    pub fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

fn build_github_sync_target(config: &UserConfig) -> Result<GithubSyncTarget> {
    let owner = config
        .selected_project_owner
        .clone()
        .unwrap_or_default()
        .trim()
        .to_string();
    let repo = config
        .selected_project_repo
        .clone()
        .unwrap_or_default()
        .trim()
        .to_string();
    let project_number = config.selected_project_number.unwrap_or_default();

    if owner.is_empty() || repo.is_empty() || project_number <= 0 {
        return Err(AppError::Validation(
            "GitHub 项目配置不完整，请先在设置中配置 owner/repo/project number".to_string(),
        ));
    }

    Ok(GithubSyncTarget {
        owner,
        repo,
        project_number,
    })
}

fn is_supported_sync_item(operation_type: &str, payload: &serde_json::Value) -> bool {
    if operation_type != "update" {
        return false;
    }

    matches!(
        payload.get("action").and_then(|v| v.as_str()),
        Some("link_github") | Some("clear_github_link") | Some("sync_issue")
    )
}

fn sync_action_from_payload(payload: &serde_json::Value) -> Option<&str> {
    payload.get("action").and_then(|v| v.as_str())
}

fn sync_issue_number_from_payload(payload: &serde_json::Value) -> Option<i64> {
    payload.get("github_issue_number").and_then(|v| v.as_i64())
}

fn sync_project_status_from_payload(payload: &serde_json::Value) -> Option<&str> {
    payload.get("project_status").and_then(|v| v.as_str())
}

fn sync_project_id_from_payload(payload: &serde_json::Value) -> Option<i64> {
    payload.get("github_project_id").and_then(|v| v.as_i64())
}

fn parse_todo_status(status: &str) -> Option<TodoStatus> {
    match status {
        "todo" => Some(TodoStatus::Todo),
        "in_progress" => Some(TodoStatus::InProgress),
        "done" => Some(TodoStatus::Done),
        _ => None,
    }
}

fn has_linked_issue(todo: &Todo) -> bool {
    todo.github_issue_number.is_some_and(|n| n > 0)
}

fn map_todo_status_to_project_status(status: &TodoStatus) -> &'static str {
    match status {
        TodoStatus::Todo => "Todo",
        TodoStatus::InProgress => "In Progress",
        TodoStatus::Done => "Done",
    }
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn normalize_user_config(mut config: UserConfig) -> UserConfig {
    config.github_token_encrypted = config.github_token_encrypted.trim().to_string();
    config.github_username = config.github_username.trim().to_string();
    config.last_sync_cursor = normalize_optional_text(config.last_sync_cursor);
    config.selected_project_owner = normalize_optional_text(config.selected_project_owner);
    config.selected_project_repo = normalize_optional_text(config.selected_project_repo);
    if config.selected_project_number.is_some_and(|n| n <= 0) {
        config.selected_project_number = None;
    }
    config
}

impl Default for PomodoroAppManager {
    fn default() -> Self {
        // 返回一个标记为未初始化的实例
        Self {
            state_manager: Arc::new(AppStateManager::new()),
            database: Arc::new(Database::init_uninitialized()),
            task_manager: Arc::new(TaskManager::new()),
            pomodoro_service: Arc::new(RwLock::new(
                PomodoroService::new(PomodoroConfig::default()),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_supported_sync_item, map_todo_status_to_project_status, sync_action_from_payload,
        sync_project_id_from_payload, sync_project_status_from_payload,
    };
    use crate::core::todo::TodoStatus;

    #[test]
    fn supported_sync_item_requires_update_and_known_action() {
        let payload = serde_json::json!({ "action": "link_github" });
        assert!(is_supported_sync_item("update", &payload));
        let payload = serde_json::json!({ "action": "sync_issue" });
        assert!(is_supported_sync_item("update", &payload));
        assert!(!is_supported_sync_item("create", &payload));
    }

    #[test]
    fn sync_action_from_payload_reads_action_field() {
        let payload = serde_json::json!({ "action": "clear_github_link" });
        assert_eq!(sync_action_from_payload(&payload), Some("clear_github_link"));
        let invalid = serde_json::json!({ "foo": "bar" });
        assert_eq!(sync_action_from_payload(&invalid), None);
    }

    #[test]
    fn todo_status_maps_to_project_status() {
        assert_eq!(map_todo_status_to_project_status(&TodoStatus::Todo), "Todo");
        assert_eq!(
            map_todo_status_to_project_status(&TodoStatus::InProgress),
            "In Progress"
        );
        assert_eq!(map_todo_status_to_project_status(&TodoStatus::Done), "Done");
    }

    #[test]
    fn sync_project_fields_can_be_parsed_from_payload() {
        let payload = serde_json::json!({
            "github_project_id": 7,
            "project_status": "In Progress"
        });
        assert_eq!(sync_project_id_from_payload(&payload), Some(7));
        assert_eq!(
            sync_project_status_from_payload(&payload),
            Some("In Progress")
        );
    }
}
