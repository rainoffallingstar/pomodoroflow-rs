import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { act } from "react";
import { TodoList } from "../components/TodoList";
import { useAppStore } from "../stores/appStore";

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

describe("TodoList", () => {
  it("renders todo title", () => {
    const mockStore = {
      todos: [
        {
          id: "1",
          title: "Test Todo 1",
          description: "Test description",
          status: "todo",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
      ],
      createTodo: vi.fn(),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
      setTodoStatus: vi.fn(),
      linkTodoGithub: vi.fn(),
      clearTodoGithubLink: vi.fn(),
      loadTags: vi.fn(),
      assignTagToTodo: vi.fn(),
      tags: [],
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore as any);
    render(<TodoList />);

    expect(screen.getByText("Todo List")).toBeInTheDocument();
    expect(screen.getByText("Test Todo 1")).toBeInTheDocument();
  });

  it("calls createTodo when Add clicked", async () => {
    const user = userEvent.setup();
    const mockStore = {
      todos: [],
      createTodo: vi.fn(async () => ({ id: "new-id" })),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
      setTodoStatus: vi.fn(),
      linkTodoGithub: vi.fn(),
      clearTodoGithubLink: vi.fn(),
      loadTags: vi.fn(),
      assignTagToTodo: vi.fn(),
      tags: [],
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore as any);
    render(<TodoList />);

    await act(async () => {
      await user.type(screen.getByPlaceholderText("Add a new task..."), "New Test Todo");
      await user.click(screen.getByRole("button", { name: "Add" }));
    });

    await waitFor(() => {
      expect(mockStore.createTodo).toHaveBeenCalledWith(
        "New Test Todo",
        undefined,
        "todo",
      );
    });
  });

  it("calls setTodoStatus when checkbox clicked", async () => {
    const mockStore = {
      todos: [
        {
          id: "1",
          title: "Test Todo",
          description: null,
          status: "todo",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
      ],
      createTodo: vi.fn(),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
      setTodoStatus: vi.fn(),
      linkTodoGithub: vi.fn(),
      clearTodoGithubLink: vi.fn(),
      loadTags: vi.fn(),
      assignTagToTodo: vi.fn(),
      tags: [],
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore as any);
    render(<TodoList />);

    fireEvent.click(screen.getByRole("checkbox"));
    await waitFor(() => {
      expect(mockStore.setTodoStatus).toHaveBeenCalledWith("1", "done");
    });
  });

  it("renders github issue badge when todo is linked", () => {
    const mockStore = {
      todos: [
        {
          id: "1",
          title: "Linked Todo",
          description: null,
          status: "todo",
          github_issue_number: 42,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
      ],
      createTodo: vi.fn(),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
      setTodoStatus: vi.fn(),
      linkTodoGithub: vi.fn(),
      clearTodoGithubLink: vi.fn(),
      loadTags: vi.fn(),
      assignTagToTodo: vi.fn(),
      tags: [],
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore as any);
    render(<TodoList />);

    expect(screen.getByText("GitHub #42")).toBeInTheDocument();
  });

  it("calls linkTodoGithub when github link button clicked", async () => {
    const user = userEvent.setup();
    const promptSpy = vi
      .spyOn(window, "prompt")
      .mockReturnValue("101,42,7");
    const mockStore = {
      todos: [
        {
          id: "1",
          title: "Link Me",
          description: null,
          status: "todo",
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
      ],
      createTodo: vi.fn(),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
      setTodoStatus: vi.fn(),
      linkTodoGithub: vi.fn(),
      clearTodoGithubLink: vi.fn(),
      loadTags: vi.fn(),
      assignTagToTodo: vi.fn(),
      tags: [],
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore as any);
    render(<TodoList />);

    await act(async () => {
      await user.click(screen.getByTitle("绑定 GitHub Issue"));
    });

    expect(mockStore.linkTodoGithub).toHaveBeenCalledWith("1", 101, 42, 7);
    promptSpy.mockRestore();
  });

  it("calls clearTodoGithubLink when github unlink button clicked", async () => {
    const user = userEvent.setup();
    const mockStore = {
      todos: [
        {
          id: "1",
          title: "Unlink Me",
          description: null,
          status: "todo",
          github_issue_number: 42,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
      ],
      createTodo: vi.fn(),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
      setTodoStatus: vi.fn(),
      linkTodoGithub: vi.fn(),
      clearTodoGithubLink: vi.fn(),
      loadTags: vi.fn(),
      assignTagToTodo: vi.fn(),
      tags: [],
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore as any);
    render(<TodoList />);

    await act(async () => {
      await user.click(screen.getByTitle("清除 GitHub 关联"));
    });

    expect(mockStore.clearTodoGithubLink).toHaveBeenCalledWith("1");
  });
});
