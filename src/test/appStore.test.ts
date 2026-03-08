import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { act } from "@testing-library/react";
import { useAppStore } from "../stores/appStore";
import { invoke } from "@tauri-apps/api/tauri";

vi.mock("@tauri-apps/api/tauri", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

describe("AppStore", () => {
  let consoleErrorSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.clearAllMocks();
    consoleErrorSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    useAppStore.setState({
      todos: [],
      pomodoroSession: null,
      userConfig: null,
      tags: [],
      selectedTodoId: null,
      error: null,
      errorCode: null,
      success: null,
    } as any);
  });

  afterEach(() => {
    consoleErrorSpy.mockRestore();
  });

  it("has initial state", () => {
    const store = useAppStore.getState();
    expect(store.todos).toEqual([]);
    expect(store.pomodoroSession).toBeNull();
    expect(store.userConfig).toBeNull();
    expect(store.errorCode).toBeNull();
  });

  it("loads todos", async () => {
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

    vi.mocked(invoke).mockImplementation((command: string) => {
      if (command === "get_todos") {
        return Promise.resolve({
          success: true,
          data: mockTodos,
          error: null,
        } as any);
      }
      if (command === "get_todo_tags") {
        return Promise.resolve({
          success: true,
          data: [],
          error: null,
        } as any);
      }
      return Promise.resolve({ success: true, data: null, error: null } as any);
    });

    await act(async () => {
      await useAppStore.getState().loadTodos();
    });

    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_todos", undefined);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_todo_tags", {
      todo_id: "1",
    });
    expect(useAppStore.getState().todos).toEqual(
      mockTodos.map((todo) => ({ ...todo, tags: [] })),
    );
  });

  it("creates todo", async () => {
    const mockTodo = {
      id: "1",
      title: "New Todo",
      description: null,
      status: "todo" as const,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    vi.mocked(invoke).mockResolvedValue({
      success: true,
      data: mockTodo,
      error: null,
    } as any);

    await act(async () => {
      await useAppStore.getState().createTodo("New Todo");
    });

    expect(vi.mocked(invoke)).toHaveBeenCalledWith("create_todo", {
      title: "New Todo",
      description: undefined,
      status: "todo",
    });
    expect(useAppStore.getState().todos.find((t) => t.id === "1")).toBeTruthy();
  });

  it("uses backend todo status as source of truth", async () => {
    const backendTodo = {
      id: "2",
      title: "Server Todo",
      description: null,
      status: "in_progress" as const,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    vi.mocked(invoke).mockResolvedValue({
      success: true,
      data: backendTodo,
      error: null,
    } as any);

    await act(async () => {
      await useAppStore.getState().createTodo("Server Todo", undefined, "todo");
    });

    const saved = useAppStore.getState().todos.find((t) => t.id === backendTodo.id);
    expect(saved?.status).toBe("in_progress");
  });

  it("rejects wrapped success with empty data for non-nullable command", async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: true,
      data: null,
      error: null,
    } as any);

    await expect(
      useAppStore.getState().createTodo("Invalid Empty Payload"),
    ).rejects.toThrow("create_todo returned empty data");
  });

  it("allows wrapped success with empty data for nullable config command", async () => {
    vi.mocked(invoke).mockImplementation((command: string) => {
      if (command === "get_user_config") {
        return Promise.resolve({
          success: true,
          data: null,
          error: null,
        } as any);
      }
      return Promise.resolve({ success: true, data: null, error: null } as any);
    });

    await act(async () => {
      await useAppStore.getState().loadUserConfig();
    });

    expect(useAppStore.getState().userConfig).toBeNull();
  });

  it("surfaces error code in wrapped command failures", async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: false,
      data: null,
      error: "title is required",
      error_code: "VALIDATION",
    } as any);

    await expect(useAppStore.getState().createTodo("")).rejects.toThrow(
      "[VALIDATION] title is required",
    );
  });

  it("keeps legacy wrapped error format without code", async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: false,
      data: null,
      error: "legacy failure",
    } as any);

    await expect(useAppStore.getState().createTodo("")).rejects.toThrow(
      "legacy failure",
    );
  });

  it("falls back to command name when wrapped error message is empty", async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: false,
      data: null,
      error: null,
      error_code: "INTERNAL",
    } as any);

    await expect(useAppStore.getState().createTodo("")).rejects.toThrow(
      "[INTERNAL] create_todo failed",
    );
  });

  it("parses structured error code from string payload", () => {
    useAppStore.getState().setError("[INVALID_STATE] timer is already running");
    const store = useAppStore.getState();
    expect(store.errorCode).toBe("INVALID_STATE");
    expect(store.error).toBe("timer is already running");
  });

  it("accepts object error payload", () => {
    useAppStore.getState().setError({
      code: "NETWORK",
      message: "request timeout",
    });
    const store = useAppStore.getState();
    expect(store.errorCode).toBe("NETWORK");
    expect(store.error).toBe("request timeout");
  });

  it("links todo github metadata and updates local state", async () => {
    const baseTodo = {
      id: "t1",
      title: "Task",
      description: null,
      status: "todo" as const,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    useAppStore.setState({ todos: [baseTodo] } as any);

    vi.mocked(invoke).mockResolvedValue({
      success: true,
      data: {
        ...baseTodo,
        github_issue_id: 101,
        github_issue_number: 12,
        github_project_id: 88,
      },
      error: null,
    } as any);

    await act(async () => {
      await useAppStore.getState().linkTodoGithub("t1", 101, 12, 88);
    });

    expect(vi.mocked(invoke)).toHaveBeenCalledWith("link_todo_github", {
      id: "t1",
      issue_id: 101,
      issue_number: 12,
      project_id: 88,
    });
    const updated = useAppStore.getState().todos.find((t) => t.id === "t1");
    expect(updated?.github_issue_id).toBe(101);
    expect(updated?.github_issue_number).toBe(12);
    expect(updated?.github_project_id).toBe(88);
  });

  it("clears todo github metadata and updates local state", async () => {
    const baseTodo = {
      id: "t2",
      title: "Task2",
      description: null,
      status: "todo" as const,
      github_issue_id: 11,
      github_issue_number: 22,
      github_project_id: 33,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    useAppStore.setState({ todos: [baseTodo] } as any);

    vi.mocked(invoke).mockResolvedValue({
      success: true,
      data: {
        ...baseTodo,
        github_issue_id: null,
        github_issue_number: null,
        github_project_id: null,
      },
      error: null,
    } as any);

    await act(async () => {
      await useAppStore.getState().clearTodoGithubLink("t2");
    });

    expect(vi.mocked(invoke)).toHaveBeenCalledWith("clear_todo_github_link", {
      id: "t2",
    });
    const updated = useAppStore.getState().todos.find((t) => t.id === "t2");
    expect(updated?.github_issue_id).toBeNull();
    expect(updated?.github_issue_number).toBeNull();
    expect(updated?.github_project_id).toBeNull();
  });

  it("sets error state when linkTodoGithub fails", async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: false,
      data: null,
      error: "invalid issue id",
      error_code: "VALIDATION",
    } as any);

    await expect(
      useAppStore.getState().linkTodoGithub("t1", 0, 1, 1),
    ).rejects.toThrow("[VALIDATION] invalid issue id");

    const store = useAppStore.getState();
    expect(store.errorCode).toBe("VALIDATION");
    expect(store.error).toBe("invalid issue id");
  });

  it("sets error state when clearTodoGithubLink fails", async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: false,
      data: null,
      error: "todo not found",
      error_code: "NOT_FOUND",
    } as any);

    await expect(
      useAppStore.getState().clearTodoGithubLink("missing"),
    ).rejects.toThrow("[NOT_FOUND] todo not found");

    const store = useAppStore.getState();
    expect(store.errorCode).toBe("NOT_FOUND");
    expect(store.error).toBe("todo not found");
  });

  it("runs github sync dry-run and returns report", async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: true,
      data: {
        dry_run: true,
        pending_items: 3,
        supported_items: 2,
        unsupported_items: 1,
        invalid_items: 0,
        target: { owner: "acme", repo: "pomoflow-rs", project_number: 1 },
        errors: [],
      },
      error: null,
    } as any);

    const report = await useAppStore.getState().runGithubSync();
    expect(report.pending_items).toBe(3);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("run_github_sync", {
      dry_run: true,
    });
  });

  it("uses snake_case payload for todo tag commands", async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: true,
      data: [],
      error: null,
    } as any);

    await act(async () => {
      await useAppStore.getState().assignTagToTodo("todo-1", "tag-1");
      await useAppStore.getState().removeTagFromTodo("todo-1", "tag-1");
      await useAppStore.getState().getTodoTags("todo-1");
    });

    expect(vi.mocked(invoke)).toHaveBeenCalledWith("assign_tag_to_todo", {
      todo_id: "todo-1",
      tag_id: "tag-1",
    });
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("remove_tag_from_todo", {
      todo_id: "todo-1",
      tag_id: "tag-1",
    });
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_todo_tags", {
      todo_id: "todo-1",
    });
  });
});
