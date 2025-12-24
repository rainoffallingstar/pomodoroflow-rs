import { ReactNode } from "react";
import { PomodoroTimer } from "./PomodoroTimer";
import { TodoList } from "./TodoList";
import "../styles/App.css";

interface MainLayoutProps {
  children?: ReactNode;
}

/**
 * iOS 18 风格主布局
 * - 左右分栏：左侧动态岛番茄钟，右侧分组列表 TodoList
 * - 浮动设置按钮（在 App.tsx 中）
 * - 移除统计面板，保持简洁
 */
export function MainLayout({ children }: MainLayoutProps) {
  return (
    <div className="ios-split-layout">
      {/* 左侧：iOS 毛玻璃风格番茄钟卡片 */}
      <div className="ios-pomodoro-card">
        <PomodoroTimer />
      </div>

      {/* 右侧：iOS 分组列表风格 TodoList */}
      <div className="ios-todo-card">
        <div className="ios-todo-section">
          <TodoList />
        </div>
      </div>

      {/* 自定义子内容 */}
      {children && <div className="mt-6">{children}</div>}
    </div>
  );
}
