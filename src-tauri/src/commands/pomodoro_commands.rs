//! 番茄钟相关的 Tauri 命令

use pomoflow_rs::{PomodoroAppManager, PomodoroConfig};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;
use tracing::error;

use super::{command_error_result, CommandError, CommandResult};

/// 获取当前番茄钟会话状态
#[tauri::command]
pub async fn get_pomodoro_session(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
) -> Result<CommandResult<Option<pomoflow_rs::core::pomodoro::PomodoroSession>>, CommandError> {
    // 使用异步锁
    let session = {
        let guard = app_manager.lock().await;
        guard.get_pomodoro_session().await
    };

    match session {
        Ok(session) => Ok(CommandResult::success(session)),
        Err(err) => {
            error!("Failed to get pomodoro session: {}", err);
            Ok(command_error_result(err))
        }
    }
}

/// 启动番茄钟
#[tauri::command]
pub async fn start_pomodoro(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
) -> Result<CommandResult<()>, CommandError> {
    // 使用异步锁
    let result = {
        let mut guard = app_manager.lock().await;
        guard.start_pomodoro().await
    };

    match result {
        Ok(_) => Ok(CommandResult::success(())),
        Err(err) => {
            error!("Failed to start pomodoro: {}", err);
            Ok(command_error_result(err))
        }
    }
}

/// 暂停番茄钟
#[tauri::command]
pub async fn pause_pomodoro(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
) -> Result<CommandResult<()>, CommandError> {
    // 使用异步锁
    let result = {
        let mut guard = app_manager.lock().await;
        guard.pause_pomodoro().await
    };

    match result {
        Ok(_) => Ok(CommandResult::success(())),
        Err(err) => {
            error!("Failed to pause pomodoro: {}", err);
            Ok(command_error_result(err))
        }
    }
}

/// 重置番茄钟
#[tauri::command]
pub async fn reset_pomodoro(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
) -> Result<CommandResult<()>, CommandError> {
    // 使用异步锁
    let result = {
        let mut guard = app_manager.lock().await;
        guard.reset_pomodoro().await
    };

    match result {
        Ok(_) => Ok(CommandResult::success(())),
        Err(err) => {
            error!("Failed to reset pomodoro: {}", err);
            Ok(command_error_result(err))
        }
    }
}

/// 跳过当前阶段
#[tauri::command]
pub async fn skip_pomodoro_phase(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
) -> Result<CommandResult<()>, CommandError> {
    // 使用异步锁
    let result = {
        let mut guard = app_manager.lock().await;
        guard.skip_pomodoro_phase().await
    };

    match result {
        Ok(_) => Ok(CommandResult::success(())),
        Err(err) => {
            error!("Failed to skip pomodoro phase: {}", err);
            Ok(command_error_result(err))
        }
    }
}

/// 更新番茄钟配置
#[tauri::command]
pub async fn update_pomodoro_config(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    work_duration: u64,
    short_break: u64,
    long_break: u64,
    cycles: u32,
) -> Result<CommandResult<()>, CommandError> {
    // 仅更新运行时番茄钟配置，不覆盖用户主题和通知偏好
    let config = PomodoroConfig {
        work_duration,
        short_break_duration: short_break,
        long_break_duration: long_break,
        cycles_until_long_break: cycles,
    };

    // 使用异步锁，调用新的 update_pomodoro_config 方法
    let result = {
        let mut guard = app_manager.lock().await;
        guard.update_pomodoro_config(config).await
    };

    match result {
        Ok(_) => Ok(CommandResult::success(())),
        Err(err) => {
            error!("Failed to update pomodoro config: {}", err);
            Ok(command_error_result(err))
        }
    }
}
