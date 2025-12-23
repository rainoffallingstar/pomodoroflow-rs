import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "../App";
import { useAppStore } from "../stores/appStore";

// Mock Tauri API
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/tauri", () => ({
  invoke: mockInvoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
  emit: vi.fn(),
}));

// Mock stores
vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

describe("Integration Tests", () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Setup default store mock
    const mockAppStore = {
      todos: [],
      pomodoroSession: null,
      userConfig: null,
      theme: "light",
      toggleTheme: vi.fn(),
      loadTodos: vi.fn(),
      createTodo: vi.fn(),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
      startPomodoro: vi.fn(),
      pausePomodoro: vi.fn(),
      resetPomodoro: vi.fn(),
      skipPomodoroPhase: vi.fn(),
      loadUserConfig: vi.fn(),
      saveUserConfig: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockAppStore);
  });

  it("should integrate pomodoro timer with todo list", async () => {
    const user = userEvent.setup();

    // Setup initial data
    const mockTodos = [
      {
        id: "1",
        title: "Complete project documentation",
        description: null,
        status: "todo",
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
    ];

    const mockSession = {
      phase: "work",
      duration: 1500,
      remaining: 1500,
      is_running: false,
      cycle_count: 0,
    };

    vi.mocked(useAppStore).mockReturnValue({
      todos: mockTodos,
      pomodoroSession: mockSession,
      userConfig: null,
      theme: "light",
      toggleTheme: vi.fn(),
      loadTodos: vi.fn(),
      createTodo: vi.fn(),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
      startPomodoro: vi.fn(),
      pausePomodoro: vi.fn(),
      resetPomodoro: vi.fn(),
      skipPomodoroPhase: vi.fn(),
      loadUserConfig: vi.fn(),
      saveUserConfig: vi.fn(),
    });

    render(<App />);

    // Verify initial state
    expect(screen.getByText("PomodoroFlow")).toBeInTheDocument();
    expect(
      screen.getByText("Complete project documentation"),
    ).toBeInTheDocument();
    expect(screen.getByText("25:00")).toBeInTheDocument();
  });

  it("should create and complete a workflow", async () => {
    const user = userEvent.setup();

    // Setup empty state
    vi.mocked(useAppStore).mockReturnValue({
      todos: [],
      pomodoroSession: null,
      userConfig: null,
      theme: "light",
      toggleTheme: vi.fn(),
      loadTodos: vi.fn(),
      createTodo: vi.fn(),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
      startPomodoro: vi.fn(),
      pausePomodoro: vi.fn(),
      resetPomodoro: vi.fn(),
      skipPomodoroPhase: vi.fn(),
      loadUserConfig: vi.fn(),
      saveUserConfig: vi.fn(),
    });

    render(<App />);

    // Create new todo
    const input = screen.getByPlaceholderText("Add a new task...");
    await user.type(input, "Write unit tests");

    const addButton = screen.getByText("Add");
    await user.click(addButton);

    expect(screen.getByText("PomodoroFlow")).toBeInTheDocument();
  });
});
