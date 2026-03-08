import { useEffect, useCallback } from "react";
import { useAppStore } from "../stores/appStore";

interface Shortcut {
  key: string;
  description: string;
}

export function useKeyboardShortcuts(onOpenSettings?: () => void) {
  const {
    pomodoroSession,
    startPomodoro,
    pausePomodoro,
    resetPomodoro,
  } = useAppStore();

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (
        event.target instanceof HTMLInputElement ||
        event.target instanceof HTMLTextAreaElement ||
        event.target instanceof HTMLSelectElement
      ) {
        return;
      }

      const key = event.key.toLowerCase();
      const isModifier = event.ctrlKey || event.metaKey;

      if (isModifier && key === " ") {
        event.preventDefault();
        if (pomodoroSession?.is_running) {
          void pausePomodoro();
        } else {
          void startPomodoro();
        }
        return;
      }

      if (isModifier && key === "r") {
        event.preventDefault();
        void resetPomodoro();
        return;
      }

      if (isModifier && key === "n") {
        event.preventDefault();
        const inputById = document.getElementById(
          "new-task-input",
        ) as HTMLInputElement | null;
        const input = inputById ?? (document.querySelector(
          'input[placeholder*="任务"], input[placeholder*="task"]',
        ) as HTMLInputElement | null);
        if (input) {
          input.focus();
        }
        return;
      }

      if (isModifier && key === ",") {
        event.preventDefault();
        onOpenSettings?.();
        return;
      }

      if (key === "?" && !isModifier && !event.altKey && !event.shiftKey) {
        event.preventDefault();
        alert(getShortcutsHelp());
      }
    },
    [onOpenSettings, pausePomodoro, pomodoroSession?.is_running, resetPomodoro, startPomodoro],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleKeyDown]);

  const getShortcutsHelp = (): string => {
    const shortcuts: Shortcut[] = [
      { key: "Ctrl/Cmd + Space", description: "开始/暂停番茄钟" },
      { key: "Ctrl/Cmd + N", description: "新建任务" },
      { key: "Ctrl/Cmd + R", description: "重置番茄钟" },
      { key: "Ctrl/Cmd + ,", description: "打开设置" },
      { key: "?", description: "显示此帮助" },
    ];

    return shortcuts.map((s) => `${s.key}: ${s.description}`).join("\n");
  };

  return { getShortcutsHelp };
}
