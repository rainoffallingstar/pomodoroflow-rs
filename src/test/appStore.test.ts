import { describe, it, expect, vi, beforeEach } from "vitest";
import { act } from "@testing-library/react";
import { useAppStore } from "../stores/appStore";

// Mock Tauri API
const mockInvoke = vi.fn();
vi.mock("@tauri-apps/api/tauri", () => ({
  invoke: mockInvoke,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

describe("AppStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset store state
    const store = useAppStore.getState();
    store.todos = [];
    store.pomodoroSession = null;
    store.userConfig = null;
  });

  it("should have initial state", () => {
    const store = useAppStore.getState();
    expect(store.todos).toEqual([]);
    expect(store.pomodoroSession).toBeNull();
    expect(store.userConfig).toBeNull();
  });

  it("should load todos successfully", async () => {
    const mockTodos = [
      {
        id: "1",
        title: "Test Todo",
        description: "Test Description",
        status: "todo" as const,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      },
    ];

    mockInvoke.mockResolvedValue(mockTodos);

    await act(async () => {
      await useAppStore.getState().loadTodos();
    });

    expect(mockInvoke).toHaveBeenCalledWith("get_todos");
    expect(useAppStore.getState().todos).toEqual(mockTodos);
  });

  it("should create todo successfully", async () => {
    const mockTodo = {
      id: "1",
      title: "New Todo",
      description: null,
      status: "todo" as const,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    mockInvoke.mockResolvedValue(mockTodo);

    await act(async () => {
      await useAppStore.getState().createTodo("New Todo");
    });

    expect(mockInvoke).toHaveBeenCalledWith("create_todo", {
      title: "New Todo",
    });
    expect(useAppStore.getState().todos).toContainEqual(mockTodo);
  });

  it("should start pomodoro successfully", async () => {
    const mockSession = {
      phase: "work" as const,
      duration: 1500,
      remaining: 1500,
      is_running: true,
      cycle_count: 0,
    };

    mockInvoke.mockResolvedValue(mockSession);

    await act(async () => {
      await useAppStore.getState().startPomodoro();
    });

    expect(mockInvoke).toHaveBeenCalledWith("start_pomodoro");
    expect(useAppStore.getState().pomodoroSession).toEqual(mockSession);
  });

  it("should pause pomodoro successfully", async () => {
    const mockSession = {
      phase: "work" as const,
      duration: 1500,
      remaining: 1500,
      is_running: false,
      cycle_count: 0,
    };

    const store = useAppStore.getState();
    store.pomodoroSession = {
      phase: "work" as const,
      duration: 1500,
      remaining: 1500,
      is_running: true,
      cycle_count: 0,
    };

    mockInvoke.mockResolvedValue(mockSession);

    await act(async () => {
      await store.pausePomodoro();
    });

    expect(mockInvoke).toHaveBeenCalledWith("pause_pomodoro");
    expect(store.pomodoroSession).toEqual(mockSession);
  });
});
