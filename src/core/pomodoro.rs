//! 番茄钟核心逻辑

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::core::error::{AppError, Result};

/// 番茄钟阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PomodoroPhase {
    Work,
    ShortBreak,
    LongBreak,
}

impl PomodoroPhase {
    /// Get phase display name
    pub fn to_string(&self) -> &'static str {
        match self {
            PomodoroPhase::Work => "Work",
            PomodoroPhase::ShortBreak => "Short Break",
            PomodoroPhase::LongBreak => "Long Break",
        }
    }

    /// Get phase duration in seconds
    pub fn default_duration(&self) -> u64 {
        match self {
            PomodoroPhase::Work => 1500,      // 25 minutes
            PomodoroPhase::ShortBreak => 300, // 5 minutes
            PomodoroPhase::LongBreak => 900,  // 15 minutes
        }
    }

    /// Get phase color for UI
    pub fn color(&self) -> &'static str {
        match self {
            PomodoroPhase::Work => "#e74c3c",       // red
            PomodoroPhase::ShortBreak => "#3498db", // blue
            PomodoroPhase::LongBreak => "#9b59b6",  // purple
        }
    }

    /// Get next phase
    pub fn next(&self, cycle_count: u32, cycles_until_long_break: u32) -> PomodoroPhase {
        match self {
            PomodoroPhase::Work => {
                // After work ends, check if long break is needed
                if (cycle_count + 1) % cycles_until_long_break == 0 {
                    PomodoroPhase::LongBreak
                } else {
                    PomodoroPhase::ShortBreak
                }
            }
            PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak => {
                // After break ends, go to work
                PomodoroPhase::Work
            }
        }
    }
}

/// Pomodoro configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroConfig {
    pub work_duration: u64,           // work duration in seconds
    pub short_break_duration: u64,    // short break duration in seconds
    pub long_break_duration: u64,     // long break duration in seconds
    pub cycles_until_long_break: u32, // work cycles until long break
}

impl Default for PomodoroConfig {
    fn default() -> Self {
        Self {
            work_duration: 1500,        // 25 minutes
            short_break_duration: 300,  // 5 minutes
            long_break_duration: 900,   // 15 minutes
            cycles_until_long_break: 4, // work cycles until long break
        }
    }
}

impl PomodoroConfig {
    /// Create default config
    pub fn new() -> Self {
        Self::default()
    }

    /// Create custom config
    pub fn with_work_duration(mut self, minutes: u64) -> Self {
        self.work_duration = minutes * 60;
        self
    }

    pub fn with_short_break(mut self, minutes: u64) -> Self {
        self.short_break_duration = minutes * 60;
        self
    }

    pub fn with_long_break(mut self, minutes: u64) -> Self {
        self.long_break_duration = minutes * 60;
        self
    }

    pub fn with_cycles_until_long_break(mut self, cycles: u32) -> Self {
        self.cycles_until_long_break = cycles;
        self
    }

    /// Get duration for a specific phase
    pub fn get_duration(&self, phase: PomodoroPhase) -> u64 {
        match phase {
            PomodoroPhase::Work => self.work_duration,
            PomodoroPhase::ShortBreak => self.short_break_duration,
            PomodoroPhase::LongBreak => self.long_break_duration,
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.work_duration < 60 {
            return Err(AppError::Validation(
                "Work duration cannot be less than 1 minute".to_string(),
            ));
        }

        if self.work_duration > 7200 {
            return Err(AppError::Validation(
                "Work duration cannot exceed 2 hours".to_string(),
            ));
        }

        if self.short_break_duration < 60 {
            return Err(AppError::Validation(
                "Short break duration cannot be less than 1 minute".to_string(),
            ));
        }

        if self.short_break_duration > 1800 {
            return Err(AppError::Validation(
                "Short break duration cannot exceed 30 minutes".to_string(),
            ));
        }

        if self.long_break_duration < 300 {
            return Err(AppError::Validation(
                "Long break duration cannot be less than 5 minutes".to_string(),
            ));
        }

        if self.long_break_duration > 3600 {
            return Err(AppError::Validation(
                "Long break duration cannot exceed 1 hour".to_string(),
            ));
        }

        if self.cycles_until_long_break < 2 {
            return Err(AppError::Validation(
                "Long break interval cannot be less than 2 cycles".to_string(),
            ));
        }

        if self.cycles_until_long_break > 10 {
            return Err(AppError::Validation(
                "Long break interval cannot exceed 10 cycles".to_string(),
            ));
        }

        Ok(())
    }
}

/// 番茄钟会话
#[derive(Debug, Clone, Serialize)]
pub struct PomodoroSession {
    pub phase: PomodoroPhase,
    pub duration: u64,    // 设定时长（秒）
    pub remaining: u64,   // 剩余时间（秒）
    pub is_running: bool, // 是否运行中
    pub cycle_count: u32, // 已完成的工作周期数
    #[serde(skip)]
    pub started_at: Option<Instant>, // 开始时间点（不序列化）
    pub config: PomodoroConfig, // 当前配置
}

// 安全实现 Send + Sync，因为 Instant 在单线程环境中是安全的
unsafe impl Send for PomodoroSession {}
unsafe impl Sync for PomodoroSession {}

impl<'de> serde::Deserialize<'de> for PomodoroSession {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PomodoroSessionData {
            phase: PomodoroPhase,
            duration: u64,
            remaining: u64,
            is_running: bool,
            cycle_count: u32,
            config: PomodoroConfig,
        }

        let data = PomodoroSessionData::deserialize(deserializer)?;

        Ok(PomodoroSession {
            phase: data.phase,
            duration: data.duration,
            remaining: data.remaining,
            is_running: data.is_running,
            cycle_count: data.cycle_count,
            started_at: None, // 重置为 None，因为 Instant 无法反序列化
            config: data.config,
        })
    }
}

impl PomodoroSession {
    /// 创建新的番茄钟会话（工作阶段）
    pub fn new(config: PomodoroConfig) -> Self {
        let work_duration = config.work_duration;
        Self {
            phase: PomodoroPhase::Work,
            duration: work_duration,
            remaining: work_duration,
            is_running: false,
            cycle_count: 0,
            started_at: None,
            config,
        }
    }

    /// 开始计时
    pub fn start(&mut self) -> Result<()> {
        if self.is_running {
            return Err(AppError::InvalidState("计时器已在运行中".to_string()));
        }

        self.is_running = true;
        self.started_at = Some(Instant::now());
        Ok(())
    }

    /// 暂停计时
    pub fn pause(&mut self) -> Result<()> {
        if !self.is_running {
            return Err(AppError::InvalidState("计时器未运行".to_string()));
        }

        self.is_running = false;
        self.started_at = None;
        Ok(())
    }

    /// 重置当前阶段
    pub fn reset(&mut self) -> Result<()> {
        self.remaining = self.duration;
        self.is_running = false;
        self.started_at = None;
        Ok(())
    }

    /// 跳过当前阶段
    pub fn skip(&mut self) -> Result<PomodoroPhase> {
        if self.is_running {
            return Err(AppError::InvalidState("请先暂停计时器".to_string()));
        }

        let next_phase = self
            .phase
            .next(self.cycle_count, self.config.cycles_until_long_break);
        self.switch_to_phase(next_phase)?;
        Ok(next_phase)
    }

    /// 切换到指定阶段
    pub fn switch_to_phase(&mut self, phase: PomodoroPhase) -> Result<()> {
        self.phase = phase;
        self.duration = self.config.get_duration(phase);
        self.remaining = self.duration;
        self.is_running = false;
        self.started_at = None;
        Ok(())
    }

    /// 更新倒计时
    pub fn tick(&mut self) -> Result<bool> {
        if !self.is_running {
            return Ok(false);
        }

        if self.remaining > 0 {
            self.remaining -= 1;
            Ok(false)
        } else {
            // 计时结束
            self.complete_phase()?;
            Ok(true)
        }
    }

    /// 完成当前阶段
    fn complete_phase(&mut self) -> Result<()> {
        self.is_running = false;
        self.started_at = None;

        // 更新工作周期计数
        if self.phase == PomodoroPhase::Work {
            self.cycle_count += 1;
        }

        // 切换到下一阶段
        let next_phase = self
            .phase
            .next(self.cycle_count, self.config.cycles_until_long_break);
        self.switch_to_phase(next_phase)?;

        Ok(())
    }

    /// 获取剩余时间（格式：MM:SS）
    pub fn formatted_time(&self) -> String {
        let minutes = self.remaining / 60;
        let seconds = self.remaining % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// 获取进度百分比
    pub fn progress(&self) -> f32 {
        if self.duration == 0 {
            0.0
        } else {
            ((self.duration - self.remaining) as f32 / self.duration as f32) * 100.0
        }
    }

    /// 检查是否完成
    pub fn is_completed(&self) -> bool {
        self.remaining == 0
    }

    /// 获取开始时间
    pub fn started_at_datetime(&self) -> Option<DateTime<Utc>> {
        self.started_at.map(|_| Utc::now())
    }

    /// 更新配置
    pub fn update_config(&mut self, new_config: PomodoroConfig) -> Result<()> {
        new_config.validate()?;
        self.config = new_config;
        // 如果计时器未运行，更新当前阶段时长
        if !self.is_running {
            self.duration = self.config.get_duration(self.phase);
            self.remaining = self.duration;
        }
        Ok(())
    }
}

/// 番茄钟服务
#[derive(Debug, Clone)]
pub struct PomodoroService {
    session: Option<PomodoroSession>,
    config: PomodoroConfig,
}

// 安全实现 Send + Sync，因为所有字段都是基本类型
unsafe impl Send for PomodoroService {}
unsafe impl Sync for PomodoroService {}

/// 番茄钟事件
#[derive(Debug, Clone)]
pub enum PomodoroEvent {
    Tick {
        remaining: u64,
        formatted: String,
        progress: f32,
    },
    PhaseCompleted {
        completed_phase: PomodoroPhase,
        next_phase: PomodoroPhase,
        cycle_count: u32,
    },
    StateChanged {
        is_running: bool,
        phase: PomodoroPhase,
    },
}

impl PomodoroService {
    /// 创建新的番茄钟服务
    pub fn new(config: PomodoroConfig) -> Self {
        Self {
            session: Some(PomodoroSession::new(config.clone())),
            config,
        }
    }

    /// 获取当前会话
    pub fn get_session(&self) -> Option<&PomodoroSession> {
        self.session.as_ref()
    }

    /// 获取可变的当前会话
    pub fn get_session_mut(&mut self) -> Option<&mut PomodoroSession> {
        self.session.as_mut()
    }

    /// 开始计时
    pub fn start(&mut self) -> Result<()> {
        if let Some(session) = self.session.as_mut() {
            session.start()?;
        }
        Ok(())
    }

    /// 暂停计时
    pub fn pause(&mut self) -> Result<()> {
        if let Some(session) = self.session.as_mut() {
            session.pause()?;
        }
        Ok(())
    }

    /// 重置计时
    pub fn reset(&mut self) -> Result<()> {
        if let Some(session) = self.session.as_mut() {
            session.reset()?;
        }
        Ok(())
    }

    /// 跳过当前阶段
    pub fn skip(&mut self) -> Result<()> {
        if let Some(session) = self.session.as_mut() {
            session.skip()?;
        }
        Ok(())
    }

    /// 更新配置
    pub fn update_config(&mut self, config: PomodoroConfig) -> Result<()> {
        if let Some(session) = self.session.as_mut() {
            session.update_config(config)?;
        }
        Ok(())
    }

    /// 处理计时器 tick
    pub async fn tick(&mut self) -> Option<PomodoroEvent> {
        if let Some(session) = self.session.as_mut() {
            if let Ok(completed) = session.tick() {
                if completed {
                    let completed_phase = session.phase;
                    let next_phase = completed_phase
                        .next(session.cycle_count, session.config.cycles_until_long_break);

                    return Some(PomodoroEvent::PhaseCompleted {
                        completed_phase,
                        next_phase,
                        cycle_count: session.cycle_count,
                    });
                } else {
                    return Some(PomodoroEvent::Tick {
                        remaining: session.remaining,
                        formatted: session.formatted_time(),
                        progress: session.progress(),
                    });
                }
            }
        }

        None
    }
}

/// 番茄钟统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroStats {
    pub total_sessions: u32,
    pub work_sessions: u32,
    pub short_break_sessions: u32,
    pub long_break_sessions: u32,
    pub total_focus_time: u64, // 总专注时间（分钟）
    pub average_daily_sessions: f32,
}

impl PomodoroStats {
    /// 创建空统计
    pub fn empty() -> Self {
        Self {
            total_sessions: 0,
            work_sessions: 0,
            short_break_sessions: 0,
            long_break_sessions: 0,
            total_focus_time: 0,
            average_daily_sessions: 0.0,
        }
    }
}
