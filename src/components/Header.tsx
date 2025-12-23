import { useState } from "react";
import { SettingsPanel } from "./SettingsPanel";
import "../styles/App.css";

interface HeaderProps {
  onOpenSettings?: () => void;
}

export function Header({ onOpenSettings }: HeaderProps) {
  const [showSettings, setShowSettings] = useState(false);
  const [currentTime, setCurrentTime] = useState(new Date());

  // 更新时间显示
  useState(() => {
    const timer = setInterval(() => {
      setCurrentTime(new Date());
    }, 60000); // 每分钟更新一次

    return () => clearInterval(timer);
  });

  const handleSettingsClick = () => {
    if (onOpenSettings) {
      onOpenSettings();
    } else {
      setShowSettings(true);
    }
  };

  const handleCloseSettings = () => {
    setShowSettings(false);
  };

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  };

  const formatDate = (date: Date) => {
    return date.toLocaleDateString([], {
      weekday: "short",
      month: "short",
      day: "numeric",
    });
  };

  return (
    <>
      <header className="app-header">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-3">
            <div className="text-2xl">🍅</div>
            <div>
              <h1 className="text-xl font-bold">PomodoroFlow-Rs</h1>
              <p className="text-sm text-tertiary">专注工作，高效生活</p>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-6">
          {/* 时间显示 */}
          <div className="hidden md:flex flex-col items-end">
            <div className="text-lg font-semibold">
              {formatTime(currentTime)}
            </div>
            <div className="text-sm text-tertiary">
              {formatDate(currentTime)}
            </div>
          </div>

          {/* 状态指示器 */}
          <div className="hidden sm:flex items-center gap-2">
            <div className="flex items-center gap-1">
              <div className="w-2 h-2 rounded-full bg-success-color"></div>
              <span className="text-sm text-tertiary">在线</span>
            </div>
          </div>

          {/* 控制按钮 */}
          <div className="header-controls">
            <button
              className="btn-icon"
              onClick={handleSettingsClick}
              title="打开设置"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
                <circle cx="12" cy="12" r="3" />
              </svg>
            </button>
          </div>
        </div>
      </header>

      {/* 设置面板模态框 */}
      {showSettings && !onOpenSettings && (
        <div className="modal-overlay" onClick={handleCloseSettings}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <SettingsPanel onClose={handleCloseSettings} />
          </div>
        </div>
      )}
    </>
  );
}
