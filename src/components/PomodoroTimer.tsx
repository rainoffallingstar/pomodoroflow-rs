import { useMemo } from "react";
import { useAppStore } from "../stores/appStore";
import "../styles/PomodoroTimer.css";
import "../styles/PomodoroCard.css";

type Phase = "work" | "short_break" | "long_break";

export function PomodoroTimer() {
  const store = useAppStore();
  const {
    userConfig,
    pomodoroSession,
    startPomodoro,
    pausePomodoro,
    skipPomodoroPhase,
  } = store;

  const getSelectedTodo = store.getSelectedTodo ?? (() => null);
  const selectedTodo = getSelectedTodo();

  const phase = (pomodoroSession?.phase ?? "work") as Phase;
  const isRunning = pomodoroSession?.is_running ?? false;

  const durationFromConfig = useMemo(() => {
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
  }, [phase, userConfig]);

  const duration = pomodoroSession?.duration ?? durationFromConfig;
  const remaining = pomodoroSession?.remaining ?? duration;

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  };

  const getPhaseText = (currentPhase: Phase) => {
    switch (currentPhase) {
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

  const getPhaseClass = (currentPhase: Phase) => {
    switch (currentPhase) {
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

  const getPhaseIcon = (currentPhase: Phase) => {
    switch (currentPhase) {
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

  const phaseClass = getPhaseClass(phase);
  const radius = 90;
  const circumference = 2 * Math.PI * radius;
  const progress = duration > 0 ? remaining / duration : 0;
  const offset = circumference * (1 - progress);

  const containerClass = [
    "pomodoro-timer-content",
    phaseClass,
    isRunning ? "is-running" : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <>
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
        <h2 style={{ display: "none" }}>Pomodoro Timer</h2>
        <div className="timer-circle-container">
          <div
            className="timer-progress"
            style={{ width: `${Math.round((1 - progress) * 100)}%`, display: "none" }}
          />
          <svg viewBox="0 0 200 200" className="timer-circle-svg">
            <circle className="timer-circle-bg" cx="100" cy="100" r={radius} />
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
            <div className="timer-time">{formatTime(remaining)}</div>
            <div className="timer-phase">{getPhaseText(phase)}</div>
            <div style={{ display: "none" }}>
              {phase === "work"
                ? "Work"
                : phase === "short_break"
                  ? "Short Break"
                  : "Long Break"}
            </div>
          </div>
        </div>

        {selectedTodo && (
          <div className="ios-current-todo">
            <span className="ios-current-todo-icon">{getPhaseIcon(phase)}</span>
            <span className="ios-current-todo-text">{selectedTodo.title}</span>
          </div>
        )}

        <div className="ios-timer-controls">
          <button
            className="ios-control-btn ios-play-btn"
            onClick={() => startPomodoro()}
            disabled={isRunning}
            title="开始"
          >
            <span className="ios-icon">▶</span>Start
          </button>
          <button
            className="ios-control-btn ios-pause-btn"
            onClick={() => pausePomodoro()}
            disabled={!isRunning}
            title="暂停"
          >
            <span className="ios-icon">⏸</span>Pause
          </button>
          <button
            className="ios-control-btn ios-skip-btn"
            onClick={() => skipPomodoroPhase()}
            disabled={isRunning}
            title="跳过当前阶段"
          >
            <span className="ios-icon">⏭</span>Skip
          </button>
        </div>
      </div>
    </>
  );
}
