//! 待办事项相关的 Tauri 命令

use super::validators::{validate_id, validate_todo_description, validate_todo_title};
use super::{command_error_result, CommandError, CommandResult};
use pomoflow_rs::{PomodoroAppManager, Todo, TodoStatus, TodoUpdate};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

fn validate_positive_i64(value: i64, field_name: &str) -> Result<(), CommandError> {
    if value <= 0 {
        return Err(CommandError::Validation(format!(
            "{} 必须是正整数",
            field_name
        )));
    }
    Ok(())
}

/// 获取所有待办事项
#[tauri::command]
pub async fn get_todos(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
) -> Result<CommandResult<Vec<Todo>>, CommandError> {
    let todos = {
        let guard = app_manager.lock().await;
        guard.get_todos().await
    };

    match todos {
        Ok(todos) => Ok(CommandResult::success(todos)),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 创建新任务
#[tauri::command]
pub async fn create_todo(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    title: String,
    description: Option<String>,
    status: Option<String>,
) -> Result<CommandResult<Todo>, CommandError> {
    // 验证输入
    if let Err(err) = validate_todo_title(&title) {
        return Ok(command_error_result(err));
    }

    if let Some(ref desc) = description {
        if let Err(err) = validate_todo_description(desc) {
            return Ok(command_error_result(err));
        }
    }

    // 解析状态字符串到 TodoStatus 枚举
    let todo_status = match status.as_deref() {
        Some("in_progress") => TodoStatus::InProgress,
        Some("done") => TodoStatus::Done,
        _ => TodoStatus::Todo,
    };

    let todo = {
        let mut guard = app_manager.lock().await;
        guard
            .create_todo_with_status(title, description, todo_status)
            .await
    };

    match todo {
        Ok(todo) => Ok(CommandResult::success(todo)),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 更新任务
#[tauri::command]
pub async fn update_todo(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    id: String,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
) -> Result<CommandResult<Todo>, CommandError> {
    // 验证ID
    if let Err(err) = validate_id(&id) {
        return Ok(command_error_result(err));
    }

    // 验证输入
    if let Some(ref title) = title {
        if let Err(err) = validate_todo_title(title) {
            return Ok(command_error_result(err));
        }
    }

    if let Some(ref desc) = description {
        if let Err(err) = validate_todo_description(desc) {
            return Ok(command_error_result(err));
        }
    }

    // 创建更新对象
    let mut updates = TodoUpdate::new();

    if let Some(title) = title {
        updates = updates.with_title(title);
    }

    if let Some(description) = description {
        updates = updates.with_description(Some(description));
    }

    if let Some(status) = status {
        let parsed_status = match TodoStatus::from_string(&status) {
            Ok(status) => status,
            Err(err) => return Ok(command_error_result(err)),
        };
        updates = updates.with_status(parsed_status);
    }

    let todo = {
        let mut guard = app_manager.lock().await;
        guard.update_todo(&id, updates).await
    };

    match todo {
        Ok(todo) => Ok(CommandResult::success(todo)),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 删除任务
#[tauri::command]
pub async fn delete_todo(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    id: String,
) -> Result<CommandResult<bool>, CommandError> {
    // 验证ID
    if let Err(err) = validate_id(&id) {
        return Ok(command_error_result(err));
    }

    match {
        let mut guard = app_manager.lock().await;
        guard.delete_todo(&id).await
    } {
        Ok(_) => Ok(CommandResult::success(true)),
        Err(e) => Ok(command_error_result(e)),
    }
}

// ========================================================================
// 标签相关命令
// ========================================================================

/// 获取所有标签
#[tauri::command]
pub async fn get_tags(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
) -> Result<CommandResult<Vec<(String, String, String)>>, CommandError> {
    let db = {
        let guard = app_manager.lock().await;
        guard.get_database().clone()
    };

    match db.get_all_tags().await {
        Ok(tags) => Ok(CommandResult::success(tags)),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 创建标签
#[tauri::command]
pub async fn create_tag(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    name: String,
    color: String,
) -> Result<CommandResult<(String, String, String)>, CommandError> {
    // 验证输入
    if name.trim().is_empty() {
        return Ok(command_error_result(CommandError::Validation(
            "标签名称不能为空".to_string(),
        )));
    }

    if name.len() > 50 {
        return Ok(command_error_result(CommandError::Validation(
            "标签名称不能超过50字符".to_string(),
        )));
    }

    // 验证颜色格式（简单的 hex 颜色代码）
    if !color.starts_with('#') || color.len() != 7 {
        return Ok(command_error_result(CommandError::Validation(
            "颜色格式无效，应为 #RRGGBB 格式".to_string(),
        )));
    }

    let db = {
        let guard = app_manager.lock().await;
        guard.get_database().clone()
    };

    match db.create_tag(&name, &color).await {
        Ok(tag) => Ok(CommandResult::success(tag)),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 删除标签
#[tauri::command]
pub async fn delete_tag(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    id: String,
) -> Result<CommandResult<bool>, CommandError> {
    // 验证ID
    if let Err(err) = validate_id(&id) {
        return Ok(command_error_result(err));
    }

    let db = {
        let guard = app_manager.lock().await;
        guard.get_database().clone()
    };

    match db.delete_tag(&id).await {
        Ok(deleted) => Ok(CommandResult::success(deleted)),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 为待办事项添加标签
#[tauri::command]
pub async fn assign_tag_to_todo(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    todo_id: String,
    tag_id: String,
) -> Result<CommandResult<()>, CommandError> {
    // 验证ID
    if let Err(err) = validate_id(&todo_id) {
        return Ok(command_error_result(err));
    }
    if let Err(err) = validate_id(&tag_id) {
        return Ok(command_error_result(err));
    }

    let db = {
        let guard = app_manager.lock().await;
        guard.get_database().clone()
    };

    match db.add_tag_to_todo(&todo_id, &tag_id).await {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 从待办事项移除标签
#[tauri::command]
pub async fn remove_tag_from_todo(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    todo_id: String,
    tag_id: String,
) -> Result<CommandResult<()>, CommandError> {
    // 验证ID
    if let Err(err) = validate_id(&todo_id) {
        return Ok(command_error_result(err));
    }
    if let Err(err) = validate_id(&tag_id) {
        return Ok(command_error_result(err));
    }

    let db = {
        let guard = app_manager.lock().await;
        guard.get_database().clone()
    };

    match db.remove_tag_from_todo(&todo_id, &tag_id).await {
        Ok(_) => Ok(CommandResult::success(())),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 获取待办事项的所有标签
#[tauri::command]
pub async fn get_todo_tags(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    todo_id: String,
) -> Result<CommandResult<Vec<(String, String, String)>>, CommandError> {
    // 验证ID
    if let Err(err) = validate_id(&todo_id) {
        return Ok(command_error_result(err));
    }

    let db = {
        let guard = app_manager.lock().await;
        guard.get_database().clone()
    };

    match db.get_todo_tags(&todo_id).await {
        Ok(tags) => Ok(CommandResult::success(tags)),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 切换任务状态
#[tauri::command]
pub async fn toggle_todo_status(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    id: String,
) -> Result<CommandResult<Todo>, CommandError> {
    // 验证ID
    if let Err(err) = validate_id(&id) {
        return Ok(command_error_result(err));
    }

    let todo = {
        let mut guard = app_manager.lock().await;
        guard.toggle_todo_status(&id).await
    };

    match todo {
        Ok(todo) => Ok(CommandResult::success(todo)),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 显式设置任务状态
#[tauri::command]
pub async fn set_todo_status(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    id: String,
    status: String,
) -> Result<CommandResult<Todo>, CommandError> {
    if let Err(err) = validate_id(&id) {
        return Ok(command_error_result(err));
    }
    let parsed_status = match TodoStatus::from_string(&status) {
        Ok(status) => status,
        Err(err) => return Ok(command_error_result(err)),
    };

    let todo = {
        let mut guard = app_manager.lock().await;
        guard.set_todo_status(&id, parsed_status).await
    };

    match todo {
        Ok(todo) => Ok(CommandResult::success(todo)),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 关联待办事项与 GitHub Issue / Project
#[tauri::command]
pub async fn link_todo_github(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    id: String,
    issue_id: i64,
    issue_number: i64,
    project_id: i64,
) -> Result<CommandResult<Todo>, CommandError> {
    if let Err(err) = validate_id(&id) {
        return Ok(command_error_result(err));
    }
    if let Err(err) = validate_positive_i64(issue_id, "issue_id") {
        return Ok(command_error_result(err));
    }
    if let Err(err) = validate_positive_i64(issue_number, "issue_number") {
        return Ok(command_error_result(err));
    }
    if let Err(err) = validate_positive_i64(project_id, "project_id") {
        return Ok(command_error_result(err));
    }

    let todo = {
        let mut guard = app_manager.lock().await;
        guard
            .link_todo_github(&id, issue_id, issue_number, project_id)
            .await
    };

    match todo {
        Ok(todo) => Ok(CommandResult::success(todo)),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 清除待办事项的 GitHub 关联
#[tauri::command]
pub async fn clear_todo_github_link(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    id: String,
) -> Result<CommandResult<Todo>, CommandError> {
    if let Err(err) = validate_id(&id) {
        return Ok(command_error_result(err));
    }

    let todo = {
        let mut guard = app_manager.lock().await;
        guard.clear_todo_github_link(&id).await
    };

    match todo {
        Ok(todo) => Ok(CommandResult::success(todo)),
        Err(e) => Ok(command_error_result(e)),
    }
}

/// 获取任务统计
#[tauri::command]
pub async fn get_todo_stats(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
) -> Result<CommandResult<serde_json::Value>, CommandError> {
    let todos = {
        let guard = app_manager.lock().await;
        guard.get_todos().await
    };

    let todos = match todos {
        Ok(todos) => todos,
        Err(e) => return Ok(command_error_result(e)),
    };

    // 计算统计信息
    let total = todos.len();
    let completed = todos
        .iter()
        .filter(|t| t.status == TodoStatus::Done)
        .count();
    let pending = total - completed;

    let stats = serde_json::json!({
        "total": total,
        "completed": completed,
        "pending": pending,
        "completion_rate": if total > 0 { (completed as f64 / total as f64 * 100.0).round() } else { 0.0 }
    });

    Ok(CommandResult::success(stats))
}

#[cfg(test)]
mod tests {
    use super::validate_positive_i64;

    #[test]
    fn validate_positive_i64_accepts_positive_values() {
        assert!(validate_positive_i64(1, "issue_id").is_ok());
    }

    #[test]
    fn validate_positive_i64_rejects_zero_and_negative_values() {
        assert!(validate_positive_i64(0, "issue_id").is_err());
        assert!(validate_positive_i64(-1, "issue_id").is_err());
    }
}
