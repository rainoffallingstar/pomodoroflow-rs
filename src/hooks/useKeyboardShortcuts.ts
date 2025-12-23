import { useEffect, useCallback } from "react";
import { useAppStore } from "../stores/appStore";

interface Shortcut {
  key: string;
  ctrlKey?: boolean;
  shiftKey?: boolean;
  altKey?: boolean;
  description: string;
  action: () => void;
}

export function useKeyboardShortcuts() {
  const {
    pomodoroSession,
    startPomodoro,
    pausePomodoro,
    resetPomodoro,
    skipPomodoroPhase,
    todos,
    createTodo,
    toggleTodoStatus,
  } = useAppStore();

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      // 忽略输入框中的按键
      if (
        event.target instanceof HTMLInputElement ||
        event.target instanceof HTMLTextAreaElement ||
        event.target instanceof HTMLSelectElement
      ) {
        return;
      }

      const { key, ctrlKey, shiftKey, altKey } = event;

      // 番茄钟控制快捷键
      if (key === " " && !ctrlKey && !shiftKey && !altKey) {
        // 空格键：开始/暂停番茄钟
        event.preventDefault();
        if (pomodoroSession?.is_running) {
          pausePomodoro();
        } else {
          startPomodoro();
        }
      } else if (key === "r" && ctrlKey && !shiftKey && !altKey) {
        // Ctrl+R：重置番茄钟
        event.preventDefault();
        resetPomodoro();
      } else if (key === "s" && ctrlKey && !shiftKey && !altKey) {
        // Ctrl+S：跳过当前阶段
        event.preventDefault();
        skipPomodoroPhase();
      } else if (key === "n" && ctrlKey && !shiftKey && !altKey) {
        // Ctrl+N：新建任务
        event.preventDefault();
        const title = prompt("输入新任务标题：");
        if (title) {
          createTodo(title);
        }
      } else if (key === "?" && !ctrlKey && !shiftKey && !altKey) {
        // ?：显示快捷键帮助
        event.preventDefault();
        alert(getShortcutsHelp());
      } else if (key === "Escape" && !ctrlKey && !shiftKey && !altKey) {
        // ESC：取消编辑/关闭模态框
        event.preventDefault();
        // 这里可以添加关闭模态框的逻辑
      }

      // 数字键：快速切换任务状态
      if (!ctrlKey && !shiftKey && !altKey && key >= "1" && key <= "9") {
        const index = parseInt(key) - 1;
        if (index < todos.length) {
          event.preventDefault();
          toggleTodoStatus(todos[index].id);
        }
      }
    },
    [
      pomodoroSession,
      startPomodoro,
      pausePomodoro,
      resetPomodoro,
      skipPomodoroPhase,
      todos,
      createTodo,
      toggleTodoStatus,
    ],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleKeyDown]);

  const getShortcutsHelp = (): string => {
    const shortcuts: Shortcut[] = [
      { key: "Space", description: "开始/暂停番茄钟", action: () => {} },
      { key: "Ctrl+R", description: "重置番茄钟", action: () => {} },
      { key: "Ctrl+S", description: "跳过当前阶段", action: () => {} },
      { key: "Ctrl+N", description: "新建任务", action: () => {} },
      { key: "1-9", description: "切换第1-9个任务的状态", action: () => {} },
      { key: "?", description: "显示此帮助", action: () => {} },
      { key: "ESC", description: "取消编辑/关闭模态框", action: () => {} },
    ];

    return shortcuts.map((s) => `${s.key}: ${s.description}`).join("\n");
  };

  return { getShortcutsHelp };
}
