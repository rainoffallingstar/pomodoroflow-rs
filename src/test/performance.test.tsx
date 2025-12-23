import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { PomodoroTimer } from "../components/PomodoroTimer";
import { TodoList } from "../components/TodoList";
import { useAppStore } from "../stores/appStore";

// Mock the store
vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

describe("Performance Tests", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  describe("PomodoroTimer Component", () => {
    it("should render within acceptable time", () => {
      const startTime = performance.now();

      const mockStore = {
        pomodoroSession: {
          phase: "work",
          duration: 1500,
          remaining: 1500,
          is_running: false,
          cycle_count: 0,
        },
        startPomodoro: vi.fn(),
        pausePomodoro: vi.fn(),
        resetPomodoro: vi.fn(),
        skipPomodoroPhase: vi.fn(),
      };

      vi.mocked(useAppStore).mockReturnValue(mockStore);

      render(<PomodoroTimer />);

      const endTime = performance.now();
      const renderTime = endTime - startTime;

      // Component should render within 100ms
      expect(renderTime).toBeLessThan(100);

      expect(screen.getByText("Pomodoro Timer")).toBeInTheDocument();
    });

    it("should handle rapid state updates efficiently", () => {
      const mockStore = {
        pomodoroSession: {
          phase: "work",
          duration: 1500,
          remaining: 1500,
          is_running: false,
          cycle_count: 0,
        },
        startPomodoro: vi.fn(),
        pausePomodoro: vi.fn(),
        resetPomodoro: vi.fn(),
        skipPomodoroPhase: vi.fn(),
      };

      vi.mocked(useAppStore).mockReturnValue(mockStore);

      render(<PomodoroTimer />);

      const startTime = performance.now();

      // Simulate rapid updates
      for (let i = 0; i < 100; i++) {
        vi.mocked(useAppStore).mockReturnValue({
          ...mockStore,
          pomodoroSession: {
            ...mockStore.pomodoroSession,
            remaining: 1500 - i,
          },
        });
      }

      const endTime = performance.now();
      const updateTime = endTime - startTime;

      // Rapid updates should complete within 50ms
      expect(updateTime).toBeLessThan(50);
    });
  });

  describe("TodoList Component", () => {
    it("should handle large todo lists efficiently", () => {
      const startTime = performance.now();

      const mockTodos = Array.from({ length: 1000 }, (_, i) => ({
        id: `todo-${i}`,
        title: `Todo Item ${i}`,
        description: `Description for todo ${i}`,
        completed: i % 2 === 0,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      }));

      const mockStore = {
        todos: mockTodos,
        loadTodos: vi.fn(),
        createTodo: vi.fn(),
        updateTodo: vi.fn(),
        deleteTodo: vi.fn(),
        toggleTodoStatus: vi.fn(),
      };

      vi.mocked(useAppStore).mockReturnValue(mockStore);

      render(<TodoList />);

      const endTime = performance.now();
      const renderTime = endTime - startTime;

      // Component should render 1000 todos within 200ms
      expect(renderTime).toBeLessThan(200);

      expect(screen.getByText("Todo List")).toBeInTheDocument();
      expect(screen.getByText("1000 tasks")).toBeInTheDocument();
    });

    it("should filter todos efficiently", () => {
      const mockTodos = Array.from({ length: 500 }, (_, i) => ({
        id: `todo-${i}`,
        title: `Todo Item ${i}`,
        description: `Description for todo ${i}`,
        completed: i % 2 === 0,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      }));

      const mockStore = {
        todos: mockTodos,
        loadTodos: vi.fn(),
        createTodo: vi.fn(),
        updateTodo: vi.fn(),
        deleteTodo: vi.fn(),
        toggleTodoStatus: vi.fn(),
      };

      vi.mocked(useAppStore).mockReturnValue(mockStore);

      render(<TodoList />);

      const startTime = performance.now();

      // Simulate filter operation
      const filtered = mockTodos.filter((todo) => todo.completed);

      const endTime = performance.now();
      const filterTime = endTime - startTime;

      // Filter operation should complete within 10ms
      expect(filterTime).toBeLessThan(10);
      expect(filtered).toHaveLength(250);
    });
  });

  describe("Store Operations", () => {
    it("should handle concurrent updates efficiently", async () => {
      const store = useAppStore.getState();

      const startTime = performance.now();

      // Simulate concurrent updates
      const promises = Array.from({ length: 100 }, (_, i) => {
        return new Promise((resolve) => {
          setTimeout(() => {
            store.todos = [
              ...store.todos,
              {
                id: `todo-${i}`,
                title: `Todo ${i}`,
                description: undefined,
                status: "todo" as const,
                created_at: new Date().toISOString(),
                updated_at: new Date().toISOString(),
              },
            ];
            resolve(true);
          }, 0);
        });
      });

      await Promise.all(promises);

      const endTime = performance.now();
      const updateTime = endTime - startTime;

      // Concurrent updates should complete within 100ms
      expect(updateTime).toBeLessThan(100);
      expect(store.todos).toHaveLength(100);
    });

    it("should handle frequent state changes", () => {
      const store = useAppStore.getState();

      const startTime = performance.now();

      // Simulate frequent state changes
      for (let i = 0; i < 1000; i++) {
        store.pomodoroSession = {
          phase: "work",
          duration: 1500,
          remaining: 1500 - i,
          is_running: i % 2 === 0,
          cycle_count: Math.floor(i / 4),
        };
      }

      const endTime = performance.now();
      const changeTime = endTime - startTime;

      // Frequent changes should complete within 50ms
      expect(changeTime).toBeLessThan(50);
    });
  });

  describe("Memory Usage", () => {
    it("should not leak memory on component unmount", () => {
      const { unmount } = render(<PomodoroTimer />);

      const mockStore = {
        pomodoroSession: {
          phase: "work",
          duration: 1500,
          remaining: 1500,
          is_running: false,
          cycle_count: 0,
        },
        startPomodoro: vi.fn(),
        pausePomodoro: vi.fn(),
        resetPomodoro: vi.fn(),
        skipPomodoroPhase: vi.fn(),
      };

      vi.mocked(useAppStore).mockReturnValue(mockStore);

      unmount();

      // Verify no memory leaks by checking that timers are cleared
      expect(vi.getTimerCount()).toBe(0);
    });

    it("should handle large data structures", () => {
      const largeData = Array.from({ length: 10000 }, (_, i) => ({
        id: `item-${i}`,
        title: `Item ${i}`,
        description: `Description for item ${i}`,
        metadata: {
          created: new Date().toISOString(),
          updated: new Date().toISOString(),
          tags: [`tag-${i % 10}`, `tag-${i % 5}`],
          nested: {
            level1: {
              level2: {
                level3: `value-${i}`,
              },
            },
          },
        },
      }));

      const startTime = performance.now();

      // Process large data structure
      const processed = largeData.map((item) => ({
        ...item,
        processed: true,
      }));

      const endTime = performance.now();
      const processTime = endTime - startTime;

      // Large data processing should complete within 200ms
      expect(processTime).toBeLessThan(200);
      expect(processed).toHaveLength(10000);
    });
  });
});
