//! 应用状态管理

use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock, oneshot};

use crate::core::error::{AppError, Result};
use crate::core::todo::{Todo, TodoStats, TodoFilter};
use crate::core::pomodoro::{PomodoroSession, PomodoroConfig, PomodoroEvent};

/// 用户配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserConfig {
    pub pomodoro_work_duration: u64,
    pub pomodoro_short_break_duration: u64,
    pub pomodoro_long_break_duration: u64,
    pub pomodoro_cycles_until_long_break: u32,
    pub notifications_enabled: bool,
    pub sound_enabled: bool,
    pub theme: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            pomodoro_work_duration: 1500,
            pomodoro_short_break_duration: 300,
            pomodoro_long_break_duration: 900,
            pomodoro_cycles_until_long_break: 4,
            notifications_enabled: true,
            sound_enabled: true,
            theme: "light".to_string(),
        }
    }
}

/// 应用全局状态
#[derive(Debug, Clone)]
pub struct AppState {
    pub todos: Vec<Todo>,
    pub todo_filter: TodoFilter,
    pub todo_stats: TodoStats,
    pub pomodoro_session: Option<PomodoroSession>,
    pub pomodoro_config: PomodoroConfig,
    pub user_config: Option<UserConfig>,
    pub error_message: Option<String>,
    pub info_message: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            todos: Vec::new(),
            todo_filter: TodoFilter::all(),
            todo_stats: TodoStats::from_todos(&[]),
            pomodoro_session: None,
            pomodoro_config: PomodoroConfig::default(),
            user_config: None,
            error_message: None,
            info_message: None,
        }
    }
}

impl AppState {
    /// 创建新的应用状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取任务筛选结果
    pub fn get_filtered_todos(&self) -> Vec<&Todo> {
        self.todo_filter.apply(&self.todos)
    }

    /// 获取待办任务数量
    pub fn get_pending_todo_count(&self) -> usize {
        self.todos.iter().filter(|t| !t.is_done()).count()
    }

    /// 获取已完成任务数量
    pub fn get_completed_todo_count(&self) -> usize {
        self.todos.iter().filter(|t| t.is_done()).count()
    }

    /// 设置错误消息
    pub fn set_error(&mut self, message: Option<String>) {
        self.error_message = message;
    }

    /// 设置信息消息
    pub fn set_info(&mut self, message: Option<String>) {
        self.info_message = message;
    }

    /// 清除所有消息
    pub fn clear_messages(&mut self) {
        self.error_message = None;
        self.info_message = None;
    }
}

/// 应用事件
#[derive(Debug, Clone)]
pub enum AppEvent {
    // 待办事项事件
    TodoCreated(Todo),
    TodoUpdated(Todo),
    TodoDeleted(String),
    TodoStatusChanged(String, String), // (id, new_status)
    TodoBulkUpdated(Vec<Todo>),

    // 番茄钟事件
    PomodoroStarted,
    PomodoroPaused,
    PomodoroResumed,
    PomodoroReset,
    PomodoroSkipped,
    PomodoroEvent(PomodoroEvent),
    PomodoroConfigUpdated(PomodoroConfig),

    // 配置事件
    UserConfigUpdated(UserConfig),
    SettingsUpdated,

    // UI 事件
    FilterChanged(TodoFilter),
    ThemeChanged(String),

    // 消息事件
    ErrorMessage(String),
    InfoMessage(String),
    MessageCleared,
}

/// 状态查询请求
#[derive(Debug)]
pub enum StateQuery {
    GetTodos,
    GetFilteredTodos,
    GetTodoStats,
    GetPomodoroSession,
    GetUserConfig,
}

/// 状态查询响应
#[derive(Debug)]
pub enum StateQueryResponse {
    Todos(Vec<Todo>),
    FilteredTodos(Vec<Todo>),
    TodoStats(TodoStats),
    PomodoroSession(Option<PomodoroSession>),
    UserConfig(Option<UserConfig>),
}

/// 应用状态管理器
#[derive(Debug, Clone)]
pub struct AppStateManager {
    state: Arc<RwLock<AppState>>,
    event_sender: mpsc::UnboundedSender<AppEvent>,
    query_sender: mpsc::UnboundedSender<(StateQuery, oneshot::Sender<StateQueryResponse>)>,
    // 新增：存储接收端
    event_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<AppEvent>>>>,
    query_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<(StateQuery, oneshot::Sender<StateQueryResponse>)>>>>,
}

// 安全实现 Send + Sync，因为 mpsc::UnboundedSender 可以安全地跨线程发送
unsafe impl Send for AppStateManager {}
unsafe impl Sync for AppStateManager {}

impl AppStateManager {
    /// 创建新的状态管理器
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let (query_sender, query_receiver) = mpsc::unbounded_channel();

        Self {
            state: Arc::new(RwLock::new(AppState::new())),
            event_sender,
            query_sender,
            event_receiver: Arc::new(Mutex::new(Some(event_receiver))),
            query_receiver: Arc::new(Mutex::new(Some(query_receiver))),
        }
    }

    /// 获取状态读取器
    pub async fn get_state(&self) -> tokio::sync::RwLockReadGuard<'_, AppState> {
        self.state.read().await
    }

    /// 获取状态写入器
    pub async fn get_state_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, AppState> {
        self.state.write().await
    }

    /// 发送事件
    pub fn send_event(&self, event: AppEvent) -> Result<()> {
        self.event_sender
            .send(event)
            .map_err(|_| AppError::Other("事件通道已关闭".to_string()))
    }

    /// 发送状态查询
    pub async fn send_query(
        &self,
        query: StateQuery,
    ) -> Result<StateQueryResponse> {
        let (tx, rx) = oneshot::channel();
        self.query_sender
            .send((query, tx))
            .map_err(|_| AppError::Other("查询通道已关闭".to_string()))?;

        rx.await
            .map_err(|_| AppError::Other("查询响应丢失".to_string()))
    }

    // ========================================================================
    // 待办事项操作
    // ========================================================================

    /// 添加任务
    pub async fn add_todo(&self, todo: Todo) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.todos.push(todo.clone());
            state.todo_stats = TodoStats::from_todos(&state.todos);
        }

        self.send_event(AppEvent::TodoCreated(todo))?;
        Ok(())
    }

    /// 更新任务
    pub async fn update_todo(&self, id: &str, updates: crate::core::todo::TodoUpdate) -> Result<Option<Todo>> {
        let mut updated_todo = None;

        {
            let mut state = self.state.write().await;
            if let Some(todo) = state.todos.iter_mut().find(|t| t.id == id) {
                if let Some(title) = updates.title {
                    todo.update_title(title);
                }
                if let Some(description) = updates.description {
                    todo.update_description(description);
                }
                if let Some(status) = updates.status {
                    todo.update_status(status);
                }

                updated_todo = Some(todo.clone());
                state.todo_stats = TodoStats::from_todos(&state.todos);
            }
        }

        if let Some(todo) = updated_todo {
            self.send_event(AppEvent::TodoUpdated(todo.clone()))?;
            Ok(Some(todo))
        } else {
            Ok(None)
        }
    }

    /// 删除任务
    pub async fn delete_todo(&self, id: &str) -> Result<bool> {
        let mut deleted = false;

        {
            let mut state = self.state.write().await;
            if let Some(pos) = state.todos.iter().position(|t| t.id == id) {
                state.todos.remove(pos);
                state.todo_stats = TodoStats::from_todos(&state.todos);
                deleted = true;
            }
        }

        if deleted {
            self.send_event(AppEvent::TodoDeleted(id.to_string()))?;
        }

        Ok(deleted)
    }

    /// 切换任务状态
    pub async fn toggle_todo_status(&self, id: &str) -> Result<Option<Todo>> {
        let mut updated_todo = None;

        {
            let mut state = self.state.write().await;
            if let Some(todo) = state.todos.iter_mut().find(|t| t.id == id) {
                todo.toggle_status();
                updated_todo = Some(todo.clone());
                state.todo_stats = TodoStats::from_todos(&state.todos);
            }
        }

        if let Some(todo) = updated_todo {
            self.send_event(AppEvent::TodoUpdated(todo.clone()))?;
            Ok(Some(todo))
        } else {
            Ok(None)
        }
    }

    /// 批量更新任务
    pub async fn bulk_update_todos(&self, todos: Vec<Todo>) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.todos = todos;
            state.todo_stats = TodoStats::from_todos(&state.todos);
        }

        self.send_event(AppEvent::TodoBulkUpdated(self.state.read().await.todos.clone()))?;
        Ok(())
    }

    // ========================================================================
    // 番茄钟操作
    // ========================================================================

    /// 设置番茄钟会话
    pub async fn set_pomodoro_session(&self, session: PomodoroSession) {
        {
            let mut state = self.state.write().await;
            state.pomodoro_session = Some(session);
        }
        let _ = self.send_event(AppEvent::PomodoroStarted);
    }

    /// 更新番茄钟配置
    pub async fn update_pomodoro_config(&self, config: PomodoroConfig) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.pomodoro_config = config.clone();
        }

        self.send_event(AppEvent::PomodoroConfigUpdated(config))?;
        Ok(())
    }

    // ========================================================================
    // 用户配置操作
    // ========================================================================

    /// 设置用户配置
    pub async fn set_user_config(&self, config: UserConfig) {
        {
            let mut state = self.state.write().await;
            state.user_config = Some(config.clone());
        }

        let _ = self.send_event(AppEvent::UserConfigUpdated(config));
    }

    /// 设置 GitHub 项目
    // ========================================================================
    // 消息操作
    // ========================================================================

    /// 设置错误消息
    pub async fn set_error_message(&self, message: String) {
        {
            let mut state = self.state.write().await;
            state.error_message = Some(message.clone());
        }

        let _ = self.send_event(AppEvent::ErrorMessage(message));
    }

    /// 设置信息消息
    pub async fn set_info_message(&self, message: String) {
        {
            let mut state = self.state.write().await;
            state.info_message = Some(message.clone());
        }

        let _ = self.send_event(AppEvent::InfoMessage(message));
    }

    /// 清除所有消息
    pub async fn clear_messages(&self) {
        {
            let mut state = self.state.write().await;
            state.clear_messages();
        }

        let _ = self.send_event(AppEvent::MessageCleared);
    }

    // ========================================================================
    // 筛选操作
    // ========================================================================

    /// 设置任务筛选器
    pub async fn set_todo_filter(&self, filter: TodoFilter) {
        {
            let mut state = self.state.write().await;
            state.todo_filter = filter.clone();
        }

        let _ = self.send_event(AppEvent::FilterChanged(filter));
    }

    // ========================================================================
    // 查询操作
    // ========================================================================

    /// 获取所有任务
    pub async fn get_all_todos(&self) -> Vec<Todo> {
        self.state.read().await.todos.clone()
    }

    /// 获取筛选后的任务
    pub async fn get_filtered_todos(&self) -> Vec<Todo> {
        let state = self.state.read().await;
        state.get_filtered_todos().into_iter().cloned().collect()
    }

    /// 获取任务统计
    pub async fn get_todo_stats(&self) -> TodoStats {
        self.state.read().await.todo_stats.clone()
    }

    /// 获取番茄钟会话
    pub async fn get_pomodoro_session(&self) -> Option<PomodoroSession> {
        self.state.read().await.pomodoro_session.clone()
    }

    /// 获取用户配置
    pub async fn get_user_config(&self) -> Option<UserConfig> {
        self.state.read().await.user_config.clone()
    }
}

impl Default for AppStateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 事件接收器
#[derive(Debug)]
pub struct EventReceiver {
    receiver: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventReceiver {
    /// 创建事件接收器
    pub fn new(receiver: mpsc::UnboundedReceiver<AppEvent>) -> Self {
        Self { receiver }
    }

    /// 接收事件
    pub async fn recv(&mut self) -> Option<AppEvent> {
        self.receiver.recv().await
    }

    /// 尝试接收事件（非阻塞）
    pub fn try_recv(&mut self) -> Option<Result<AppEvent>> {
        match self.receiver.try_recv() {
            Ok(event) => Some(Ok(event)),
            Err(e) => {
                match e {
                    tokio::sync::mpsc::error::TryRecvError::Empty => None,
                    tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                        Some(Err(AppError::ChannelError("通道已关闭".to_string())))
                    }
                }
            }
        }
    }
}

impl AppStateManager {
    /// 创建事件接收器
    pub fn create_event_receiver(&self) -> Result<EventReceiver> {
        let receiver = self.event_receiver.lock()
            .map_err(|e| AppError::Other(format!("Failed to lock event receiver: {}", e)))?
            .take()
            .ok_or_else(|| AppError::Other("Event receiver already taken".to_string()))?;

        Ok(EventReceiver::new(receiver))
    }
}
