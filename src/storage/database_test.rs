//! 数据库模块单元测试

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::Path;

    async fn create_test_database() -> Database {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        Database::init(&db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_database_init() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = Database::init(&db_path).await.unwrap();

        assert!(db_path.exists());
    }

    #[tokio::test]
    async fn test_save_and_load_user_config() {
        let db = create_test_database().await;

        let config = UserConfig {
            github_token_encrypted: "test_token".to_string(),
            github_username: "test_user".to_string(),
            selected_project_owner: Some("owner".to_string()),
            selected_project_repo: Some("repo".to_string()),
            selected_project_number: Some(1),
            pomodoro_work_duration: 1800,
            pomodoro_short_break_duration: 600,
            pomodoro_long_break_duration: 1200,
            pomodoro_cycles_until_long_break: 3,
            notifications_enabled: false,
            sound_enabled: true,
            system_notifications: false,
            theme: "dark".to_string(),
        };

        db.save_user_config(&config).await.unwrap();

        let loaded_config = db.load_user_config().await.unwrap().unwrap();

        assert_eq!(loaded_config.github_token_encrypted, "test_token");
        assert_eq!(loaded_config.github_username, "test_user");
        assert_eq!(loaded_config.selected_project_owner, Some("owner".to_string()));
        assert_eq!(loaded_config.selected_project_repo, Some("repo".to_string()));
        assert_eq!(loaded_config.selected_project_number, Some(1));
        assert_eq!(loaded_config.pomodoro_work_duration, 1800);
        assert_eq!(loaded_config.pomodoro_short_break_duration, 600);
        assert_eq!(loaded_config.pomodoro_long_break_duration, 1200);
        assert_eq!(loaded_config.pomodoro_cycles_until_long_break, 3);
        assert_eq!(loaded_config.notifications_enabled, false);
        assert_eq!(loaded_config.sound_enabled, true);
        assert_eq!(loaded_config.system_notifications, false);
        assert_eq!(loaded_config.theme, "dark");
    }

    #[tokio::test]
    async fn test_load_user_config_empty() {
        let db = create_test_database().await;

        let config = db.load_user_config().await.unwrap();

        assert!(config.is_some());
    }

    #[tokio::test]
    async fn test_create_todo() {
        let db = create_test_database().await;

        let new_todo = NewTodo {
            title: "测试任务".to_string(),
            description: Some("任务描述".to_string()),
        };

        let todo = db.create_todo(&new_todo).await.unwrap();

        assert_eq!(todo.title, "测试任务");
        assert_eq!(todo.description, Some("任务描述".to_string()));
        assert_eq!(todo.status, TodoStatus::Todo);
        assert!(todo.sync_pending);
    }

    #[tokio::test]
    async fn test_create_todo_invalid_title() {
        let db = create_test_database().await;

        let new_todo = NewTodo {
            title: "".to_string(),
            description: None,
        };

        let result = db.create_todo(&new_todo).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_todo_title_too_long() {
        let db = create_test_database().await;

        let new_todo = NewTodo {
            title: "a".repeat(201),
            description: None,
        };

        let result = db.create_todo(&new_todo).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_all_todos() {
        let db = create_test_database().await;

        // 创建多个任务
        for i in 1..=5 {
            let new_todo = NewTodo {
                title: format!("任务 {}", i),
                description: None,
            };
            db.create_todo(&new_todo).await.unwrap();
        }

        let todos = db.get_all_todos().await.unwrap();

        assert_eq!(todos.len(), 5);
    }

    #[tokio::test]
    async fn test_get_todo_by_id() {
        let db = create_test_database().await;

        let new_todo = NewTodo {
            title: "测试任务".to_string(),
            description: None,
        };
        let created_todo = db.create_todo(&new_todo).await.unwrap();

        let retrieved_todo = db.get_todo_by_id(&created_todo.id).await.unwrap().unwrap();

        assert_eq!(retrieved_todo.id, created_todo.id);
        assert_eq!(retrieved_todo.title, created_todo.title);
    }

    #[tokio::test]
    async fn test_get_todo_by_id_not_found() {
        let db = create_test_database().await;

        let result = db.get_todo_by_id("non-existent-id").await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_update_todo_title() {
        let db = create_test_database().await;

        let new_todo = NewTodo {
            title: "原始标题".to_string(),
            description: None,
        };
        let created_todo = db.create_todo(&new_todo).await.unwrap();

        let updates = TodoUpdate::new().with_title("新标题".to_string());
        let updated_todo = db.update_todo(&created_todo.id, &updates).await.unwrap().unwrap();

        assert_eq!(updated_todo.title, "新标题");
    }

    #[tokio::test]
    async fn test_update_todo_description() {
        let db = create_test_database().await;

        let new_todo = NewTodo {
            title: "测试任务".to_string(),
            description: None,
        };
        let created_todo = db.create_todo(&new_todo).await.unwrap();

        let updates = TodoUpdate::new().with_description(Some("新描述".to_string()));
        let updated_todo = db.update_todo(&created_todo.id, &updates).await.unwrap().unwrap();

        assert_eq!(updated_todo.description, Some("新描述".to_string()));
    }

    #[tokio::test]
    async fn test_update_todo_status() {
        let db = create_test_database().await;

        let new_todo = NewTodo {
            title: "测试任务".to_string(),
            description: None,
        };
        let created_todo = db.create_todo(&new_todo).await.unwrap();

        let updates = TodoUpdate::new().with_status(TodoStatus::Done);
        let updated_todo = db.update_todo(&created_todo.id, &updates).await.unwrap().unwrap();

        assert_eq!(updated_todo.status, TodoStatus::Done);
    }

    #[tokio::test]
    async fn test_update_todo_not_found() {
        let db = create_test_database().await;

        let updates = TodoUpdate::new().with_title("新标题".to_string());
        let result = db.update_todo("non-existent-id", &updates).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_todo() {
        let db = create_test_database().await;

        let new_todo = NewTodo {
            title: "测试任务".to_string(),
            description: None,
        };
        let created_todo = db.create_todo(&new_todo).await.unwrap();

        let deleted = db.delete_todo(&created_todo.id).await.unwrap();

        assert!(deleted);

        // 验证任务已被删除
        let result = db.get_todo_by_id(&created_todo.id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_todo_not_found() {
        let db = create_test_database().await;

        let deleted = db.delete_todo("non-existent-id").await.unwrap();

        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_get_pending_sync_todos() {
        let db = create_test_database().await;

        // 创建任务
        let new_todo1 = NewTodo {
            title: "任务1".to_string(),
            description: None,
        };
        let created_todo1 = db.create_todo(&new_todo1).await.unwrap();

        let new_todo2 = NewTodo {
            title: "任务2".to_string(),
            description: None,
        };
        let created_todo2 = db.create_todo(&new_todo2).await.unwrap();

        // 标记一个任务为已同步
        db.mark_todo_synced(&created_todo2.id).await.unwrap();

        let pending_todos = db.get_pending_sync_todos().await.unwrap();

        assert_eq!(pending_todos.len(), 1);
        assert_eq!(pending_todos[0].id, created_todo1.id);
    }

    #[tokio::test]
    async fn test_mark_todo_synced() {
        let db = create_test_database().await;

        let new_todo = NewTodo {
            title: "测试任务".to_string(),
            description: None,
        };
        let created_todo = db.create_todo(&new_todo).await.unwrap();

        assert!(created_todo.sync_pending);

        db.mark_todo_synced(&created_todo.id).await.unwrap();

        // 验证任务已被标记为已同步
        let retrieved_todo = db.get_todo_by_id(&created_todo.id).await.unwrap().unwrap();
        assert!(!retrieved_todo.sync_pending);
    }

    #[tokio::test]
    async fn test_add_to_sync_queue() {
        let db = create_test_database().await;

        let payload = serde_json::json!({
            "title": "测试任务",
            "description": "任务描述"
        });

        let queue_id = db.add_to_sync_queue("create", "test-id", &payload).await.unwrap();

        assert!(queue_id > 0);
    }

    #[tokio::test]
    async fn test_get_pending_sync_queue() {
        let db = create_test_database().await;

        // 添加多个操作到队列
        let payload1 = serde_json::json!({"title": "任务1"});
        db.add_to_sync_queue("create", "id1", &payload1).await.unwrap();

        let payload2 = serde_json::json!({"title": "任务2"});
        db.add_to_sync_queue("update", "id2", &payload2).await.unwrap();

        let queue_items = db.get_pending_sync_queue().await.unwrap();

        assert_eq!(queue_items.len(), 2);
    }

    #[tokio::test]
    async fn test_mark_sync_queue_synced() {
        let db = create_test_database().await;

        let payload = serde_json::json!({"title": "测试任务"});
        let queue_id = db.add_to_sync_queue("create", "test-id", &payload).await.unwrap();

        db.mark_sync_queue_synced(queue_id).await.unwrap();

        // 验证队列项已被标记为已同步
        let queue_items = db.get_pending_sync_queue().await.unwrap();
        assert_eq!(queue_items.len(), 0);
    }

    #[tokio::test]
    async fn test_mark_sync_queue_failed() {
        let db = create_test_database().await;

        let payload = serde_json::json!({"title": "测试任务"});
        let queue_id = db.add_to_sync_queue("create", "test-id", &payload).await.unwrap();

        db.mark_sync_queue_failed(queue_id, "测试错误").await.unwrap();

        // 验证队列项已被标记为失败
        let queue_items = db.get_pending_sync_queue().await.unwrap();
        assert_eq!(queue_items.len(), 0);
    }

    #[tokio::test]
    async fn test_cleanup_sync_queue() {
        let db = create_test_database().await;

        // 添加多个操作并标记为已同步
        for i in 0..=105 {
            let payload = serde_json::json!({"title": format!("任务 {}", i)});
            let queue_id = db.add_to_sync_queue("create", &format!("id{}", i), &payload).await.unwrap();
            db.mark_sync_queue_synced(queue_id).await.unwrap();
        }

        let cleaned_count = db.cleanup_sync_queue().await.unwrap();

        assert_eq!(cleaned_count, 6); // 应该清理 106 - 100 = 6 个
    }

    #[tokio::test]
    async fn test_record_pomodoro_session() {
        let db = create_test_database().await;

        let result = db.record_pomodoro_session(PomodoroPhase::Work, 1500, 1).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_today_pomodoro_sessions() {
        let db = create_test_database().await;

        // 记录多个会话
        db.record_pomodoro_session(PomodoroPhase::Work, 1500, 1).await.unwrap();
        db.record_pomodoro_session(PomodoroPhase::ShortBreak, 300, 1).await.unwrap();
        db.record_pomodoro_session(PomodoroPhase::Work, 1500, 2).await.unwrap();

        let sessions = db.get_today_pomodoro_sessions().await.unwrap();

        assert_eq!(sessions.len(), 3);
    }

    #[tokio::test]
    async fn test_permanently_delete_todo() {
        let db = create_test_database().await;

        let new_todo = NewTodo {
            title: "测试任务".to_string(),
            description: None,
        };
        let created_todo = db.create_todo(&new_todo).await.unwrap();

        let deleted = db.permanently_delete_todo(&created_todo.id).await.unwrap();

        assert!(deleted);
    }
}
