//! 核心业务逻辑模块
//!
//! 包含番茄钟逻辑、待办事项模型和错误处理

pub mod error;
pub mod github_sync;
pub mod pomodoro;
pub mod state;
pub mod state_updater;
pub mod todo;

pub use error::{AppError, Result};
pub use pomodoro::{PomodoroConfig, PomodoroPhase, PomodoroService, PomodoroSession};
pub use state::{AppStateManager, UserConfig};
pub use state_updater::{StateUpdater, StateUpdaterConfig};
pub use todo::{NewTodo, Tag, Todo, TodoFilter, TodoService, TodoStatus, TodoUpdate};
