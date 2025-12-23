//! 任务管理模块单元测试

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_todo_status_to_string() {
        assert_eq!(TodoStatus::Todo.to_string(), "待办");
        assert_eq!(TodoStatus::InProgress.to_string(), "进行中");
        assert_eq!(TodoStatus::Done.to_string(), "已完成");
    }

    #[test]
    fn test_todo_status_from_string() {
        assert_eq!(TodoStatus::from_string("todo").unwrap(), TodoStatus::Todo);
        assert_eq!(TodoStatus::from_string("in_progress").unwrap(), TodoStatus::InProgress);
        assert_eq!(TodoStatus::from_string("done").unwrap(), TodoStatus::Done);
    }

    #[test]
    fn test_todo_status_from_string_invalid() {
        assert!(TodoStatus::from_string("invalid").is_err());
    }

    #[test]
    fn test_todo_new() {
        let todo = Todo::new("测试任务".to_string(), Some("任务描述".to_string()));

        assert!(!todo.id.is_empty());
        assert_eq!(todo.title, "测试任务");
        assert_eq!(todo.description, Some("任务描述".to_string()));
        assert_eq!(todo.status, TodoStatus::Todo);
        assert!(todo.github_issue_id.is_none());
        assert!(todo.github_issue_number.is_none());
        assert!(todo.github_project_id.is_none());
        assert!(todo.sync_pending);
    }

    #[test]
    fn test_todo_new_with_empty_description() {
        let todo = Todo::new("测试任务".to_string(), None);

        assert_eq!(todo.description, None);
    }

    #[test]
    fn test_todo_update_title() {
        let mut todo = Todo::new("原始标题".to_string(), None);
        let original_updated_at = todo.updated_at;

        // 等待一点时间确保 updated_at 会变化
        std::thread::sleep(std::time::Duration::from_millis(10));

        todo.update_title("新标题".to_string());

        assert_eq!(todo.title, "新标题");
        assert!(todo.updated_at > original_updated_at);
        assert!(todo.sync_pending);
    }

    #[test]
    fn test_todo_update_description() {
        let mut todo = Todo::new("测试任务".to_string(), None);
        let original_updated_at = todo.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));

        todo.update_description(Some("新描述".to_string()));

        assert_eq!(todo.description, Some("新描述".to_string()));
        assert!(todo.updated_at > original_updated_at);
        assert!(todo.sync_pending);
    }

    #[test]
    fn test_todo_update_description_to_none() {
        let mut todo = Todo::new("测试任务".to_string(), Some("原描述".to_string()));

        todo.update_description(None);

        assert_eq!(todo.description, None);
        assert!(todo.sync_pending);
    }

    #[test]
    fn test_todo_update_status() {
        let mut todo = Todo::new("测试任务".to_string(), None);

        todo.update_status(TodoStatus::InProgress);
        assert_eq!(todo.status, TodoStatus::InProgress);
        assert!(todo.sync_pending);

        todo.update_status(TodoStatus::Done);
        assert_eq!(todo.status, TodoStatus::Done);
        assert!(todo.sync_pending);
    }

    #[test]
    fn test_todo_toggle_status() {
        let mut todo = Todo::new("测试任务".to_string(), None);

        assert_eq!(todo.status, TodoStatus::Todo);

        todo.toggle_status();
        assert_eq!(todo.status, TodoStatus::InProgress);

        todo.toggle_status();
        assert_eq!(todo.status, TodoStatus::Done);

        todo.toggle_status();
        assert_eq!(todo.status, TodoStatus::Todo);
    }

    #[test]
    fn test_todo_mark_synced() {
        let mut todo = Todo::new("测试任务".to_string(), None);

        assert!(todo.sync_pending);

        todo.mark_synced();
        assert!(!todo.sync_pending);
    }

    #[test]
    fn test_todo_set_github_info() {
        let mut todo = Todo::new("测试任务".to_string(), None);

        todo.set_github_info(12345, 1, 67890);

        assert_eq!(todo.github_issue_id, Some(12345));
        assert_eq!(todo.github_issue_number, Some(1));
        assert_eq!(todo.github_project_id, Some(67890));
        assert!(!todo.sync_pending);
    }

    #[test]
    fn test_todo_is_done() {
        let mut todo = Todo::new("测试任务".to_string(), None);

        assert!(!todo.is_done());

        todo.update_status(TodoStatus::Done);
        assert!(todo.is_done());
    }

    #[test]
    fn test_todo_needs_sync() {
        let mut todo = Todo::new("测试任务".to_string(), None);

        assert!(todo.needs_sync());

        todo.mark_synced();
        assert!(!todo.needs_sync());
    }

    #[test]
    fn test_new_todo_validate_valid() {
        let new_todo = NewTodo {
            title: "有效任务".to_string(),
            description: Some("任务描述".to_string()),
        };

        assert!(new_todo.validate().is_ok());
    }

    #[test]
    fn test_new_todo_validate_empty_title() {
        let new_todo = NewTodo {
            title: "".to_string(),
            description: None,
        };

        assert!(new_todo.validate().is_err());
    }

    #[test]
    fn test_new_todo_validate_title_too_long() {
        let new_todo = NewTodo {
            title: "a".repeat(201),
            description: None,
        };

        assert!(new_todo.validate().is_err());
    }

    #[test]
    fn test_new_todo_validate_description_too_long() {
        let new_todo = NewTodo {
            title: "有效任务".to_string(),
            description: Some("a".repeat(5001)),
        };

        assert!(new_todo.validate().is_err());
    }

    #[test]
    fn test_todo_update_new() {
        let update = TodoUpdate::new();

        assert!(!update.has_updates());
    }

    #[test]
    fn test_todo_update_with_title() {
        let update = TodoUpdate::new().with_title("新标题".to_string());

        assert!(update.has_updates());
        assert_eq!(update.title, Some("新标题".to_string()));
    }

    #[test]
    fn test_todo_update_with_description() {
        let update = TodoUpdate::new().with_description(Some("新描述".to_string()));

        assert!(update.has_updates());
        assert_eq!(update.description, Some(Some("新描述".to_string())));
    }

    #[test]
    fn test_todo_update_with_status() {
        let update = TodoUpdate::new().with_status(TodoStatus::Done);

        assert!(update.has_updates());
        assert_eq!(update.status, Some(TodoStatus::Done));
    }

    #[test]
    fn test_todo_filter_default() {
        let filter = TodoFilter::default();

        assert_eq!(filter.status, None);
        assert_eq!(filter.search, None);
        assert!(filter.show_completed);
        assert_eq!(filter.limit, None);
    }

    #[test]
    fn test_todo_filter_all() {
        let filter = TodoFilter::all();

        assert_eq!(filter.status, None);
        assert_eq!(filter.search, None);
        assert!(filter.show_completed);
        assert_eq!(filter.limit, None);
    }

    #[test]
    fn test_todo_filter_pending() {
        let filter = TodoFilter::pending();

        assert_eq!(filter.status, Some(TodoStatus::Todo));
        assert!(!filter.show_completed);
    }

    #[test]
    fn test_todo_filter_completed() {
        let filter = TodoFilter::completed();

        assert_eq!(filter.status, Some(TodoStatus::Done));
        assert!(filter.show_completed);
    }

    #[test]
    fn test_todo_filter_with_search() {
        let filter = TodoFilter::all().with_search("测试".to_string());

        assert_eq!(filter.search, Some("测试".to_string()));
    }

    #[test]
    fn test_todo_filter_with_status() {
        let filter = TodoFilter::all().with_status(TodoStatus::InProgress);

        assert_eq!(filter.status, Some(TodoStatus::InProgress));
    }

    #[test]
    fn test_todo_filter_with_limit() {
        let filter = TodoFilter::all().with_limit(10);

        assert_eq!(filter.limit, Some(10));
    }

    #[test]
    fn test_todo_filter_apply_no_filter() {
        let todos = vec![
            Todo::new("任务1".to_string(), None),
            Todo::new("任务2".to_string(), None),
        ];

        let filter = TodoFilter::all();
        let filtered = filter.apply(&todos);

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_todo_filter_apply_with_status() {
        let mut todo1 = Todo::new("任务1".to_string(), None);
        todo1.update_status(TodoStatus::Done);

        let mut todo2 = Todo::new("任务2".to_string(), None);
        todo2.update_status(TodoStatus::Todo);

        let todos = vec![todo1, todo2];

        let filter = TodoFilter::pending();
        let filtered = filter.apply(&todos);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].status, TodoStatus::Todo);
    }

    #[test]
    fn test_todo_filter_apply_with_search() {
        let todo1 = Todo::new("测试任务1".to_string(), None);
        let todo2 = Todo::new("其他任务".to_string(), None);
        let todo3 = Todo::new("测试任务2".to_string(), None);

        let todos = vec![todo1, todo2, todo3];

        let filter = TodoFilter::all().with_search("测试".to_string());
        let filtered = filter.apply(&todos);

        assert_eq!(filtered.len(), 2);
        assert!(filtered[0].title.contains("测试"));
        assert!(filtered[1].title.contains("测试"));
    }

    #[test]
    fn test_todo_filter_apply_with_limit() {
        let todos = vec![
            Todo::new("任务1".to_string(), None),
            Todo::new("任务2".to_string(), None),
            Todo::new("任务3".to_string(), None),
        ];

        let filter = TodoFilter::all().with_limit(2);
        let filtered = filter.apply(&todos);

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_todo_stats_from_todos() {
        let mut todo1 = Todo::new("任务1".to_string(), None);
        todo1.update_status(TodoStatus::Done);

        let mut todo2 = Todo::new("任务2".to_string(), None);
        todo2.update_status(TodoStatus::Todo);

        let mut todo3 = Todo::new("任务3".to_string(), None);
        todo3.update_status(TodoStatus::InProgress);

        let todos = vec![todo1, todo2, todo3];

        let stats = TodoStats::from_todos(&todos);

        assert_eq!(stats.total, 3);
        assert_eq!(stats.todo, 1);
        assert_eq!(stats.in_progress, 1);
        assert_eq!(stats.done, 1);
        assert_eq!(stats.needs_sync, 3);
    }

    #[test]
    fn test_todo_stats_completion_rate() {
        let mut todo1 = Todo::new("任务1".to_string(), None);
        todo1.update_status(TodoStatus::Done);

        let mut todo2 = Todo::new("任务2".to_string(), None);
        todo2.update_status(TodoStatus::Todo);

        let todos = vec![todo1, todo2];

        let stats = TodoStats::from_todos(&todos);

        assert_eq!(stats.completion_rate(), 50.0);
    }

    #[test]
    fn test_todo_stats_completion_rate_empty() {
        let stats = TodoStats::from_todos(&[]);

        assert_eq!(stats.completion_rate(), 0.0);
    }

    #[test]
    fn test_todo_export() {
        let todo = Todo::new("测试任务".to_string(), None);
        let todos = vec![todo];

        let export = TodoExport::export(&todos);

        assert_eq!(export.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(export.todos.len(), 1);
        assert_eq!(export.stats.total, 1);
    }
}
