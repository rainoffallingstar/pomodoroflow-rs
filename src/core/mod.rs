//! 核心业务逻辑模块
//!
//! 包含番茄钟逻辑、待办事项模型和错误处理

pub mod error;
pub mod state;
pub mod pomodoro;
pub mod todo;
pub mod state_updater;

pub use error::{AppError, Result};
pub use state::{AppStateManager, UserConfig};
pub use pomodoro::{PomodoroService, PomodoroPhase, PomodoroSession, PomodoroConfig};
pub use todo::{TodoService, Todo, NewTodo, TodoUpdate, TodoStatus, TodoFilter, Tag};
pub use state_updater::{StateUpdater, StateUpdaterConfig};
