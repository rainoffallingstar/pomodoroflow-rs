//! Tauri 命令模块
//!
//! 这个模块定义了所有前端可以调用的命令接口

use serde::{Deserialize, Serialize};

pub mod config_commands;
pub mod pomodoro_commands;
pub mod sync_commands;
pub mod todo_commands;
pub mod validators;

// 重新导出所有命令
pub use config_commands::*;
pub use pomodoro_commands::*;
pub use sync_commands::*;
pub use todo_commands::*;

// 通用错误类型
#[derive(thiserror::Error, Debug, serde::Serialize)]
pub enum CommandError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Permission denied: {0}")]
    Permission(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl CommandError {
    pub fn code(&self) -> &'static str {
        match self {
            CommandError::Database(_) => "DATABASE",
            CommandError::Validation(_) => "VALIDATION",
            CommandError::InvalidState(_) => "INVALID_STATE",
            CommandError::NotFound(_) => "NOT_FOUND",
            CommandError::Authentication(_) => "AUTH",
            CommandError::Permission(_) => "PERMISSION",
            CommandError::Network(_) => "NETWORK",
            CommandError::Internal(_) => "INTERNAL",
        }
    }
}

// 从核心库的 AppError 转换
impl From<pomoflow_rs::core::error::AppError> for CommandError {
    fn from(error: pomoflow_rs::core::error::AppError) -> Self {
        use pomoflow_rs::core::error::AppError;
        match error {
            AppError::Database(e) => CommandError::Database(e.to_string()),
            AppError::Network(e) => CommandError::Network(e),
            AppError::Authentication(e) => CommandError::Authentication(e),
            AppError::NotFound(e) => CommandError::NotFound(e),
            AppError::InvalidState(e) => CommandError::InvalidState(e),
            AppError::Validation(e) => CommandError::Validation(e),
            AppError::Timeout => CommandError::Network("Request timeout".to_string()),
            AppError::Serialization(e) => CommandError::Internal(e.to_string()),
            AppError::Io(e) => CommandError::Internal(e.to_string()),
            AppError::Time(e) => CommandError::Internal(e.to_string()),
            AppError::TaskError(e) => CommandError::Internal(e),
            AppError::ChannelError(e) => CommandError::Internal(e),
            AppError::Other(e) => CommandError::Internal(e),
        }
    }
}

// 从 JoinError 转换
impl From<tokio::task::JoinError> for CommandError {
    fn from(error: tokio::task::JoinError) -> Self {
        CommandError::Internal(format!("Task execution error: {}", error))
    }
}

// 从 String 转换
impl From<String> for CommandError {
    fn from(error: String) -> Self {
        CommandError::Internal(error)
    }
}

// 成功响应的包装器
#[derive(Serialize, Deserialize)]
pub struct CommandResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub error_code: Option<String>,
}

impl<T> CommandResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            error_code: None,
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            error_code: Some("INTERNAL".to_string()),
        }
    }

    pub fn error_with_code(error: String, error_code: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            error_code: Some(error_code.into()),
        }
    }
}

pub fn command_error_result<T, E>(error: E) -> CommandResult<T>
where
    E: Into<CommandError>,
{
    let err = error.into();
    CommandResult::error_with_code(err.to_string(), err.code())
}

#[cfg(test)]
mod tests {
    use super::{command_error_result, CommandError, CommandResult};
    use pomoflow_rs::core::error::AppError;

    #[test]
    fn command_result_success_wraps_data() {
        let result = CommandResult::success(123_u32);
        assert!(result.success);
        assert_eq!(result.data, Some(123));
        assert_eq!(result.error, None);
        assert_eq!(result.error_code, None);
    }

    #[test]
    fn command_result_error_has_no_data() {
        let result: CommandResult<u32> = CommandResult::error("boom".to_string());
        assert!(!result.success);
        assert_eq!(result.data, None);
        assert_eq!(result.error.as_deref(), Some("boom"));
        assert_eq!(result.error_code.as_deref(), Some("INTERNAL"));
    }

    #[test]
    fn command_result_error_with_code_has_explicit_code() {
        let result: CommandResult<u32> =
            CommandResult::error_with_code("invalid input".to_string(), "VALIDATION");
        assert!(!result.success);
        assert_eq!(result.error.as_deref(), Some("invalid input"));
        assert_eq!(result.error_code.as_deref(), Some("VALIDATION"));
    }

    #[test]
    fn app_error_maps_to_expected_command_error_and_code() {
        let validation = CommandError::from(AppError::Validation("bad input".to_string()));
        assert!(matches!(validation, CommandError::Validation(_)));
        assert_eq!(validation.code(), "VALIDATION");

        let invalid_state = CommandError::from(AppError::InvalidState("not running".to_string()));
        assert!(matches!(invalid_state, CommandError::InvalidState(_)));
        assert_eq!(invalid_state.code(), "INVALID_STATE");

        let timeout = CommandError::from(AppError::Timeout);
        assert!(matches!(timeout, CommandError::Network(_)));
        assert_eq!(timeout.code(), "NETWORK");
    }

    #[test]
    fn command_error_code_covers_all_variants() {
        let cases = vec![
            (CommandError::Database("db".to_string()), "DATABASE"),
            (CommandError::Validation("v".to_string()), "VALIDATION"),
            (CommandError::InvalidState("s".to_string()), "INVALID_STATE"),
            (CommandError::NotFound("n".to_string()), "NOT_FOUND"),
            (CommandError::Authentication("a".to_string()), "AUTH"),
            (CommandError::Permission("p".to_string()), "PERMISSION"),
            (CommandError::Network("net".to_string()), "NETWORK"),
            (CommandError::Internal("i".to_string()), "INTERNAL"),
        ];

        for (err, expected_code) in cases {
            assert_eq!(err.code(), expected_code);
        }
    }

    #[test]
    fn app_error_maps_auth_to_auth_code() {
        let auth = CommandError::from(AppError::Authentication("bad token".to_string()));
        assert!(matches!(auth, CommandError::Authentication(_)));
        assert_eq!(auth.code(), "AUTH");
    }

    #[test]
    fn command_error_result_uses_validation_code() {
        let result: CommandResult<()> =
            command_error_result(CommandError::Validation("bad input".to_string()));
        assert!(!result.success);
        assert_eq!(result.error_code.as_deref(), Some("VALIDATION"));
    }

    #[test]
    fn command_error_result_maps_invalid_state_from_app_error() {
        let result: CommandResult<()> =
            command_error_result(AppError::InvalidState("not running".to_string()));
        assert!(!result.success);
        assert_eq!(result.error_code.as_deref(), Some("INVALID_STATE"));
    }

    #[test]
    fn command_error_result_maps_not_found_from_app_error() {
        let result: CommandResult<()> = command_error_result(AppError::NotFound("todo".to_string()));
        assert!(!result.success);
        assert_eq!(result.error_code.as_deref(), Some("NOT_FOUND"));
    }
}
