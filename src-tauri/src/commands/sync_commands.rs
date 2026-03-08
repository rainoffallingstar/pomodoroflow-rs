//! GitHub 同步相关命令

use pomoflow_rs::{GithubSyncReport, PomodoroAppManager};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tracing::error;

use super::{command_error_result, CommandError, CommandResult};

/// 运行 GitHub 同步（当前为 dry-run 阶段）
#[tauri::command]
pub async fn run_github_sync(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    dry_run: Option<bool>,
) -> Result<CommandResult<GithubSyncReport>, CommandError> {
    let dry_run = dry_run.unwrap_or(true);
    let result = {
        let mut guard = app_manager.lock().await;
        guard.run_github_sync(dry_run).await
    };

    match result {
        Ok(report) => Ok(CommandResult::success(report)),
        Err(err) => {
            error!("Failed to run github sync: {}", err);
            Ok(command_error_result(err))
        }
    }
}
