import { create } from "zustand";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";

// 检查是否在 Tauri 环境中运行
const isRunningInTauri =
  typeof window !== "undefined" && !!(window as any).__TAURI__;
const shouldUseTauriApi =
  isRunningInTauri || typeof (globalThis as any).vi !== "undefined";

export interface Todo {
  id: string;
  title: string;
  description?: string;
  status: "todo" | "in_progress" | "done";
  github_issue_id?: number | null;
  github_project_id?: number | null;
  github_issue_number?: number | null;
  created_at: string;
  updated_at: string;
  tags?: Tag[];
}

export interface PomodoroSession {
  phase: "work" | "short_break" | "long_break";
  duration: number;
  remaining: number;
  is_running: boolean;
  cycle_count: number;
  started_at?: string;
}

export interface UserConfig {
  github_token_encrypted: string;
  github_username: string;
  last_sync_cursor?: string | null;
  selected_project_owner?: string | null;
  selected_project_repo?: string | null;
  selected_project_number?: number | null;
  pomodoro_work_duration: number;
  pomodoro_short_break_duration: number;
  pomodoro_long_break_duration: number;
  pomodoro_cycles_until_long_break: number;
  notifications_enabled: boolean;
  sound_enabled: boolean;
  theme: string;
}

export interface GithubSyncTarget {
  owner: string;
  repo: string;
  project_number: number;
}

export interface GithubSyncReport {
  dry_run: boolean;
  pending_items: number;
  supported_items: number;
  unsupported_items: number;
  invalid_items: number;
  target: GithubSyncTarget;
  errors: string[];
}

export interface Tag {
  id: string;
  name: string;
  color: string;
  created_at?: string;
}

interface CommandResult<T> {
  success: boolean;
  data: T | null;
  error: string | null;
  error_code?: string | null;
}

type TagTuple = [string, string, string];
export interface AppErrorPayload {
  message: string;
  code?: string | null;
}
interface InvokeOptions {
  allowNullData?: boolean;
}

function parseErrorPayload(input: string): AppErrorPayload {
  const trimmed = input.trim();
  const match = trimmed.match(/^\[([A-Z0-9_]+)\]\s*(.+)$/);
  if (match) {
    return {
      code: match[1],
      message: match[2].trim(),
    };
  }
  return { message: trimmed };
}

async function invokeCommand<T>(
  command: string,
  payload?: Record<string, unknown>,
  options?: InvokeOptions,
): Promise<T> {
  const result = await invoke<CommandResult<T> | T>(command, payload);
  // Backward compatibility: some callers/tests still mock raw payloads.
  if (
    typeof result === "object" &&
    result !== null &&
    "success" in result
  ) {
    const wrapped = result as CommandResult<T>;
    if (!wrapped.success) {
      const message = wrapped.error || `${command} failed`;
      const code = wrapped.error_code?.trim();
      throw new Error(code ? `[${code}] ${message}` : message);
    }
    if (wrapped.data === null && !options?.allowNullData) {
      throw new Error(`${command} returned empty data`);
    }
    return wrapped.data as T;
  }
  return result as T;
}

interface AppState {
  todos: Todo[];
  pomodoroSession: PomodoroSession | null;
  userConfig: UserConfig | null;
  isLoading: boolean;
  theme: "light" | "dark" | "system";
  unlistenFunctions: (() => void)[]; // 存储事件监听器清理函数
  error: string | null; // 全局错误信息
  errorCode: string | null;
  success: string | null; // 全局成功信息
  selectedTodoId: string | null; // 当前选中的待办事项 ID

  // Actions
  initializeApp: () => Promise<void>;
  loadTodos: () => Promise<void>;
  loadPomodoroSession: () => Promise<void>;
  loadUserConfig: () => Promise<void>;
  setupEventListeners: () => Promise<void>;
  cleanupEventListeners: () => void;
  setError: (error: string | AppErrorPayload | null) => void;
  setSuccess: (success: string | null) => void;
  clearMessages: () => void;

  // Pomodoro actions
  startPomodoro: () => Promise<void>;
  pausePomodoro: () => Promise<void>;
  resetPomodoro: () => Promise<void>;
  skipPomodoroPhase: () => Promise<void>;

  // Todo actions
  createTodo: (title: string, description?: string, initialStatus?: "todo" | "in_progress" | "done") => Promise<Todo | null>;
  updateTodo: (id: string, updates: Partial<Todo>) => Promise<void>;
  deleteTodo: (id: string) => Promise<void>;
  toggleTodoStatus: (id: string) => Promise<void>;
  setTodoStatus: (id: string, status: "todo" | "in_progress" | "done") => Promise<void>;
  linkTodoGithub: (
    id: string,
    issueId: number,
    issueNumber: number,
    projectId: number,
  ) => Promise<void>;
  clearTodoGithubLink: (id: string) => Promise<void>;

  // Config actions
  saveUserConfig: (config: UserConfig) => Promise<void>;
  runGithubSync: (dryRun?: boolean) => Promise<GithubSyncReport>;

  // Theme actions
  setTheme: (theme: "light" | "dark" | "system") => void;
  toggleTheme: () => void;

  // Todo selection actions
  selectTodo: (id: string | null) => void;
  getSelectedTodo: () => Todo | null;

  // Tag actions
  tags: Tag[];
  loadTags: () => Promise<void>;
  createTag: (name: string, color: string) => Promise<void>;
  deleteTag: (id: string) => Promise<void>;
  assignTagToTodo: (todoId: string, tagId: string) => Promise<void>;
  removeTagFromTodo: (todoId: string, tagId: string) => Promise<void>;
  getTodoTags: (todoId: string) => Promise<Tag[]>;
}

export const useAppStore = create<AppState>((set, get) => ({
  todos: [],
  pomodoroSession: null,
  userConfig: null,
  isLoading: false,
  theme: "system",
  unlistenFunctions: [],
  error: null,
  errorCode: null,
  success: null,
  selectedTodoId: null,
  tags: [],

  initializeApp: async () => {
    set({ isLoading: true });
    try {
      // 检查是否在 Tauri 环境中运行
      if (!shouldUseTauriApi) {
        console.warn("Running in browser mode - Tauri features disabled");
        // 在浏览器中运行时，只加载基本功能
        return;
      }

      console.log("Initializing app in Tauri mode...");

      // 设置事件监听
      await get().setupEventListeners();

      // 并行加载数据，添加错误处理
      const promises = [
        get()
          .loadTodos()
          .catch((err) => {
            console.warn("Failed to load todos:", err);
            return [];
          }),
        get()
          .loadPomodoroSession()
          .catch((err) => {
            console.warn("Failed to load pomodoro session:", err);
            return null;
          }),
        get()
          .loadUserConfig()
          .catch((err) => {
            console.warn("Failed to load user config:", err);
            return null;
          }),
      ];

      await Promise.allSettled(promises);

      // 从配置中加载主题
      const config = get().userConfig;
      if (config) {
        get().setTheme(config.theme as "light" | "dark" | "system");
      }

      console.log("App initialization completed");
    } catch (error) {
      console.error("App initialization failed:", error);
    } finally {
      set({ isLoading: false });
    }
  },

  setupEventListeners: async () => {
    // 先清理现有监听器
    get().cleanupEventListeners();

    if (!shouldUseTauriApi) {
      console.warn("Skipping event listeners setup - not in Tauri environment");
      return;
    }

    console.log("Setting up pomodoro listeners...");

    const unlistenFunctions: (() => void)[] = [];

    try {
      // 监听番茄钟进度更新事件
      const unlistenTick = await listen<PomodoroSession>("pomodoro-tick", (event) => {
        if (!event.payload) {
          return;
        }
        set({ pomodoroSession: event.payload });
      });
      unlistenFunctions.push(() => unlistenTick());

      // 监听番茄钟阶段完成事件
      const unlistenPhase = await listen<PomodoroSession>(
        "pomodoro-phase-completed",
        (event) => {
          if (!event.payload) {
            return;
          }

          set({ pomodoroSession: event.payload });
          console.log("Pomodoro phase completed:", event.payload);
        },
      );
      unlistenFunctions.push(() => unlistenPhase());

      console.log("Pomodoro listeners setup completed");
    } catch (error) {
      console.error("Failed to setup event listeners:", error);
    }

    // 存储清理函数
    set({ unlistenFunctions });
  },

  cleanupEventListeners: () => {
    const { unlistenFunctions } = get();
    unlistenFunctions.forEach((unlisten) => unlisten());
    set({ unlistenFunctions: [] });
  },

  setError: (errorInput: string | AppErrorPayload | null) => {
    if (!errorInput) {
      set({ error: null, errorCode: null });
      return;
    }

    const parsed =
      typeof errorInput === "string" ? parseErrorPayload(errorInput) : errorInput;
    set({ error: parsed.message, errorCode: parsed.code ?? null });
    // 3秒后自动清除错误
    setTimeout(() => {
      get().clearMessages();
    }, 3000);
  },

  setSuccess: (success: string | null) => {
    set({ success });
    // 3秒后自动清除成功消息
    if (success) {
      setTimeout(() => {
        get().clearMessages();
      }, 3000);
    }
  },

  clearMessages: () => {
    set({ error: null, errorCode: null, success: null });
  },

  loadTodos: async () => {
    try {
      if (!shouldUseTauriApi) {
        console.warn("Skipping loadTodos - not in Tauri environment");
        return;
      }
      const todos = await invokeCommand<Todo[]>("get_todos");
      const todosWithTags = await Promise.all(
        todos.map(async (todo) => {
          const tagPayload = await invokeCommand<unknown>("get_todo_tags", {
            todo_id: todo.id,
          }).catch(() => []);
          const tags = Array.isArray(tagPayload)
            ? (tagPayload.filter(
                (item): item is TagTuple =>
                  Array.isArray(item) && item.length === 3,
              ) as TagTuple[])
            : [];
          return {
            ...todo,
            tags: tags.map(([id, name, color]) => ({ id, name, color })),
          };
        }),
      );
      set({ todos: todosWithTags });
      console.log("Loaded todos:", todosWithTags.length);
    } catch (error) {
      console.error("Failed to load todos:", error);
    }
  },

  loadPomodoroSession: async () => {
    try {
      if (!shouldUseTauriApi) {
        console.warn("Skipping loadPomodoroSession - not in Tauri environment");
        return;
      }
      const session = await invokeCommand<PomodoroSession | null>(
        "get_pomodoro_session",
        undefined,
        { allowNullData: true },
      );
      set({ pomodoroSession: session });
      console.log("Loaded pomodoro session:", session);
    } catch (error) {
      console.error("Failed to load pomodoro session:", error);
    }
  },

  loadUserConfig: async () => {
    try {
      if (!shouldUseTauriApi) {
        console.warn("Skipping loadUserConfig - not in Tauri environment");
        return;
      }
      const config = await invokeCommand<UserConfig | null>(
        "get_user_config",
        undefined,
        { allowNullData: true },
      );
      set({ userConfig: config });
      console.log("Loaded user config:", config);
    } catch (error) {
      console.error("Failed to load user config:", error);
    }
  },

  startPomodoro: async () => {
    try {
      console.log("🍅 Starting pomodoro...");
      await invokeCommand<{}>("start_pomodoro");
      console.log("✅ Pomodoro start command sent");
      await get().loadPomodoroSession();
      console.log("📊 Session loaded:", get().pomodoroSession);
    } catch (error) {
      console.error("❌ Failed to start pomodoro:", error);
      get().setError(error instanceof Error ? error.message : "启动番茄钟失败");
    }
  },

  pausePomodoro: async () => {
    try {
      console.log("⏸️ Pausing pomodoro...");
      await invokeCommand<{}>("pause_pomodoro");
      console.log("✅ Pomodoro paused");
      await get().loadPomodoroSession();
    } catch (error) {
      console.error("❌ Failed to pause pomodoro:", error);
    }
  },

  resetPomodoro: async () => {
    try {
      console.log("🔄 Resetting pomodoro...");
      await invokeCommand<{}>("reset_pomodoro");
      console.log("✅ Pomodoro reset");
      await get().loadPomodoroSession();
    } catch (error) {
      console.error("❌ Failed to reset pomodoro:", error);
    }
  },

  skipPomodoroPhase: async () => {
    try {
      await invokeCommand<{}>("skip_pomodoro_phase");
      await get().loadPomodoroSession();
    } catch (error) {
      console.error("Failed to skip pomodoro phase:", error);
    }
  },

  createTodo: async (title: string, description?: string, initialStatus?: "todo" | "in_progress" | "done") => {
    const { todos } = get();
    
    // 创建临时待办事项（乐观更新）
    const optimisticId = `temp-${Date.now()}`;
    const finalStatus = initialStatus || "todo";
    const tempTodo: Todo = {
      id: optimisticId,
      title,
      description,
      status: finalStatus,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    // 立即更新UI
    set({ todos: [...todos, tempTodo] });

    try {
      // 使用真实的后端命令，传递初始状态
      const payload: any = {
        title,
        description,
        status: finalStatus,
      };

      const createdTodo = await invokeCommand<Todo>("create_todo", payload);
      if (createdTodo) {
        // 成功后替换临时待办事项为后端权威返回值
        set((state) => {
          const newTodos = state.todos.filter(t => t.id !== optimisticId);
          const updatedTodos = [...newTodos, createdTodo];
          return { todos: updatedTodos };
        });
        get().setSuccess("待办事项创建成功");
        return createdTodo;
      } else {
        throw new Error("创建待办事项失败");
      }
    } catch (error) {
      console.error("Failed to create todo:", error);
      // 失败后回滚
      set((state) => ({
        todos: state.todos.filter((todo) => todo.id !== optimisticId),
      }));
      // 显示错误提示
      get().setError(
        error instanceof Error ? error.message : "创建待办事项失败，请重试",
      );
      throw error;
    }
  },

  updateTodo: async (id: string, updates: Partial<Todo>) => {
    try {
      await invokeCommand<Todo>("update_todo", { id, ...updates });
      await get().loadTodos();
    } catch (error) {
      console.error("Failed to update todo:", error);
    }
  },

  deleteTodo: async (id: string) => {
    const { todos } = get();
    // 保存被删除的待办事项用于回滚
    const todoToDelete = todos.find((todo) => todo.id === id);
    if (!todoToDelete) return;

    // 立即从UI中移除（乐观更新）
    set({ todos: todos.filter((todo) => todo.id !== id) });

    try {
      // 使用真实的后端命令，线程安全问题已修复
      const deleted = await invokeCommand<boolean>("delete_todo", { id });
      if (!deleted) {
        throw new Error("Delete operation failed");
      }
    } catch (error) {
      console.error("Failed to delete todo:", error);
      // 失败后回滚
      if (todoToDelete) {
        set({ todos: [...todos] }); // 恢复原列表
      }
      // 显示错误提示
      get().setError("删除待办事项失败，请重试");
    }
  },

  toggleTodoStatus: async (id: string) => {
    const todo = get().todos.find((item) => item.id === id);
    if (!todo) return;

    const nextStatus =
      todo.status === "todo"
        ? "in_progress"
        : todo.status === "in_progress"
          ? "done"
          : "todo";

    await get().setTodoStatus(id, nextStatus);
  },

  setTodoStatus: async (id: string, status: "todo" | "in_progress" | "done") => {
    const { todos } = get();
    const originalTodos = [...todos];

    set({
      todos: todos.map((todo) =>
        todo.id === id
          ? { ...todo, status, updated_at: new Date().toISOString() }
          : todo,
      ),
    });

    try {
      await invokeCommand<Todo>("set_todo_status", { id, status });
      await get().loadTodos();
    } catch (error) {
      console.error("Failed to set todo status:", error);
      set({ todos: originalTodos });
      get().setError("更新待办事项状态失败，请重试");
    }
  },

  saveUserConfig: async (config: UserConfig) => {
    try {
      await invokeCommand<{}>("save_user_config", { config });

      // 更新本地状态
      set({ userConfig: config });

      get().setSuccess("设置保存成功");
    } catch (error) {
      console.error("Failed to save user config:", error);
      get().setError(error instanceof Error ? error.message : "保存配置失败");
      throw error; // 重新抛出错误让调用者处理
    }
  },

  runGithubSync: async (dryRun = true) => {
    try {
      const report = await invokeCommand<GithubSyncReport>("run_github_sync", {
        dry_run: dryRun,
      });
      get().setSuccess(
        `同步检查完成: pending=${report.pending_items}, supported=${report.supported_items}`,
      );
      return report;
    } catch (error) {
      console.error("Failed to run github sync:", error);
      get().setError(error instanceof Error ? error.message : "执行 GitHub 同步失败");
      throw error;
    }
  },

  setTheme: (theme: "light" | "dark" | "system") => {
    set({ theme });

    // 应用主题到 DOM
    const root = document.documentElement;
    const systemTheme = window.matchMedia("(prefers-color-scheme: dark)")
      .matches
      ? "dark"
      : "light";
    const effectiveTheme = theme === "system" ? systemTheme : theme;

    root.className = effectiveTheme;
    root.setAttribute("data-theme", theme);
  },

  toggleTheme: () => {
    const currentTheme = get().theme;
    const newTheme =
      currentTheme === "light"
        ? "dark"
        : currentTheme === "dark"
          ? "system"
          : "light";
    get().setTheme(newTheme);
  },

  // Todo selection actions
  selectTodo: (id: string | null) => {
    set({ selectedTodoId: id });
  },

  getSelectedTodo: () => {
    const { todos, selectedTodoId } = get();
    if (!selectedTodoId) return null;
    return todos.find((todo) => todo.id === selectedTodoId) || null;
  },

  // ========================================================================
  // Tag methods
  // ========================================================================

  loadTags: async () => {
    try {
      if (!shouldUseTauriApi) {
        console.warn("Skipping loadTags - not in Tauri environment");
        return;
      }
      const result = await invokeCommand<TagTuple[]>("get_tags");
      const tags = result.map(([id, name, color]) => ({ id, name, color }));
      set({ tags });
      console.log("Loaded tags:", tags.length);
    } catch (error) {
      console.error("Failed to load tags:", error);
    }
  },

  createTag: async (name: string, color: string) => {
    try {
      if (!shouldUseTauriApi) {
        console.warn("Skipping createTag - not in Tauri environment");
        return;
      }
      const result = await invokeCommand<TagTuple>("create_tag", {
        name,
        color,
      });
      const newTag: Tag = {
        id: result[0],
        name: result[1],
        color: result[2],
        created_at: new Date().toISOString(),
      };
      set({ tags: [...get().tags, newTag] });
      get().setSuccess("标签创建成功");
    } catch (error) {
      console.error("Failed to create tag:", error);
      get().setError(error instanceof Error ? error.message : "创建标签失败");
      throw error;
    }
  },

  linkTodoGithub: async (
    id: string,
    issueId: number,
    issueNumber: number,
    projectId: number,
  ) => {
    try {
      const updatedTodo = await invokeCommand<Todo>("link_todo_github", {
        id,
        issue_id: issueId,
        issue_number: issueNumber,
        project_id: projectId,
      });

      set((state) => ({
        todos: state.todos.map((todo) =>
          todo.id === id ? { ...todo, ...updatedTodo } : todo,
        ),
      }));
    } catch (error) {
      console.error("Failed to link todo github:", error);
      get().setError((error as Error).message || "关联 GitHub 失败");
      throw error;
    }
  },

  clearTodoGithubLink: async (id: string) => {
    try {
      const updatedTodo = await invokeCommand<Todo>("clear_todo_github_link", {
        id,
      });

      set((state) => ({
        todos: state.todos.map((todo) =>
          todo.id === id ? { ...todo, ...updatedTodo } : todo,
        ),
      }));
    } catch (error) {
      console.error("Failed to clear todo github link:", error);
      get().setError((error as Error).message || "清除 GitHub 关联失败");
      throw error;
    }
  },

  deleteTag: async (id: string) => {
    try {
      if (!shouldUseTauriApi) {
        console.warn("Skipping deleteTag - not in Tauri environment");
        return;
      }
      const deleted = await invokeCommand<boolean>("delete_tag", { id });
      if (deleted) {
        set({ tags: get().tags.filter((t) => t.id !== id) });
        get().setSuccess("标签删除成功");
      } else {
        throw new Error("删除标签失败");
      }
    } catch (error) {
      console.error("Failed to delete tag:", error);
      get().setError(error instanceof Error ? error.message : "删除标签失败");
      throw error;
    }
  },

  assignTagToTodo: async (todoId: string, tagId: string) => {
    try {
      if (!shouldUseTauriApi) {
        console.warn("Skipping assignTagToTodo - not in Tauri environment");
        return;
      }
      await invokeCommand<{}>("assign_tag_to_todo", {
        todo_id: todoId,
        tag_id: tagId,
      });
      get().setSuccess("标签分配成功");
    } catch (error) {
      console.error("Failed to assign tag to todo:", error);
      get().setError(error instanceof Error ? error.message : "分配标签失败");
      throw error;
    }
  },

  removeTagFromTodo: async (todoId: string, tagId: string) => {
    try {
      if (!shouldUseTauriApi) {
        console.warn("Skipping removeTagFromTodo - not in Tauri environment");
        return;
      }
      await invokeCommand<{}>("remove_tag_from_todo", {
        todo_id: todoId,
        tag_id: tagId,
      });
      get().setSuccess("标签移除成功");
    } catch (error) {
      console.error("Failed to remove tag from todo:", error);
      get().setError(error instanceof Error ? error.message : "移除标签失败");
      throw error;
    }
  },

  getTodoTags: async (todoId: string) => {
    try {
      if (!shouldUseTauriApi) {
        console.warn("Skipping getTodoTags - not in Tauri environment");
        return [];
      }
      const tags = await invokeCommand<TagTuple[]>("get_todo_tags", {
        todo_id: todoId,
      });
      return tags.map(([id, name, color]) => ({ id, name, color }));
    } catch (error) {
      console.error("Failed to get todo tags:", error);
      return [];
    }
  },
}));
