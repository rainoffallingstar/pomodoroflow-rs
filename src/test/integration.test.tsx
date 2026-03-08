import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { act } from "react";
import App from "../App";
import { useAppStore } from "../stores/appStore";

vi.mock("@tauri-apps/api/tauri", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

function makeStore(overrides: Record<string, unknown> = {}) {
  return {
    todos: [],
    pomodoroSession: {
      phase: "work",
      duration: 1500,
      remaining: 1500,
      is_running: false,
      cycle_count: 0,
    },
    userConfig: null,
    theme: "light",
    isLoading: false,
    initializeApp: vi.fn(async () => {}),
    setupEventListeners: vi.fn(async () => {}),
    cleanupEventListeners: vi.fn(),
    clearMessages: vi.fn(),
    setError: vi.fn(),
    setSuccess: vi.fn(),
    createTodo: vi.fn(async () => ({ id: "new-id" })),
    updateTodo: vi.fn(),
    deleteTodo: vi.fn(),
    toggleTodoStatus: vi.fn(),
    startPomodoro: vi.fn(),
    pausePomodoro: vi.fn(),
    resetPomodoro: vi.fn(),
    skipPomodoroPhase: vi.fn(),
    loadTodos: vi.fn(),
    loadUserConfig: vi.fn(),
    saveUserConfig: vi.fn(),
    loadPomodoroSession: vi.fn(),
    setTheme: vi.fn(),
    toggleTheme: vi.fn(),
    selectedTodoId: null,
    selectTodo: vi.fn(),
    getSelectedTodo: vi.fn(() => null),
    tags: [],
    loadTags: vi.fn(),
    createTag: vi.fn(),
    deleteTag: vi.fn(),
    assignTagToTodo: vi.fn(),
    removeTagFromTodo: vi.fn(),
    getTodoTags: vi.fn(async () => []),
    ...overrides,
  };
}

describe("Integration Tests", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("renders app and exits loading state", async () => {
    vi.mocked(useAppStore).mockReturnValue(
      makeStore({
        todos: [
          {
            id: "1",
            title: "Complete project documentation",
            description: null,
            status: "todo",
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          },
        ],
      }) as any,
    );

    render(<App />);

    expect(
      await screen.findByText("Complete project documentation"),
    ).toBeInTheDocument();
    expect(screen.getByText("25:00")).toBeInTheDocument();
  });

  it("creates todo via input flow", async () => {
    const createTodo = vi.fn(async () => ({ id: "created-id" }));

    vi.mocked(useAppStore).mockReturnValue(
      makeStore({
        todos: [],
        createTodo,
      }) as any,
    );

    render(<App />);

    const input = await screen.findByPlaceholderText("Add a new task...");
    fireEvent.change(input, { target: { value: "Write unit tests" } });
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: "Add" }));
      await Promise.resolve();
    });

    await waitFor(() => {
      expect(createTodo).toHaveBeenCalledWith(
        "Write unit tests",
        undefined,
        "todo",
      );
    });
  });
});
