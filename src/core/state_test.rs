//! 状态管理模块单元测试

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[test]
    fn test_user_config_default() {
        let config = UserConfig::default();

        assert_eq!(config.github_token_encrypted, "");
        assert_eq!(config.github_username, "");
        assert_eq!(config.pomodoro_work_duration, 1500);
        assert_eq!(config.pomodoro_short_break_duration, 300);
        assert_eq!(config.pomodoro_long_break_duration, 900);
        assert_eq!(config.pomodoro_cycles_until_long_break, 4);
        assert!(config.notifications_enabled);
        assert!(config.sound_enabled);
        assert!(config.system_notifications);
        assert_eq!(config.theme, "light");
    }

    #[test]
    fn test_github_project_default() {
        let project = GithubProject {
            id: 0,
            owner: "test".to_string(),
            repo: "test-repo".to_string(),
            number: 1,
            name: "Test Project".to_string(),
        };

        assert_eq!(project.owner, "test");
        assert_eq!(project.repo, "test-repo");
        assert_eq!(project.number, 1);
        assert_eq!(project.name, "Test Project");
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();

        assert!(state.todos.is_empty());
        assert_eq!(state.todo_filter, TodoFilter::all());
        assert_eq!(state.todo_stats.total, 0);
        assert!(state.pomodoro_session.is_none());
        assert_eq!(state.pomodoro_config, PomodoroConfig::default());
        assert!(state.user_config.is_none());
        assert!(state.selected_github_project.is_none());
        assert!(state.is_online);
        assert!(!state.pending_sync);
        assert!(!state.sync_in_progress);
        assert!(state.error_message.is_none());
        assert!(state.info_message.is_none());
    }

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();

        assert_eq!(state, AppState::default());
    }

    #[tokio::test]
    async fn test_app_state_get_filtered_todos() {
        let state = AppState::default();

        let filtered = state.get_filtered_todos();

        assert!(filtered.is_empty());
    }

    #[tokio::test]
    async fn test_app_state_get_pending_todo_count() {
        let mut state = AppState::default();

        assert_eq!(state.get_pending_todo_count(), 0);

        let todo1 = Todo::new("任务1".to_string(), None);
        let todo2 = Todo::new("任务2".to_string(), None);
        let mut todo3 = Todo::new("任务3".to_string(), None);
        todo3.update_status(TodoStatus::Done);

        state.todos = vec![todo1, todo2, todo3];

        assert_eq!(state.get_pending_todo_count(), 2);
    }

    #[tokio::test]
    async fn test_app_state_get_completed_todo_count() {
        let mut state = AppState::default();

        assert_eq!(state.get_completed_todo_count(), 0);

        let todo1 = Todo::new("任务1".to_string(), None);
        let mut todo2 = Todo::new("任务2".to_string(), None);
        todo2.update_status(TodoStatus::Done);
        let mut todo3 = Todo::new("任务3".to_string(), None);
        todo3.update_status(TodoStatus::Done);

        state.todos = vec![todo1, todo2, todo3];

        assert_eq!(state.get_completed_todo_count(), 2);
    }

    #[tokio::test]
    async fn test_app_state_has_unsynced_todos() {
        let state = AppState::default();

        assert!(!state.has_unsynced_todos());
    }

    #[tokio::test]
    async fn test_app_state_set_error() {
        let mut state = AppState::default();

        state.set_error(Some("测试错误".to_string()));

        assert_eq!(state.error_message, Some("测试错误".to_string()));

        state.set_error(None);
        assert!(state.error_message.is_none());
    }

    #[tokio::test]
    async fn test_app_state_set_info() {
        let mut state = AppState::default();

        state.set_info(Some("测试信息".to_string()));

        assert_eq!(state.info_message, Some("测试信息".to_string()));

        state.set_info(None);
        assert!(state.info_message.is_none());
    }

    #[tokio::test]
    async fn test_app_state_clear_messages() {
        let mut state = AppState::default();

        state.set_error(Some("错误".to_string()));
        state.set_info(Some("信息".to_string()));

        assert!(state.error_message.is_some());
        assert!(state.info_message.is_some());

        state.clear_messages();

        assert!(state.error_message.is_none());
        assert!(state.info_message.is_none());
    }

    #[tokio::test]
    async fn test_app_state_manager_new() {
        let manager = AppStateManager::new();

        assert!(!manager.is_running());
    }

    #[tokio::test]
    async fn test_app_state_manager_get_state() {
        let manager = AppStateManager::new();

        let state = manager.get_state().await;

        assert!(state.todos.is_empty());
    }

    #[tokio::test]
    async fn test_app_state_manager_get_state_mut() {
        let manager = AppStateManager::new();

        let mut state = manager.get_state_mut().await;
        state.set_error(Some("测试错误".to_string()));

        let state = manager.get_state().await;
        assert_eq!(state.error_message, Some("测试错误".to_string()));
    }

    #[tokio::test]
    async fn test_app_state_manager_send_event() {
        let manager = AppStateManager::new();

        let result = manager.send_event(AppEvent::InfoMessage("测试消息".to_string()));

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_app_state_manager_add_todo() {
        let manager = AppStateManager::new();

        let todo = Todo::new("测试任务".to_string(), None);
        manager.add_todo(todo).await.unwrap();

        let state = manager.get_state().await;
        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "测试任务");
    }

    #[tokio::test]
    async fn test_app_state_manager_update_todo() {
        let manager = AppStateManager::new();

        let todo = Todo::new("原始标题".to_string(), None);
        manager.add_todo(todo.clone()).await.unwrap();

        let updates = TodoUpdate::new().with_title("新标题".to_string());
        let updated = manager.update_todo(&todo.id, updates).await.unwrap().unwrap();

        assert_eq!(updated.title, "新标题");
    }

    #[tokio::test]
    async fn test_app_state_manager_delete_todo() {
        let manager = AppStateManager::new();

        let todo = Todo::new("测试任务".to_string(), None);
        manager.add_todo(todo.clone()).await.unwrap();

        let deleted = manager.delete_todo(&todo.id).await.unwrap();

        assert!(deleted);

        let state = manager.get_state().await;
        assert_eq!(state.todos.len(), 0);
    }

    #[tokio::test]
    async fn test_app_state_manager_toggle_todo_status() {
        let manager = AppStateManager::new();

        let todo = Todo::new("测试任务".to_string(), None);
        manager.add_todo(todo.clone()).await.unwrap();

        let updated = manager.toggle_todo_status(&todo.id).await.unwrap().unwrap();

        assert_eq!(updated.status, TodoStatus::InProgress);
    }

    #[tokio::test]
    async fn test_app_state_manager_bulk_update_todos() {
        let manager = AppStateManager::new();

        let todos = vec![
            Todo::new("任务1".to_string(), None),
            Todo::new("任务2".to_string(), None),
        ];

        manager.bulk_update_todos(todos.clone()).await.unwrap();

        let state = manager.get_state().await;
        assert_eq!(state.todos.len(), 2);
        assert_eq!(state.todos[0].title, "任务1");
    }

    #[tokio::test]
    async fn test_app_state_manager_set_pomodoro_session() {
        let manager = AppStateManager::new();

        let config = PomodoroConfig::default();
        let session = PomodoroSession::new(config);

        manager.set_pomodoro_session(session.clone()).await;

        let retrieved_session = manager.get_pomodoro_session().await.unwrap();
        assert_eq!(retrieved_session.phase, session.phase);
    }

    #[tokio::test]
    async fn test_app_state_manager_update_pomodoro_config() {
        let manager = AppStateManager::new();

        let mut config = PomodoroConfig::default();
        config.work_duration = 1800;

        manager.update_pomodoro_config(config.clone()).await.unwrap();

        let state = manager.get_state().await;
        assert_eq!(state.pomodoro_config.work_duration, 1800);
    }

    #[tokio::test]
    async fn test_app_state_manager_set_user_config() {
        let manager = AppStateManager::new();

        let config = UserConfig {
            github_username: "test_user".to_string(),
            ..Default::default()
        };

        manager.set_user_config(config.clone()).await;

        let retrieved_config = manager.get_user_config().await.unwrap();
        assert_eq!(retrieved_config.github_username, "test_user");
    }

    #[tokio::test]
    async fn test_app_state_manager_set_github_project() {
        let manager = AppStateManager::new();

        let project = GithubProject {
            id: 1,
            owner: "test".to_string(),
            repo: "test-repo".to_string(),
            number: 1,
            name: "Test Project".to_string(),
        };

        manager.set_github_project(project.clone()).await;

        let retrieved_project = manager.get_github_project().await.unwrap();
        assert_eq!(retrieved_project.owner, "test");
    }

    #[tokio::test]
    async fn test_app_state_manager_set_sync_in_progress() {
        let manager = AppStateManager::new();

        manager.set_sync_in_progress(true).await;

        let state = manager.get_state().await;
        assert!(state.sync_in_progress);
    }

    #[tokio::test]
    async fn test_app_state_manager_mark_sync_completed() {
        let manager = AppStateManager::new();

        manager.set_sync_in_progress(true).await;
        manager.mark_sync_completed().await;

        let state = manager.get_state().await;
        assert!(!state.sync_in_progress);
        assert!(!state.pending_sync);
    }

    #[tokio::test]
    async fn test_app_state_manager_mark_sync_failed() {
        let manager = AppStateManager::new();

        manager.set_sync_in_progress(true).await;
        manager.mark_sync_failed("测试错误".to_string()).await;

        let state = manager.get_state().await;
        assert!(!state.sync_in_progress);
        assert!(state.pending_sync);
    }

    #[tokio::test]
    async fn test_app_state_manager_set_network_status() {
        let manager = AppStateManager::new();

        manager.set_network_status(false).await;

        let is_online = manager.get_network_status().await;
        assert!(!is_online);
    }

    #[tokio::test]
    async fn test_app_state_manager_set_error_message() {
        let manager = AppStateManager::new();

        manager.set_error_message("测试错误".to_string()).await;

        let state = manager.get_state().await;
        assert_eq!(state.error_message, Some("测试错误".to_string()));
    }

    #[tokio::test]
    async fn test_app_state_manager_set_info_message() {
        let manager = AppStateManager::new();

        manager.set_info_message("测试信息".to_string()).await;

        let state = manager.get_state().await;
        assert_eq!(state.info_message, Some("测试信息".to_string()));
    }

    #[tokio::test]
    async fn test_app_state_manager_clear_messages() {
        let manager = AppStateManager::new();

        manager.set_error_message("错误".to_string()).await;
        manager.set_info_message("信息".to_string()).await;

        manager.clear_messages().await;

        let state = manager.get_state().await;
        assert!(state.error_message.is_none());
        assert!(state.info_message.is_none());
    }

    #[tokio::test]
    async fn test_app_state_manager_set_todo_filter() {
        let manager = AppStateManager::new();

        let filter = TodoFilter::pending();
        manager.set_todo_filter(filter.clone()).await;

        let state = manager.get_state().await;
        assert_eq!(state.todo_filter.status, Some(TodoStatus::Todo));
    }

    #[tokio::test]
    async fn test_app_state_manager_get_all_todos() {
        let manager = AppStateManager::new();

        let todos = vec![
            Todo::new("任务1".to_string(), None),
            Todo::new("任务2".to_string(), None),
        ];

        manager.bulk_update_todos(todos.clone()).await.unwrap();

        let retrieved_todos = manager.get_all_todos().await;

        assert_eq!(retrieved_todos.len(), 2);
    }

    #[tokio::test]
    async fn test_app_state_manager_get_filtered_todos() {
        let manager = AppStateManager::new();

        let todos = vec![
            Todo::new("任务1".to_string(), None),
            Todo::new("任务2".to_string(), None),
        ];

        manager.bulk_update_todos(todos).await.unwrap();

        let filtered_todos = manager.get_filtered_todos().await;

        assert_eq!(filtered_todos.len(), 2);
    }

    #[tokio::test]
    async fn test_app_state_manager_get_todo_stats() {
        let manager = AppStateManager::new();

        let todos = vec![
            Todo::new("任务1".to_string(), None),
            Todo::new("任务2".to_string(), None),
        ];

        manager.bulk_update_todos(todos).await.unwrap();

        let stats = manager.get_todo_stats().await;

        assert_eq!(stats.total, 2);
    }

    #[tokio::test]
    async fn test_app_state_manager_get_pending_sync_count() {
        let manager = AppStateManager::new();

        let count = manager.get_pending_sync_count().await;

        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_event_receiver_new() {
        let (_, receiver) = mpsc::unbounded_channel::<AppEvent>();

        let event_receiver = EventReceiver::new(receiver);

        // 这里只是测试可以创建实例
        assert!(!event_receiver.receiver.is_closed());
    }

    #[test]
    fn test_app_event_variants() {
        // 测试所有 AppEvent 变体都可以创建
        let _ = AppEvent::InfoMessage("test".to_string());
        let _ = AppEvent::ErrorMessage("test".to_string());
        let _ = AppEvent::SettingsUpdated;
        let _ = AppEvent::MessageCleared;
    }

    #[test]
    fn test_state_query_variants() {
        // 测试所有 StateQuery 变体都可以创建
        let _ = StateQuery::GetTodos;
        let _ = StateQuery::GetFilteredTodos;
        let _ = StateQuery::GetTodoStats;
        let _ = StateQuery::GetPomodoroSession;
        let _ = StateQuery::GetUserConfig;
        let _ = StateQuery::GetGithubProject;
        let _ = StateQuery::GetIsOnline;
        let _ = StateQuery::GetPendingSyncCount;
    }

    #[test]
    fn test_state_query_response_variants() {
        // 测试所有 StateQueryResponse 变体都可以创建
        let _ = StateQueryResponse::Todos(vec![]);
        let _ = StateQueryResponse::FilteredTodos(vec![]);
        let _ = StateQueryResponse::TodoStats(TodoStats::from_todos(&[]));
        let _ = StateQueryResponse::PomodoroSession(None);
        let _ = StateQueryResponse::UserConfig(None);
        let _ = StateQueryResponse::GithubProject(None);
        let _ = StateQueryResponse::IsOnline(true);
        let _ = StateQueryResponse::PendingSyncCount(0);
    }
}
