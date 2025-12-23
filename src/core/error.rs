//! 统一错误类型定义

use thiserror::Error;

/// 应用统一错误类型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Time error: {0}")]
    Time(#[from] chrono::ParseError),

    #[error("Request timeout")]
    Timeout,

    #[error("Task error: {0}")]
    TaskError(String),

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error("Unknown error: {0}")]
    Other(String),
}

impl AppError {
    /// 判断错误是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AppError::Network(_) | AppError::Timeout
        )
    }

    /// 判断是否是认证错误
    pub fn is_auth_error(&self) -> bool {
        matches!(self, AppError::Authentication(_))
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            AppError::Database(_) =>
                "Local database error, please restart the app".to_string(),
            AppError::Network(_) =>
                "Network error, please check connection".to_string(),
            AppError::Authentication(_) =>
                "Authentication failed".to_string(),
            AppError::Timeout =>
                "Request timeout, please check connection".to_string(),
            AppError::Validation(msg) =>
                format!("Input validation failed: {}", msg),
            AppError::NotFound(msg) =>
                format!("Not found: {}", msg),
            AppError::InvalidState(msg) =>
                format!("State error: {}", msg),
            AppError::Serialization(_) =>
                "Data serialization error".to_string(),
            _ =>
                format!("Unknown error: {}", self),
        }
    }

    /// Get error code (for logging and debugging)
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "E001",
            AppError::Network(_) => "E003",
            AppError::Authentication(_) => "E004",
            AppError::Timeout => "E005",
            AppError::Validation(_) => "E007",
            AppError::NotFound(_) => "E008",
            AppError::InvalidState(_) => "E010",
            AppError::Serialization(_) => "E011",
            _ => "E999",
        }
    }
}

/// 简化的 Result 类型
pub type Result<T> = std::result::Result<T, AppError>;

/// 从其他错误类型转换为 AppError
impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        AppError::Other(error.to_string())
    }
}
