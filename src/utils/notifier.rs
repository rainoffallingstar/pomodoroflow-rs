//! 系统通知工具

use crate::core::error::{AppError, Result};

/// 通知管理器
#[derive(Debug, Clone)]
pub struct Notifier {
    enabled: bool,
}

impl Notifier {
    /// 创建新的通知管理器
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// 发送通知
    pub fn notify(&self, title: &str, message: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            use winrt_notification::{Duration, Sound, Toast};
            let toast = Toast::new(Toast::POWERSHELL_APP_ID)
                .title(title)
                .text1(message)
                .sound(Some(Sound::SMS))
                .duration(Duration::Short);

            toast.show()
                .map_err(|e| AppError::Other(format!("Windows 通知错误: {}", e)))?;
        }

        #[cfg(target_os = "macos")]
        {
            // macOS 通知
            // 这里简化处理
            println!("[通知] {}: {}", title, message);
        }

        #[cfg(target_os = "linux")]
        {
            // Linux 通知
            // 使用 libnotify
            println!("[通知] {}: {}", title, message);
        }

        Ok(())
    }

    /// 发送番茄钟完成通知
    pub fn notify_pomodoro_complete(&self, phase: &str) -> Result<()> {
        let title = "Pomodoro";
        let message = format!("{} time ended!", phase);
        self.notify(title, &message)
    }

    /// 发送任务同步通知
    pub fn notify_sync_complete(&self, success_count: usize, failed_count: usize) -> Result<()> {
        let title = "GitHub Sync";
        if failed_count == 0 {
            let message = format!("Sync completed, {} tasks", success_count);
            self.notify(title, &message)
        } else {
            let message = format!("Sync completed: {} successful, {} failed", success_count, failed_count);
            self.notify(title, &message)
        }
    }

    /// 发送错误通知
    pub fn notify_error(&self, error: &str) -> Result<()> {
        let title = "Error";
        self.notify(title, error)
    }

    /// 启用/禁用通知
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for Notifier {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notifier_creation() {
        let notifier = Notifier::new(true);
        assert!(notifier.is_enabled());
    }

    #[test]
    fn test_notifier_disabled() {
        let notifier = Notifier::new(false);
        assert!(!notifier.is_enabled());
        assert!(notifier.notify("Test", "Message").is_ok());
    }
}
