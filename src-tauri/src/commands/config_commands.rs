//! 配置管理相关的 Tauri 命令

use pomoflow_rs::{PomodoroAppManager, PomodoroConfig};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;
use tracing::error;

use super::{command_error_result, CommandError, CommandResult};
use super::validators::{validate_github_token, validate_username};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubSyncConfig {
    pub owner: String,
    pub repo: String,
    pub project_number: i64,
}

/// 获取用户配置
#[tauri::command]
pub async fn get_user_config(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
) -> Result<CommandResult<Option<pomoflow_rs::core::state::app_state::UserConfig>>, CommandError> {
    let config = {
        let guard = app_manager.lock().await;
        guard.get_user_config().await
    };

    match config {
        Ok(config) => Ok(CommandResult::success(config)),
        Err(err) => {
            Ok(command_error_result(err))
        }
    }
}

/// 获取 GitHub 同步配置（仅当配置完整时返回）
#[tauri::command]
pub async fn get_github_sync_config(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
) -> Result<CommandResult<Option<GithubSyncConfig>>, CommandError> {
    let config = {
        let guard = app_manager.lock().await;
        guard.get_user_config().await
    };

    match config {
        Ok(Some(user_config)) => {
            let owner = user_config
                .selected_project_owner
                .unwrap_or_default()
                .trim()
                .to_string();
            let repo = user_config
                .selected_project_repo
                .unwrap_or_default()
                .trim()
                .to_string();
            let project_number = user_config.selected_project_number.unwrap_or_default();

            if owner.is_empty() || repo.is_empty() || project_number <= 0 {
                return Ok(CommandResult::success(None));
            }

            Ok(CommandResult::success(Some(GithubSyncConfig {
                owner,
                repo,
                project_number,
            })))
        }
        Ok(None) => Ok(CommandResult::success(None)),
        Err(err) => Ok(command_error_result(err)),
    }
}

/// 验证用户配置
fn validate_user_config(config: &pomoflow_rs::core::state::app_state::UserConfig) -> Result<(), CommandError> {
    // 使用核心层统一校验规则，避免命令层与核心层不一致
    let pomodoro_config = PomodoroConfig {
        work_duration: config.pomodoro_work_duration,
        short_break_duration: config.pomodoro_short_break_duration,
        long_break_duration: config.pomodoro_long_break_duration,
        cycles_until_long_break: config.pomodoro_cycles_until_long_break,
    };
    pomodoro_config
        .validate()
        .map_err(|e| CommandError::Validation(e.to_string()))?;

    // 验证主题
    if config.theme != "light" && config.theme != "dark" && config.theme != "system" {
        return Err(CommandError::Validation(
            "主题必须是 light、dark 或 system".to_string()
        ));
    }

    let has_project_owner = config
        .selected_project_owner
        .as_deref()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let has_project_repo = config
        .selected_project_repo
        .as_deref()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let has_project_number = config
        .selected_project_number
        .map(|v| v > 0)
        .unwrap_or(false);
    let has_any_project = has_project_owner || has_project_repo || has_project_number;
    let has_all_project = has_project_owner && has_project_repo && has_project_number;

    if has_any_project && !has_all_project {
        return Err(CommandError::Validation(
            "GitHub 项目配置需要同时提供 owner、repo 和有效的 project number".to_string(),
        ));
    }
    if has_all_project && config.github_token_encrypted.trim().is_empty() {
        return Err(CommandError::Validation(
            "已配置 GitHub 项目时必须提供 GitHub Token".to_string(),
        ));
    }
    if !config.github_username.trim().is_empty() {
        validate_username(&config.github_username)?;
    }
    if !config.github_token_encrypted.trim().is_empty() {
        validate_github_token(&config.github_token_encrypted)?;
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
        return Ok(command_error_result(err));
    }

    let result = {
        let mut guard = app_manager.lock().await;
        guard.save_user_config(config).await
    };

    match result {
        Ok(_) => Ok(CommandResult::success(())),
        Err(err) => {
            error!("Failed to save user config: {}", err);
            Ok(command_error_result(err))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_user_config;
    use pomoflow_rs::core::state::app_state::UserConfig;

    #[test]
    fn rejects_config_that_core_would_reject() {
        let config = UserConfig {
            github_token_encrypted: String::new(),
            github_username: String::new(),
            last_sync_cursor: None,
            selected_project_owner: None,
            selected_project_repo: None,
            selected_project_number: None,
            pomodoro_work_duration: 1500,
            pomodoro_short_break_duration: 300,
            pomodoro_long_break_duration: 60,
            pomodoro_cycles_until_long_break: 1,
            notifications_enabled: true,
            sound_enabled: true,
            theme: "light".to_string(),
        };

        assert!(validate_user_config(&config).is_err());
    }

    #[test]
    fn rejects_partial_github_project_config() {
        let config = UserConfig {
            github_token_encrypted: String::new(),
            github_username: String::new(),
            last_sync_cursor: None,
            selected_project_owner: Some("acme".to_string()),
            selected_project_repo: None,
            selected_project_number: Some(1),
            pomodoro_work_duration: 1500,
            pomodoro_short_break_duration: 300,
            pomodoro_long_break_duration: 900,
            pomodoro_cycles_until_long_break: 4,
            notifications_enabled: true,
            sound_enabled: true,
            theme: "light".to_string(),
        };

        assert!(validate_user_config(&config).is_err());
    }

    #[test]
    fn accepts_complete_github_project_config() {
        let config = UserConfig {
            github_token_encrypted: "github_pat_xxxxxxxxxxxx".to_string(),
            github_username: "acme".to_string(),
            last_sync_cursor: None,
            selected_project_owner: Some("acme".to_string()),
            selected_project_repo: Some("pomoflow-rs".to_string()),
            selected_project_number: Some(2),
            pomodoro_work_duration: 1500,
            pomodoro_short_break_duration: 300,
            pomodoro_long_break_duration: 900,
            pomodoro_cycles_until_long_break: 4,
            notifications_enabled: true,
            sound_enabled: true,
            theme: "light".to_string(),
        };

        assert!(validate_user_config(&config).is_ok());
    }

    #[test]
    fn rejects_project_config_without_token() {
        let config = UserConfig {
            github_token_encrypted: String::new(),
            github_username: String::new(),
            last_sync_cursor: None,
            selected_project_owner: Some("acme".to_string()),
            selected_project_repo: Some("pomoflow-rs".to_string()),
            selected_project_number: Some(2),
            pomodoro_work_duration: 1500,
            pomodoro_short_break_duration: 300,
            pomodoro_long_break_duration: 900,
            pomodoro_cycles_until_long_break: 4,
            notifications_enabled: true,
            sound_enabled: true,
            theme: "light".to_string(),
        };

        assert!(validate_user_config(&config).is_err());
    }
}
