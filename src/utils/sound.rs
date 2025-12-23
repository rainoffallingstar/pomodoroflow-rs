//! 音频播放工具

use crate::core::error::Result;

/// 声音播放器
#[derive(Debug, Clone)]
pub struct SoundPlayer {
    enabled: bool,
}

impl SoundPlayer {
    /// 创建新的声音播放器
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// 播放提示音
    pub fn play_beep(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            // Windows 系统提示音
            unsafe {
                winapi::um::winuser::MessageBeep(0xFFFFFFFF);
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS 系统提示音
            print!("\x07"); // Bell character
        }

        #[cfg(target_os = "linux")]
        {
            // Linux 终端提示音
            print!("\x07");
        }

        Ok(())
    }

    /// 播放成功音效
    pub fn play_success(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // 播放较短的提示音
        self.play_beep()
    }

    /// 播放错误音效
    pub fn play_error(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // 播放两声提示音
        self.play_beep()?;
        std::thread::sleep(std::time::Duration::from_millis(200));
        self.play_beep()
    }

    /// 播放番茄钟完成音效
    pub fn play_pomodoro_complete(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // 播放三声提示音
        for i in 0..3 {
            self.play_beep()?;
            if i < 2 {
                std::thread::sleep(std::time::Duration::from_millis(300));
            }
        }

        Ok(())
    }

    /// 播放通知音效
    pub fn play_notification(&self) -> Result<()> {
        self.play_success()
    }

    /// 启用/禁用声音
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for SoundPlayer {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(target_os = "windows")]
mod winapi {
    pub mod um {
        pub mod winuser {
            extern "system" {
                pub fn MessageBeep(uType: u32) -> i32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_player_creation() {
        let player = SoundPlayer::new(true);
        assert!(player.is_enabled());
    }

    #[test]
    fn test_sound_player_disabled() {
        let player = SoundPlayer::new(false);
        assert!(!player.is_enabled());
        assert!(player.play_beep().is_ok());
    }
}
