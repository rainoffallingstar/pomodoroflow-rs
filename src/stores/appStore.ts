import { create } from "zustand";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";

// 检查是否在 Tauri 环境中运行
const isRunningInTauri =
  typeof window !== "undefined" && !!(window as any).__TAURI__;

export interface Todo {
  id: string;
  title: string;
  description?: string;
  status: "todo" | "in_progress" | "done";
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
  pomodoro_work_duration: number;
  pomodoro_short_break_duration: number;
  pomodoro_long_break_duration: number;
  pomodoro_cycles_until_long_break: number;
  notifications_enabled: boolean;
  sound_enabled: boolean;
  theme: string;
}

export interface Tag {
  id: string;
  name: string;
  color: string;
  created_at?: string;
}

interface AppState {
  todos: Todo[];
  pomodoroSession: PomodoroSession | null;
  userConfig: UserConfig | null;
  isLoading: boolean;
  theme: "light" | "dark" | "system";
  unlistenFunctions: (() => void)[]; // 存储事件监听器清理函数
  error: string | null; // 全局错误信息
  success: string | null; // 全局成功信息
  selectedTodoId: string | null; // 当前选中的待办事项 ID

  // Actions
  initializeApp: () => void;
  loadTodos: () => Promise<void>;
  loadPomodoroSession: () => Promise<void>;
  loadUserConfig: () => Promise<void>;
  setupEventListeners: () => void;
  cleanupEventListeners: () => void;
  setError: (error: string | null) => void;
  setSuccess: (success: string | null) => void;
  clearMessages: () => void;

  // Pomodoro actions
  startPomodoro: () => Promise<void>;
  pausePomodoro: () => Promise<void>;
  resetPomodoro: () => Promise<void>;
  skipPomodoroPhase: () => Promise<void>;

  // Todo actions
  createTodo: (title: string, description?: string, initialStatus?: "todo" | "in_progress" | "done") => Promise<void>;
  updateTodo: (id: string, updates: Partial<Todo>) => Promise<void>;
  deleteTodo: (id: string) => Promise<void>;
  toggleTodoStatus: (id: string) => Promise<void>;

  // Config actions
  saveUserConfig: (config: UserConfig) => Promise<void>;

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
  success: null,
  selectedTodoId: null,
  tags: [],

  initializeApp: async () => {
    set({ isLoading: true });
    try {
      // 检查是否在 Tauri 环境中运行
      if (!isRunningInTauri) {
        console.warn("Running in browser mode - Tauri features disabled");
        // 在浏览器中运行时，只加载基本功能
        return;
      }

      console.log("Initializing app in Tauri mode...");

      // 设置事件监听
      get().setupEventListeners();

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

    if (!isRunningInTauri) {
      console.warn("Skipping event listeners setup - not in Tauri environment");
      return;
    }

    console.log("Setting up pomodoro listeners...");

    const unlistenFunctions: (() => void)[] = [];

    try {
      // 监听番茄钟进度更新事件
      const unlistenTick = await listen<string>("pomodoro-tick", (event) => {
        try {
          const data = JSON.parse(event.payload) as PomodoroSession;
          set({ pomodoroSession: data });
        } catch (error) {
          console.error("Failed to parse pomodoro-tick event:", error);
        }
      });
      unlistenFunctions.push(() => unlistenTick());

      // 监听番茄钟阶段完成事件
      const unlistenPhase = await listen<string>(
        "pomodoro-phase-completed",
        async (event) => {
          try {
            const data = JSON.parse(event.payload) as PomodoroSession;
            set({ pomodoroSession: data });
            console.log("Pomodoro phase completed:", data);

            // 重新加载会话状态以获取新阶段信息（后端已自动切换到下一阶段）
            await get().loadPomodoroSession();

            // 自动开始下一阶段计时
            await get().startPomodoro();
            console.log("Auto-started next phase");

            // 可以在这里显示通知或播放声音
            if (get().userConfig?.notifications_enabled) {
              // 这里可以调用显示通知的命令
            }
          } catch (error) {
            console.error(
              "Failed to parse pomodoro-phase-completed event:",
              error,
            );
          }
        },
      );
      unlistenFunctions.push(() => unlistenPhase());

      // 添加事件系统健康检查
      const healthCheckInterval = setInterval(async () => {
        try {
          const session = await get().loadPomodoroSession();
          console.log("Pomodoro health check completed");
        } catch (error) {
          console.warn("Pomodoro health check failed:", error);
        }
      }, 5000); // 每5秒检查一次

      unlistenFunctions.push(() => clearInterval(healthCheckInterval));

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

  setError: (error: string | null) => {
    set({ error });
    // 3秒后自动清除错误
    if (error) {
      setTimeout(() => {
        get().clearMessages();
      }, 3000);
    }
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
    set({ error: null, success: null });
  },

  loadTodos: async () => {
    try {
      if (!isRunningInTauri) {
        console.warn("Skipping loadTodos - not in Tauri environment");
        return;
      }
      const result = await invoke<{ data: Todo[] }>("get_todos");
      if (result.data) {
        set({ todos: result.data });
        console.log("Loaded todos:", result.data.length);
      }
    } catch (error) {
      console.error("Failed to load todos:", error);
    }
  },

  loadPomodoroSession: async () => {
    try {
      if (!isRunningInTauri) {
        console.warn("Skipping loadPomodoroSession - not in Tauri environment");
        return;
      }
      const result = await invoke<{ data: PomodoroSession | null }>(
        "get_pomodoro_session",
      );
      set({ pomodoroSession: result.data });
      console.log("Loaded pomodoro session:", result.data);
    } catch (error) {
      console.error("Failed to load pomodoro session:", error);
    }
  },

  loadUserConfig: async () => {
    try {
      if (!isRunningInTauri) {
        console.warn("Skipping loadUserConfig - not in Tauri environment");
        return;
      }
      const result = await invoke<{ data: UserConfig | null }>(
        "get_user_config",
      );
      set({ userConfig: result.data });
      console.log("Loaded user config:", result.data);
    } catch (error) {
      console.error("Failed to load user config:", error);
    }
  },

  startPomodoro: async () => {
    try {
      console.log("🍅 Starting pomodoro...");
      await invoke("start_pomodoro");
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
      await invoke("pause_pomodoro");
      console.log("✅ Pomodoro paused");
      await get().loadPomodoroSession();
    } catch (error) {
      console.error("❌ Failed to pause pomodoro:", error);
    }
  },

  resetPomodoro: async () => {
    try {
      console.log("🔄 Resetting pomodoro...");
      await invoke("reset_pomodoro");
      console.log("✅ Pomodoro reset");
      await get().loadPomodoroSession();
    } catch (error) {
      console.error("❌ Failed to reset pomodoro:", error);
    }
  },

  skipPomodoroPhase: async () => {
    try {
      await invoke("skip_pomodoro_phase");
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

      const result = await invoke<{
        success: boolean;
        data?: Todo;
        error?: string;
      }>("create_todo", payload);

      if (result.success && result.data) {
        // 成功后替换临时待办事项为真实数据，强制使用前端的状态
        set((state) => {
          const newTodos = state.todos.filter(t => t.id !== optimisticId);
          const finalTodo = {
            ...result.data!,
            status: finalStatus  // 强制使用前端设置的状态
          };
          const updatedTodos = [...newTodos, finalTodo];
          return { todos: updatedTodos };
        });
        get().setSuccess("待办事项创建成功");
      } else {
        // 回滚乐观更新
        set((state) => ({
          todos: state.todos.filter((todo) => todo.id !== optimisticId),
        }));
        throw new Error(result.error || "创建待办事项失败");
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
      await invoke("update_todo", { id, ...updates });
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
      const result = await invoke<{ data: boolean }>("delete_todo", { id });
      if (!result.data) {
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
    const { todos } = get();
    // 保存原始状态用于回滚
    const originalTodos = [...todos];

    // 立即更新UI（乐观更新）
    const updatedTodos = todos.map((todo) => {
      if (todo.id === id) {
        const newStatus: "todo" | "in_progress" | "done" =
          todo.status === "done" ? "todo" : "done";
        return {
          ...todo,
          status: newStatus,
          updated_at: new Date().toISOString(),
        };
      }
      return todo;
    });
    set({ todos: updatedTodos });

    try {
      // 使用真实的后端命令，线程安全问题已修复
      const result = await invoke<{ data: Todo }>("toggle_todo_status", { id });
      if (!result.data) {
        throw new Error("Toggle operation failed");
      }
      // 成功后重新加载列表以确保数据一致性
      await get().loadTodos();
    } catch (error) {
      console.error("Failed to toggle todo status:", error);
      // 失败后回滚
      set({ todos: originalTodos });
      // 显示错误提示
      get().setError("切换待办事项状态失败，请重试");
    }
  },

  saveUserConfig: async (config: UserConfig) => {
    try {
      const result = await invoke<{
        success: boolean;
        data?: {};
        error?: string;
      }>("save_user_config", { config });

      // 检查后端返回的成功状态
      if (!result.success) {
        throw new Error(result.error || "保存配置失败");
      }

      // 更新本地状态
      set({ userConfig: config });

      // 更新运行中的番茄钟服务配置
      try {
        await invoke("update_pomodoro_config", {
          work_duration: config.pomodoro_work_duration,
          short_break: config.pomodoro_short_break_duration,
          long_break: config.pomodoro_long_break_duration,
          cycles: config.pomodoro_cycles_until_long_break,
        });
      } catch (updateError) {
        console.warn("Failed to update pomodoro service config:", updateError);
        // 不阻塞保存流程，只记录警告
      }

      get().setSuccess("设置保存成功");
    } catch (error) {
      console.error("Failed to save user config:", error);
      get().setError(error instanceof Error ? error.message : "保存配置失败");
      throw error; // 重新抛出错误让调用者处理
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
      if (!isRunningInTauri) {
        console.warn("Skipping loadTags - not in Tauri environment");
        return;
      }
      const result = await invoke<{ data: Tag[] }>("get_tags");
      if (result.data) {
        set({ tags: result.data });
        console.log("Loaded tags:", result.data.length);
      }
    } catch (error) {
      console.error("Failed to load tags:", error);
    }
  },

  createTag: async (name: string, color: string) => {
    try {
      if (!isRunningInTauri) {
        console.warn("Skipping createTag - not in Tauri environment");
        return;
      }
      const result = await invoke<{ success: boolean; data: [string, string, string]; error?: string }>("create_tag", {
        name,
        color,
      });
      if (result.success && result.data) {
        const newTag: Tag = {
          id: result.data[0],
          name: result.data[1],
          color: result.data[2],
          created_at: new Date().toISOString(),
        };
        set({ tags: [...get().tags, newTag] });
        get().setSuccess("标签创建成功");
      } else {
        throw new Error(result.error || "创建标签失败");
      }
    } catch (error) {
      console.error("Failed to create tag:", error);
      get().setError(error instanceof Error ? error.message : "创建标签失败");
      throw error;
    }
  },

  deleteTag: async (id: string) => {
    try {
      if (!isRunningInTauri) {
        console.warn("Skipping deleteTag - not in Tauri environment");
        return;
      }
      const result = await invoke<{ success: boolean; data?: boolean; error?: string }>("delete_tag", { id });
      if (result.success && result.data) {
        set({ tags: get().tags.filter((t) => t.id !== id) });
        get().setSuccess("标签删除成功");
      } else {
        throw new Error(result.error || "删除标签失败");
      }
    } catch (error) {
      console.error("Failed to delete tag:", error);
      get().setError(error instanceof Error ? error.message : "删除标签失败");
      throw error;
    }
  },

  assignTagToTodo: async (todoId: string, tagId: string) => {
    try {
      if (!isRunningInTauri) {
        console.warn("Skipping assignTagToTodo - not in Tauri environment");
        return;
      }
      await invoke("assign_tag_to_todo", { todoId, tagId });
      get().setSuccess("标签分配成功");
    } catch (error) {
      console.error("Failed to assign tag to todo:", error);
      get().setError(error instanceof Error ? error.message : "分配标签失败");
      throw error;
    }
  },

  removeTagFromTodo: async (todoId: string, tagId: string) => {
    try {
      if (!isRunningInTauri) {
        console.warn("Skipping removeTagFromTodo - not in Tauri environment");
        return;
      }
      await invoke("remove_tag_from_todo", { todoId, tagId });
      get().setSuccess("标签移除成功");
    } catch (error) {
      console.error("Failed to remove tag from todo:", error);
      get().setError(error instanceof Error ? error.message : "移除标签失败");
      throw error;
    }
  },

  getTodoTags: async (todoId: string) => {
    try {
      if (!isRunningInTauri) {
        console.warn("Skipping getTodoTags - not in Tauri environment");
        return [];
      }
      const result = await invoke<{ success: boolean; data?: Tag[]; error?: string }>("get_todo_tags", { todoId });
      if (result.success && result.data) {
        return result.data;
      }
      return [];
    } catch (error) {
      console.error("Failed to get todo tags:", error);
      return [];
    }
  },
}));

