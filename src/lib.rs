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
use core::StateUpdater;
use std::sync::Arc;
use storage::database::Database;

/// 全局应用实例管理器
pub struct PomodoroAppManager {
    /// 应用状态管理器
    state_manager: Arc<AppStateManager>,

    /// 数据库
    database: Arc<Database>,

    /// 任务管理器
    task_manager: Arc<TaskManager>,

    /// 番茄钟服务
    pomodoro_service: PomodoroService,

    /// 状态更新器
    state_updater: StateUpdater,
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
        let pomodoro_service = PomodoroService::new(pomodoro_config);
        println!("✅ Pomodoro service created successfully");

        // 初始化状态更新器
        println!("🔄 Creating state updater...");
        let state_updater = StateUpdater::new(
            Arc::clone(&state_manager),
            pomodoro_service.clone(),
            Arc::clone(&database),
            Default::default(),
        );
        println!("✅ State updater created successfully");

        println!("✅ Application core initialized successfully");

        Ok(Self {
            state_manager,
            database,
            task_manager,
            pomodoro_service,
            state_updater,
        })
    }

    /// 启动应用
    pub async fn start(&mut self) -> Result<()> {
        println!("🚀 Starting PomodoroFlow-Rs...");

        // 启动状态更新器（现在在后台运行，不会阻塞）
        self.state_updater
            .start()
            .await
            .map_err(|e| AppError::Other(format!("Failed to start state updater: {}", e)))?;

        println!("🎉 PomodoroFlow-Rs started successfully!");
        Ok(())
    }

    /// 获取数据库引用（用于标签命令）
    pub fn get_database(&self) -> Arc<Database> {
        Arc::clone(&self.database)
    }

    /// 获取当前番茄钟会话
    pub async fn get_pomodoro_session(&self) -> Result<Option<PomodoroSession>> {
        Ok(self.pomodoro_service.get_session().cloned())
    }

    /// 开始番茄钟
    pub async fn start_pomodoro(&mut self) -> Result<()> {
        // 检查会话状态，如果 remaining = 0 或处于初始状态，先重置
        if let Some(session) = self.pomodoro_service.get_session() {
            // 如果剩余时间为 0（阶段刚结束），需要确保状态正确
            if session.remaining == 0 {
                // 重置当前阶段，确保有正确的时间
                self.pomodoro_service.reset()?;
            }
            // 如果会话已经停止且处于初始状态（未完成过任何 tick），也需要重置
            else if !session.is_running && session.remaining == session.duration {
                self.pomodoro_service.reset()?;
            }
        }

        // 加载任务列表到状态
        let todos = self.database.get_all_todos().await?;
        self.state_manager.bulk_update_todos(todos).await?;

        // 启动番茄钟
        self.pomodoro_service.start()?;
        if let Some(session) = self.pomodoro_service.get_session().cloned() {
            self.state_manager.set_pomodoro_session(session).await;
        }

        // 启动定时器任务
        let state_manager = Arc::new(self.state_manager.clone());
        let pomodoro_service = Arc::new(tokio::sync::RwLock::new(self.pomodoro_service.clone()));
        let _ = self.task_manager.spawn(
            crate::async_utils::task_manager::TaskNames::POMODORO_TICK.to_string(),
            move || {
                let state_manager = state_manager.clone();
                let pomodoro_service = pomodoro_service.clone();
                async move {
                    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
                    loop {
                        interval.tick().await;

                        // 更新番茄钟状态
                        let mut service = pomodoro_service.write().await;
                        if let Some(session) = service.get_session_mut() {
                            if let Ok(_phase_completed) = session.tick() {
                                // 计时器完成了一个阶段
                                if let Some(updated_session) = service.get_session().cloned() {
                                    let _ = state_manager.send_event(
                                        crate::core::state::app_state::AppEvent::PomodoroEvent(
                                            crate::core::pomodoro::PomodoroEvent::PhaseCompleted {
                                                completed_phase: updated_session.phase.clone(),
                                                next_phase: updated_session.phase.clone(),
                                                cycle_count: updated_session.cycle_count,
                                            },
                                        ),
                                    );
                                }
                            }

                            // 更新状态管理器中的会话信息
                            if let Some(updated_session) = service.get_session().cloned() {
                                let _ = state_manager.set_pomodoro_session(updated_session).await;
                            }
                        }
                    }
                }
            },
        );

        Ok(())
    }

    /// 暂停番茄钟
    pub async fn pause_pomodoro(&mut self) -> Result<()> {
        self.pomodoro_service.pause()?;
        if let Some(session) = self.pomodoro_service.get_session().cloned() {
            self.state_manager.set_pomodoro_session(session).await;
        }
        Ok(())
    }

    /// 重置番茄钟
    pub async fn reset_pomodoro(&mut self) -> Result<()> {
        self.pomodoro_service.reset()?;
        if let Some(session) = self.pomodoro_service.get_session().cloned() {
            self.state_manager.set_pomodoro_session(session).await;
        }
        Ok(())
    }

    /// 跳过当前阶段
    pub async fn skip_pomodoro_phase(&mut self) -> Result<()> {
        self.pomodoro_service.skip()?;
        if let Some(session) = self.pomodoro_service.get_session().cloned() {
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
                    return Ok(updated_todo);
                }
                break;
            }
        }

        Err(AppError::NotFound(format!("Todo with id {} not found", id)))
    }

    /// 获取用户配置
    pub async fn get_user_config(&self) -> Result<Option<UserConfig>> {
        let config = self.database.load_user_config().await?;
        Ok(config)
    }

    /// 保存用户配置
    pub async fn save_user_config(&mut self, config: UserConfig) -> Result<()> {
        self.database.save_user_config(&config).await
    }

    /// 更新番茄钟配置（运行时）
    pub async fn update_pomodoro_config(&mut self, config: UserConfig) -> Result<()> {
        // 更新数据库
        self.database.save_user_config(&config).await?;
        
        // 更新 PomodoroService 配置
        let pomodoro_config = PomodoroConfig {
            work_duration: config.pomodoro_work_duration,
            short_break_duration: config.pomodoro_short_break_duration,
            long_break_duration: config.pomodoro_long_break_duration,
            cycles_until_long_break: config.pomodoro_cycles_until_long_break,
        };
        
        self.pomodoro_service.update_config(pomodoro_config)?;
        
        // 更新状态管理器中的配置
        self.state_manager.set_user_config(config).await;
        
        Ok(())
    }

    /// 获取应用版本
    pub fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

impl Default for PomodoroAppManager {
    fn default() -> Self {
        // 返回一个标记为未初始化的实例
        Self {
            state_manager: Arc::new(AppStateManager::new()),
            database: Arc::new(Database::init_uninitialized()),
            task_manager: Arc::new(TaskManager::new()),
            pomodoro_service: PomodoroService::new(PomodoroConfig::default()),
            state_updater: StateUpdater::new_uninitialized(),
        }
    }
}
