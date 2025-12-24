import { useState, useEffect } from "react";
import { useAppStore, UserConfig } from "../stores/appStore";
import "../styles/PomodoroTimer.css";
import "../styles/PomodoroCard.css";

/**
 * iOS 18 动态岛风格的番茄钟组件
 * - 前端实现倒计时，每秒递减
 * - 倒计时到 0 自动切换阶段
 * - 毛玻璃背景 + 流畅动画
 */
export function PomodoroTimer() {
  const {
    userConfig,
    selectedTodoId,
    getSelectedTodo,
    startPomodoro,
    pausePomodoro,
    skipPomodoroPhase,
    loadPomodoroSession,
  } = useAppStore();

  // 本地倒计时状态
  const [localRemaining, setLocalRemaining] = useState<number>(() => {
    return userConfig?.pomodoro_work_duration || 1500;
  });
  
  const [localPhase, setLocalPhase] = useState<string>("work");
  const [isRunning, setIsRunning] = useState<boolean>(false);
  const [cycleCount, setCycleCount] = useState<number>(0);

  // 获取选中的 Todo
  const selectedTodo = getSelectedTodo();

  // 根据阶段获取时长
  const getDurationForPhase = (phase: string): number => {
    if (!userConfig) return 1500;
    switch (phase) {
      case "work":
        return userConfig.pomodoro_work_duration;
      case "short_break":
        return userConfig.pomodoro_short_break_duration;
      case "long_break":
        return userConfig.pomodoro_long_break_duration;
      default:
        return 1500;
    }
  };

  // 获取下一阶段 - 使用循环计数
  const getNextPhase = (currentPhase: string, currentCycleCount: number): string => {
    const cyclesUntilLong = userConfig?.pomodoro_cycles_until_long_break || 4;
    
    if (currentPhase === "work") {
      // 工作完成后，检查是否需要长休息
      const nextCount = currentCycleCount + 1;
      if (nextCount % cyclesUntilLong === 0) {
        return "long_break";
      }
      return "short_break";
    } else if (currentPhase === "short_break" || currentPhase === "long_break") {
      // 休息结束后，回到工作
      return "work";
    } else {
      return "work";
    }
  };

  // 倒计时定时器 - 前端实现
  useEffect(() => {
    if (!isRunning) return;

    const timer = setInterval(() => {
      setLocalRemaining((prev) => {
        if (prev <= 1) {
          // 倒计时结束，触发阶段切换
          handlePhaseComplete();
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, [isRunning, localPhase]);

  // 阶段完成处理
  const handlePhaseComplete = async () => {
    console.log("⏰ Phase complete, switching to next phase");
    setIsRunning(false);
    
    // 如果是工作阶段完成，递增循环计数
    if (localPhase === "work") {
      const newCount = cycleCount + 1;
      setCycleCount(newCount);
      console.log(`Work cycle ${newCount} completed`);
    }
    
    // 调用后端跳过当前阶段
    await skipPomodoroPhase();
    
    // 重新加载会话状态
    await loadPomodoroSession();
    
    // 前端切换到下一阶段并自动开始
    setTimeout(() => {
      const nextPhase = getNextPhase(localPhase, cycleCount);
      console.log("Switching to phase:", nextPhase);
      setLocalPhase(nextPhase);
      setLocalRemaining(getDurationForPhase(nextPhase));
      setIsRunning(true); // 自动开始下一阶段
    }, 100);
  };

  // 启动处理
  const handleStart = async () => {
    console.log("🍅 Starting pomodoro timer (frontend)");
    await startPomodoro();
    setIsRunning(true);
    // 如果 remaining 是 0 或默认值，重置为当前阶段时长
    if (localRemaining <= 0 || localRemaining === 1500) {
      setLocalRemaining(getDurationForPhase(localPhase));
    }
  };

  // 暂停处理
  const handlePause = async () => {
    console.log("⏸️ Pausing pomodoro timer");
    await pausePomodoro();
    setIsRunning(false);
  };

  // 跳过处理
  const handleSkip = async () => {
    console.log("⏭️ Skipping to next phase");
    setIsRunning(false);
    await skipPomodoroPhase();
    
    // 如果是工作阶段被跳过，也递增计数
    if (localPhase === "work") {
      const newCount = cycleCount + 1;
      setCycleCount(newCount);
    }
    
    // 前端切换到下一阶段
    const nextPhase = getNextPhase(localPhase, cycleCount);
    setLocalPhase(nextPhase);
    setLocalRemaining(getDurationForPhase(nextPhase));
  };

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  };

  const getPhaseText = (phase: string) => {
    switch (phase) {
      case "work":
        return "专注工作";
      case "short_break":
        return "短暂休息";
      case "long_break":
        return "长休息";
      default:
        return "准备开始";
    }
  };

  const getPhaseClass = (phase: string) => {
    switch (phase) {
      case "work":
        return "phase-work";
      case "short_break":
        return "phase-short-break";
      case "long_break":
        return "phase-long-break";
      default:
        return "";
    }
  };

  const phaseClass = getPhaseClass(localPhase);

  // 圆环进度条计算
  const radius = 90;
  const circumference = 2 * Math.PI * radius;
  const duration = getDurationForPhase(localPhase);
  const progress = localRemaining / duration;
  const offset = circumference * (1 - progress);

  // 获取阶段图标
  const getPhaseIcon = (phase: string) => {
    switch (phase) {
      case "work":
        return "🎯";
      case "short_break":
        return "🍅";
      case "long_break":
        return "☕️";
      default:
        return "⏱️";
    }
  };

  const containerClass = [
    "pomodoro-timer-content",
    phaseClass,
    isRunning ? "is-running" : ""
  ].filter(Boolean).join(" ");

  return (
    <>
      {/* SVG 渐变定义 */}
      <svg className="svg-gradients">
        <defs>
          <linearGradient id="gradient-work" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#ff9500" />
            <stop offset="100%" stopColor="#ff6b00" />
          </linearGradient>
          <linearGradient id="gradient-short-break" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#34c759" />
            <stop offset="100%" stopColor="#30d158" />
          </linearGradient>
          <linearGradient id="gradient-long-break" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#007aff" />
            <stop offset="100%" stopColor="#5ac8fa" />
          </linearGradient>
        </defs>
      </svg>

      <div className={containerClass}>
        {/* 圆环进度条 */}
        <div className="timer-circle-container">
          <svg viewBox="0 0 200 200" className="timer-circle-svg">
            <circle
              className="timer-circle-bg"
              cx="100"
              cy="100"
              r={radius}
            />
            <circle
              className="timer-circle-progress"
              cx="100"
              cy="100"
              r={radius}
              strokeDasharray={circumference}
              strokeDashoffset={offset}
            />
          </svg>
          <div className="timer-content">
            <div className="timer-time">{formatTime(localRemaining)}</div>
            <div className="timer-phase">{getPhaseText(localPhase)}</div>
          </div>
        </div>

        {/* 当前选中的任务 */}
        {selectedTodo && (
          <div className="ios-current-todo">
            <span className="ios-current-todo-icon">{getPhaseIcon(localPhase)}</span>
            <span className="ios-current-todo-text">{selectedTodo.title}</span>
          </div>
        )}

        {/* iOS 风格控制按钮 */}
        <div className="ios-timer-controls">
          <button
            className="ios-control-btn ios-play-btn"
            onClick={handleStart}
            disabled={isRunning}
            title="开始"
          >
            <span className="ios-icon">▶</span>
          </button>
          <button
            className="ios-control-btn ios-pause-btn"
            onClick={handlePause}
            disabled={!isRunning}
            title="暂停"
          >
            <span className="ios-icon">⏸</span>
          </button>
          <button
            className="ios-control-btn ios-skip-btn"
            onClick={handleSkip}
            disabled={isRunning}
            title="跳过当前阶段"
          >
            <span className="ios-icon">⏭</span>
          </button>
        </div>
      </div>
    </>
  );
}
