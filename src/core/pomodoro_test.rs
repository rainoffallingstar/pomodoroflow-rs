//! 番茄钟模块单元测试

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn test_pomodoro_phase_default_duration() {
        assert_eq!(PomodoroPhase::Work.default_duration(), 1500);
        assert_eq!(PomodoroPhase::ShortBreak.default_duration(), 300);
        assert_eq!(PomodoroPhase::LongBreak.default_duration(), 900);
    }

    #[test]
    fn test_pomodoro_phase_to_string() {
        assert_eq!(PomodoroPhase::Work.to_string(), "工作");
        assert_eq!(PomodoroPhase::ShortBreak.to_string(), "短休息");
        assert_eq!(PomodoroPhase::LongBreak.to_string(), "长休息");
    }

    #[test]
    fn test_pomodoro_phase_next() {
        let config = PomodoroConfig::default();
        assert_eq!(
            PomodoroPhase::Work.next(0, config.cycles_until_long_break),
            PomodoroPhase::ShortBreak
        );
        assert_eq!(
            PomodoroPhase::Work.next(3, 4), // 第4个工作周期
            PomodoroPhase::LongBreak
        );
        assert_eq!(
            PomodoroPhase::ShortBreak.next(0, 4),
            PomodoroPhase::Work
        );
        assert_eq!(
            PomodoroPhase::LongBreak.next(0, 4),
            PomodoroPhase::Work
        );
    }

    #[test]
    fn test_pomodoro_config_default() {
        let config = PomodoroConfig::default();
        assert_eq!(config.work_duration, 1500);
        assert_eq!(config.short_break_duration, 300);
        assert_eq!(config.long_break_duration, 900);
        assert_eq!(config.cycles_until_long_break, 4);
    }

    #[test]
    fn test_pomodoro_config_validate_valid() {
        let config = PomodoroConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_pomodoro_config_validate_invalid() {
        // 工作时长过短
        let mut config = PomodoroConfig::default();
        config.work_duration = 30;
        assert!(config.validate().is_err());

        // 工作时长过长
        let mut config = PomodoroConfig::default();
        config.work_duration = 7201;
        assert!(config.validate().is_err());

        // 长休息间隔过小
        let mut config = PomodoroConfig::default();
        config.cycles_until_long_break = 1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_pomodoro_session_new() {
        let config = PomodoroConfig::default();
        let session = PomodoroSession::new(config);

        assert_eq!(session.phase, PomodoroPhase::Work);
        assert_eq!(session.duration, 1500);
        assert_eq!(session.remaining, 1500);
        assert_eq!(session.is_running, false);
        assert_eq!(session.cycle_count, 0);
    }

    #[test]
    fn test_pomodoro_session_start() {
        let config = PomodoroConfig::default();
        let mut session = PomodoroSession::new(config);

        assert!(session.start().is_ok());
        assert!(session.is_running);
    }

    #[test]
    fn test_pomodoro_session_start_already_running() {
        let config = PomodoroConfig::default();
        let mut session = PomodoroSession::new(config);

        session.start().unwrap();
        assert!(session.start().is_err());
    }

    #[test]
    fn test_pomodoro_session_pause() {
        let config = PomodoroConfig::default();
        let mut session = PomodoroSession::new(config);

        session.start().unwrap();
        assert!(session.pause().is_ok());
        assert!(!session.is_running);
    }

    #[test]
    fn test_pomodoro_session_pause_not_running() {
        let config = PomodoroConfig::default();
        let mut session = PomodoroSession::new(config);

        assert!(session.pause().is_err());
    }

    #[test]
    fn test_pomodoro_session_reset() {
        let config = PomodoroConfig::default();
        let mut session = PomodoroSession::new(config);

        session.start().unwrap();
        session.remaining = 1200;
        assert!(session.reset().is_ok());
        assert_eq!(session.remaining, 1500);
        assert!(!session.is_running);
    }

    #[test]
    fn test_pomodoro_session_formatted_time() {
        let config = PomodoroConfig::default();
        let session = PomodoroSession::new(config);

        assert_eq!(session.formatted_time(), "25:00");

        let mut session = PomodoroSession::new(config);
        session.remaining = 65;
        assert_eq!(session.formatted_time(), "01:05");
    }

    #[test]
    fn test_pomodoro_session_progress() {
        let config = PomodoroConfig::default();
        let session = PomodoroSession::new(config);

        assert_eq!(session.progress(), 0.0);

        let mut session = PomodoroSession::new(config);
        session.remaining = 750;
        assert_eq!(session.progress(), 50.0);

        let mut session = PomodoroSession::new(config);
        session.remaining = 0;
        assert_eq!(session.progress(), 100.0);
    }

    #[tokio::test]
    async fn test_pomodoro_service_new() {
        let config = PomodoroConfig::default();
        let service = PomodoroService::new(config);

        assert!(service.get_session().is_some());
    }

    #[tokio::test]
    async fn test_pomodoro_service_start_pause() {
        let config = PomodoroConfig::default();
        let mut service = PomodoroService::new(config);

        assert!(service.start().is_ok());
        assert!(service.get_session().unwrap().is_running);

        assert!(service.pause().is_ok());
        assert!(!service.get_session().unwrap().is_running);
    }

    #[tokio::test]
    async fn test_pomodoro_service_tick() {
        let config = PomodoroConfig::default();
        let mut service = PomodoroService::new(config);

        service.start().unwrap();
        let initial_remaining = service.get_session().unwrap().remaining;

        // 等待超过1秒
        sleep(Duration::from_millis(1100)).await;

        let event = service.tick().await;
        assert!(event.is_some());

        if let Some(PomodoroEvent::Tick { remaining, .. }) = event {
            assert!(remaining < initial_remaining);
        }
    }

    #[test]
    fn test_pomodoro_config_builder() {
        let config = PomodoroConfig::new()
            .with_work_duration(30)
            .with_short_break(10)
            .with_long_break(20)
            .with_cycles_until_long_break(3);

        assert_eq!(config.work_duration, 1800);
        assert_eq!(config.short_break_duration, 600);
        assert_eq!(config.long_break_duration, 1200);
        assert_eq!(config.cycles_until_long_break, 3);
    }
}
