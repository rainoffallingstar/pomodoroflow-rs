import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TodoList } from "../components/TodoList";
import { useAppStore } from "../stores/appStore";

// Mock the store
vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

describe("TodoList", () => {
  it("renders the todo list component", () => {
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
      loadTodos: vi.fn(),
      createTodo: vi.fn(),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<TodoList />);

    expect(screen.getByText("Todo List")).toBeInTheDocument();
    expect(screen.getByText("Test Todo 1")).toBeInTheDocument();
  });

  it("calls createTodo when new todo is submitted", async () => {
    const mockStore = {
      todos: [],
      loadTodos: vi.fn(),
      createTodo: vi.fn(),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<TodoList />);

    const input = screen.getByPlaceholderText("Add a new task...");
    const button = screen.getByText("Add");

    fireEvent.change(input, { target: { value: "New Test Todo" } });
    fireEvent.click(button);

    expect(mockStore.createTodo).toHaveBeenCalledWith("New Test Todo");
  });

  it("calls toggleTodoStatus when checkbox is clicked", () => {
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
      loadTodos: vi.fn(),
      createTodo: vi.fn(),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<TodoList />);

    const checkbox = screen.getByRole("checkbox");
    fireEvent.click(checkbox);

    expect(mockStore.toggleTodoStatus).toHaveBeenCalledWith("1");
  });

  it("displays empty state when no todos", () => {
    const mockStore = {
      todos: [],
      loadTodos: vi.fn(),
      createTodo: vi.fn(),
      updateTodo: vi.fn(),
      deleteTodo: vi.fn(),
      toggleTodoStatus: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<TodoList />);

    expect(screen.getByText("Todo List")).toBeInTheDocument();
  });
});
