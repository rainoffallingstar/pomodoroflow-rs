import { ReactNode } from "react";
import { PomodoroTimer } from "./PomodoroTimer";
import { TodoList } from "./TodoList";
import "../styles/App.css";

interface MainLayoutProps {
  children?: ReactNode;
}

/**
 * iOS 18 风格主布局
 * - 动态岛风格的番茄钟
 * - 分组列表风格的 TodoList
 * - 移除统计面板，保持简洁
 */
export function MainLayout({ children }: MainLayoutProps) {
  return (
    <div className="ios-main-layout">
      {/* iOS 动态岛风格番茄钟 */}
      <div className="ios-dynamic-island">
        <PomodoroTimer />
      </div>

      {/* iOS 分组列表风格 TodoList */}
      <div className="ios-todo-section">
        <TodoList />
      </div>

      {/* 自定义子内容 */}
      {children && <div className="mt-6">{children}</div>}
    </div>
  );
}
