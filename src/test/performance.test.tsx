import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { PomodoroTimer } from "../components/PomodoroTimer";
import { TodoList } from "../components/TodoList";
import { useAppStore } from "../stores/appStore";

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

describe("Performance Tests", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders PomodoroTimer within a reasonable time", () => {
    vi.mocked(useAppStore).mockReturnValue({
      pomodoroSession: {
        phase: "work",
        duration: 1500,
        remaining: 1500,
        is_running: false,
        cycle_count: 0,
      },
      startPomodoro: vi.fn(),
      pausePomodoro: vi.fn(),
      skipPomodoroPhase: vi.fn(),
      getSelectedTodo: vi.fn(() => null),
      userConfig: null,
    } as any);

    const start = performance.now();
    render(<PomodoroTimer />);
    const elapsed = performance.now() - start;

    expect(elapsed).toBeLessThan(1000);
    expect(screen.getByText("Pomodoro Timer")).toBeInTheDocument();
  });

  it("renders large todo list and summary", () => {
    const mockTodos = Array.from({ length: 1000 }, (_, i) => ({
      id: `todo-${i}`,
      title: `Todo Item ${i}`,
      description: `Description ${i}`,
      status: i % 3 === 0 ? "done" : i % 2 === 0 ? "in_progress" : "todo",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    }));

    vi.mocked(useAppStore).mockReturnValue({
      todos: mockTodos,
      createTodo: vi.fn(async () => null),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
      loadTags: vi.fn(),
      assignTagToTodo: vi.fn(),
      tags: [],
      selectedTodoId: null,
      selectTodo: vi.fn(),
    } as any);

    const start = performance.now();
    render(<TodoList />);
    const elapsed = performance.now() - start;

    expect(elapsed).toBeLessThan(2000);
    expect(screen.getByText("Todo List")).toBeInTheDocument();
    expect(screen.getByText("1000 tasks")).toBeInTheDocument();
  });
});
