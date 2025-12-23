//! 配置管理相关的 Tauri 命令

use pomoflow_rs::PomodoroAppManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;
use tracing::error;

use super::{CommandError, CommandResult};

/// 获取用户配置
#[tauri::command]
pub async fn get_user_config(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
) -> Result<CommandResult<Option<pomoflow_rs::core::state::app_state::UserConfig>>, CommandError> {
    let config = {
        let guard = app_manager.lock().await;
        guard.get_user_config().await
    };

    let config = config.map_err(|e| CommandError::from(e))?;
    Ok(CommandResult::success(config))
}

/// 验证用户配置
fn validate_user_config(config: &pomoflow_rs::core::state::app_state::UserConfig) -> Result<(), CommandError> {
    // 验证番茄钟工作时间（60-3600秒，即1-60分钟）
    if config.pomodoro_work_duration < 60 || config.pomodoro_work_duration > 3600 {
        return Err(CommandError::Validation(
            "番茄钟工作时间必须在1-60分钟之间".to_string()
        ));
    }

    // 验证短休息时间（60-1800秒，即1-30分钟）
    if config.pomodoro_short_break_duration < 60 || config.pomodoro_short_break_duration > 1800 {
        return Err(CommandError::Validation(
            "短休息时间必须在1-30分钟之间".to_string()
        ));
    }

    // 验证长休息时间（60-3600秒，即1-60分钟）
    if config.pomodoro_long_break_duration < 60 || config.pomodoro_long_break_duration > 3600 {
        return Err(CommandError::Validation(
            "长休息时间必须在1-60分钟之间".to_string()
        ));
    }

    // 验证循环次数（1-10次）
    if config.pomodoro_cycles_until_long_break < 1 || config.pomodoro_cycles_until_long_break > 10 {
        return Err(CommandError::Validation(
            "长休息前循环次数必须在1-10次之间".to_string()
        ));
    }

    // 验证主题
    if config.theme != "light" && config.theme != "dark" && config.theme != "system" {
        return Err(CommandError::Validation(
            "主题必须是 light、dark 或 system".to_string()
        ));
    }

    Ok(())
}

/// 保存用户配置
#[tauri::command]
pub async fn save_user_config(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    config: pomoflow_rs::core::state::app_state::UserConfig,
) -> Result<CommandResult<()>, CommandError> {
    // 验证配置
    if let Err(err) = validate_user_config(&config) {
        return Ok(CommandResult {
            success: false,
            data: None,
            error: Some(err.to_string()),
        });
    }

    let result = {
        let mut guard = app_manager.lock().await;
        guard.save_user_config(config).await
    };

    match result {
        Ok(_) => Ok(CommandResult {
            success: true,
            data: Some(()),
            error: None,
        }),
        Err(err) => {
            error!("Failed to save user config: {}", err);
            Ok(CommandResult {
                success: false,
                data: None,
                error: Some(err.to_string()),
            })
        }
    }
}
