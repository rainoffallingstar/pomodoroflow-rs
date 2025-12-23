//! Tauri 命令模块
//!
//! 这个模块定义了所有前端可以调用的命令接口

use serde::{Deserialize, Serialize};

pub mod config_commands;
pub mod pomodoro_commands;
pub mod todo_commands;
pub mod validators;

// 重新导出所有命令
pub use config_commands::*;
pub use pomodoro_commands::*;
pub use todo_commands::*;

// 通用错误类型
#[derive(thiserror::Error, Debug, serde::Serialize)]
pub enum CommandError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

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

// 从核心库的 AppError 转换
impl From<pomoflow_rs::core::error::AppError> for CommandError {
    fn from(error: pomoflow_rs::core::error::AppError) -> Self {
        match error {
            pomoflow_rs::core::error::AppError::Database(e) => {
                CommandError::Database(e.to_string())
            }
            pomoflow_rs::core::error::AppError::Network(e) => CommandError::Network(e.to_string()),
            pomoflow_rs::core::error::AppError::NotFound(e) => CommandError::NotFound(e),
            pomoflow_rs::core::error::AppError::Other(e) => CommandError::Internal(e),
            _ => CommandError::Internal("Unknown error".to_string()),
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
}

impl<T> CommandResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> Self
    where
        T: Default,
    {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }

    pub fn error_with_type(_error: String) -> Self
    where
        T: Default,
    {
        Self {
            success: false,
            data: None,
            error: Some(_error),
        }
    }
}

// 删除未使用的From实现
