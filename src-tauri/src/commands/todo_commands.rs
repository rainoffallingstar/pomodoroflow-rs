//! 待办事项相关的 Tauri 命令

use super::validators::{validate_id, validate_todo_description, validate_todo_title};
use super::{CommandError, CommandResult};
use pomoflow_rs::{PomodoroAppManager, Todo, TodoStatus, TodoUpdate};
use pomoflow_rs::core::Tag;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

/// 获取所有待办事项
#[tauri::command]
pub async fn get_todos(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
) -> Result<CommandResult<Vec<Todo>>, CommandError> {
    let todos = {
        let guard = app_manager.lock().await;
        guard.get_todos().await
    };

    let todos = todos.map_err(|e| CommandError::from(e))?;
    Ok(CommandResult::success(todos))
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
    validate_todo_title(&title)?;

    if let Some(ref desc) = description {
        validate_todo_description(desc)?;
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

    let todo = todo.map_err(|e| CommandError::from(e))?;
    Ok(CommandResult::success(todo))
}

/// 更新任务
#[tauri::command]
pub async fn update_todo(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    id: String,
    title: Option<String>,
    description: Option<String>,
) -> Result<CommandResult<Todo>, CommandError> {
    // 验证ID
    validate_id(&id)?;

    // 验证输入
    if let Some(ref title) = title {
        validate_todo_title(title)?;
    }

    if let Some(ref desc) = description {
        validate_todo_description(desc)?;
    }

    // 创建更新对象
    let mut updates = TodoUpdate::new();

    if let Some(title) = title {
        updates = updates.with_title(title);
    }

    if let Some(description) = description {
        updates = updates.with_description(Some(description));
    }

    let todo = {
        let mut guard = app_manager.lock().await;
        guard.update_todo(&id, updates).await
    };

    let todo = todo.map_err(|e| CommandError::from(e))?;
    Ok(CommandResult::success(todo))
}

/// 删除任务
#[tauri::command]
pub async fn delete_todo(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    id: String,
) -> Result<CommandResult<bool>, CommandError> {
    // 验证ID
    validate_id(&id)?;

    {
        let mut guard = app_manager.lock().await;
        guard.delete_todo(&id).await
    }
    .map_err(|e| CommandError::from(e))?;

    Ok(CommandResult::success(true))
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

    let tags = db.get_all_tags().await.map_err(|e| CommandError::from(e))?;
    Ok(CommandResult::success(tags))
}

/// 创建标签
#[tauri::command]
pub async fn create_tag(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    name: String,
    color: String,
) -> Result<CommandResult<(String, String)>, CommandError> {
    // 验证输入
    if name.trim().is_empty() {
        return Err(CommandError::Validation("标签名称不能为空".to_string()));
    }

    if name.len() > 50 {
        return Err(CommandError::Validation("标签名称不能超过50字符".to_string()));
    }

    // 验证颜色格式（简单的 hex 颜色代码）
    if !color.starts_with('#') || color.len() != 7 {
        return Err(CommandError::Validation("颜色格式无效，应为 #RRGGBB 格式".to_string()));
    }

    let db = {
        let guard = app_manager.lock().await;
        guard.get_database().clone()
    };

    let tag = db.create_tag(&name, &color).await.map_err(|e| CommandError::from(e))?;
    Ok(CommandResult::success(tag))
}

/// 删除标签
#[tauri::command]
pub async fn delete_tag(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    id: String,
) -> Result<CommandResult<bool>, CommandError> {
    // 验证ID
    validate_id(&id)?;

    let db = {
        let guard = app_manager.lock().await;
        guard.get_database().clone()
    };

    let deleted = db.delete_tag(&id).await.map_err(|e| CommandError::from(e))?;
    Ok(CommandResult::success(deleted))
}

/// 为待办事项添加标签
#[tauri::command]
pub async fn assign_tag_to_todo(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    todo_id: String,
    tag_id: String,
) -> Result<CommandResult<()>, CommandError> {
    // 验证ID
    validate_id(&todo_id)?;
    validate_id(&tag_id)?;

    let db = {
        let guard = app_manager.lock().await;
        guard.get_database().clone()
    };

    db.add_tag_to_todo(&todo_id, &tag_id).await.map_err(|e| CommandError::from(e))?;
    Ok(CommandResult::success(()))
}

/// 从待办事项移除标签
#[tauri::command]
pub async fn remove_tag_from_todo(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    todo_id: String,
    tag_id: String,
) -> Result<CommandResult<()>, CommandError> {
    // 验证ID
    validate_id(&todo_id)?;
    validate_id(&tag_id)?;

    let db = {
        let guard = app_manager.lock().await;
        guard.get_database().clone()
    };

    db.remove_tag_from_todo(&todo_id, &tag_id).await.map_err(|e| CommandError::from(e))?;
    Ok(CommandResult::success(()))
}

/// 获取待办事项的所有标签
#[tauri::command]
pub async fn get_todo_tags(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    todo_id: String,
) -> Result<CommandResult<Vec<(String, String, String)>>, CommandError> {
    // 验证ID
    validate_id(&todo_id)?;

    let db = {
        let guard = app_manager.lock().await;
        guard.get_database().clone()
    };

    let tags = db.get_todo_tags(&todo_id).await.map_err(|e| CommandError::from(e))?;
    Ok(CommandResult::success(tags))
}

/// 切换任务状态
#[tauri::command]
pub async fn toggle_todo_status(
    app_manager: State<'_, Arc<Mutex<PomodoroAppManager>>>,
    id: String,
) -> Result<CommandResult<Todo>, CommandError> {
    // 验证ID
    validate_id(&id)?;

    let todo = {
        let mut guard = app_manager.lock().await;
        guard.toggle_todo_status(&id).await
    };

    let todo = todo.map_err(|e| CommandError::from(e))?;
    Ok(CommandResult::success(todo))
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

    let todos = todos.map_err(|e| CommandError::from(e))?;

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
