import { useAppStore, Todo } from "../stores/appStore";
import { useState, useMemo, useEffect } from "react";
import "../styles/TodoList.css";

type SortType = "created_at" | "title" | "status";
type TabType = "todo" | "in_progress" | "done";

/**
 * iOS 18 分组列表风格的 TodoList 组件
 * - 三标签页：待办 / 进行中 / 已完成
 * - inset grouped 样式
 * - iOS 原生复选框
 * - 标签支持
 */
export function TodoList() {
  const {
    todos,
    selectedTodoId,
    createTodo,
    updateTodo,
    deleteTodo,
    toggleTodoStatus,
    selectTodo,
    tags,
    loadTags,
    createTag,
    deleteTag,
    assignTagToTodo,
    removeTagFromTodo,
  } = useAppStore();

  const [newTodoTitle, setNewTodoTitle] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editTitle, setEditTitle] = useState("");
  const [sortBy, setSortBy] = useState<SortType>("created_at");
  const [sortAsc, setSortAsc] = useState(false);
  const [selectedTag, setSelectedTag] = useState<string>("");
  const [activeTab, setActiveTab] = useState<TabType>("todo");

  // 加载标签
  useEffect(() => {
    loadTags();
  }, [loadTags]);

  // 根据标签页过滤任务
  const filteredAndSortedTodos = useMemo(() => {
    let filtered = [...todos];

    // 按标签页过滤
    filtered = filtered.filter((todo) => {
      switch (activeTab) {
        case "todo":
          return todo.status === "todo";
        case "in_progress":
          return todo.status === "in_progress";
        case "done":
          return todo.status === "done";
        default:
          return true;
      }
    });

    // 排序
    filtered.sort((a, b) => {
      let aValue, bValue;

      switch (sortBy) {
        case "title":
          aValue = a.title.toLowerCase();
          bValue = b.title.toLowerCase();
          break;
        case "status":
          aValue = a.status;
          bValue = b.status;
          break;
        case "created_at":
        default:
          aValue = new Date(a.created_at).getTime();
          bValue = new Date(b.created_at).getTime();
          break;
      }

      if (aValue < bValue) return sortAsc ? -1 : 1;
      if (aValue > bValue) return sortAsc ? 1 : -1;
      return 0;
    });

    return filtered;
  }, [todos, activeTab, sortBy, sortAsc]);

  // 统计信息
  const stats = useMemo(() => {
    const total = todos.length;
    const todo = todos.filter((t) => t.status === "todo").length;
    const inProgress = todos.filter((t) => t.status === "in_progress").length;
    const done = todos.filter((t) => t.status === "done").length;

    return { total, todo, inProgress, done };
  }, [todos]);

  const handleCreate = async () => {
    if (newTodoTitle.trim()) {
      // 将当前活动标签页作为初始状态传递
      await createTodo(newTodoTitle.trim(), undefined, activeTab);
      // 如果选择了标签，为新任务分配标签
      if (selectedTag) {
        await assignTagToTodo(newTodoTitle.trim(), selectedTag);
        setSelectedTag(""); // 重置标签选择
      }
      setNewTodoTitle("");
    }
  };

  const handleEdit = (todo: Todo, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingId(todo.id);
    setEditTitle(todo.title);
  };

  const handleSave = async (id: string) => {
    if (editTitle.trim()) {
      await updateTodo(id, {
        title: editTitle.trim(),
      });
    }
    setEditingId(null);
    setEditTitle("");
  };

  const handleCancel = () => {
    setEditingId(null);
    setEditTitle("");
  };

  const handleDelete = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    deleteTodo(id);
  };

  const handleToggleStatus = async (
    id: string,
    currentStatus: string,
    e: React.MouseEvent,
  ) => {
    e.stopPropagation();
    let nextStatus: "todo" | "in_progress" | "done";

    switch (currentStatus) {
      case "todo":
        nextStatus = "in_progress";
        break;
      case "in_progress":
        nextStatus = "done";
        break;
      case "done":
        nextStatus = "todo";
        break;
      default:
        nextStatus = "todo";
    }

    await updateTodo(id, { status: nextStatus });
  };

  const handleSort = (type: SortType) => {
    if (sortBy === type) {
      setSortAsc(!sortAsc);
    } else {
      setSortBy(type);
      setSortAsc(false);
    }
  };

  const isSelected = (todoId: string) => selectedTodoId === todoId;
  const isCompleted = (status: string) => status === "done";

  const getStatusText = (status: string) => {
    switch (status) {
      case "todo":
        return "待办";
      case "in_progress":
        return "进行中";
      case "done":
        return "已完成";
      default:
        return "未知";
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "todo":
        return "";
      case "in_progress":
        return "▶";
      case "done":
        return "✓";
      default:
        return "";
    }
  };

  const getTabClass = (tab: TabType) => {
    return activeTab === tab ? "active" : "";
  };

  return (
    <div className="ios-todo-section">
      {/* iOS 风格三标签页 */}
      <div className="ios-segmented-control">
        <button
          className={`ios-segment ${getTabClass("todo")}`}
          onClick={() => setActiveTab("todo")}
        >
          <span className="tab-count">{stats.todo}</span>
          待办
        </button>
        <button
          className={`ios-segment ${getTabClass("in_progress")}`}
          onClick={() => setActiveTab("in_progress")}
        >
          <span className="tab-count">{stats.inProgress}</span>
          进行中
        </button>
        <button
          className={`ios-segment ${getTabClass("done")}`}
          onClick={() => setActiveTab("done")}
        >
          <span className="tab-count">{stats.done}</span>
          已完成
        </button>
      </div>

      {/* 创建新任务输入框 */}
      {editingId === null && (
        <>
          {/* 标签选择器 */}
          {tags.length > 0 && (
            <div className="tag-selector">
              <select
                className="ios-select"
                value={selectedTag}
                onChange={(e) => setSelectedTag(e.target.value)}
              >
                <option value="">无标签</option>
                {tags.map((tag) => (
                  <option key={tag.id} value={tag.id}>
                    {tag.name}
                  </option>
                ))}
              </select>
            </div>
          )}
          <div className="ios-create-row">
            <input
              type="text"
              value={newTodoTitle}
              onChange={(e) => setNewTodoTitle(e.target.value)}
              placeholder="添加新任务..."
              onKeyPress={(e) => e.key === "Enter" && handleCreate()}
              className="ios-input"
            />
          </div>
        </>
      )}

      {/* iOS 分组列表 - 可滚动区域 */}
      <div className="ios-grouped-list-wrapper">
        <div className="ios-grouped-list">
          {filteredAndSortedTodos.map((todo) => (
            <div
              key={todo.id}
              className={`ios-list-item ${
                isSelected(todo.id) ? "selected" : ""
              } ${isCompleted(todo.status) ? "completed" : ""}`}
              onClick={() => selectTodo(todo.id)}
            >
            {/* iOS 状态指示器 */}
            <div
              className={`ios-status-indicator status-${todo.status}`}
              onClick={(e) => handleToggleStatus(todo.id, todo.status, e)}
              title={getStatusText(todo.status)}
            >
              {getStatusIcon(todo.status)}
            </div>

            {/* 任务内容 */}
            {editingId === todo.id ? (
              <input
                type="text"
                value={editTitle}
                onChange={(e) => setEditTitle(e.target.value)}
                onKeyPress={(e) => e.key === "Enter" && handleSave(todo.id)}
                onBlur={() => handleSave(todo.id)}
                autoFocus
                className="ios-input ios-edit-input"
                onClick={(e) => e.stopPropagation()}
              />
            ) : (
              <div className="ios-todo-content">
                <span className="ios-todo-title">{todo.title}</span>

                {/* 显示任务标签 */}
                {todo.tags && todo.tags.length > 0 && (
                  <div className="ios-todo-tags">
                    {todo.tags.map((tag) => (
                      <span
                        key={tag.id}
                        className="todo-tag"
                        style={{ backgroundColor: tag.color }}
                      >
                        {tag.name}
                      </span>
                    ))}
                  </div>
                )}
              </div>
            )}

            {/* 操作按钮 */}
            {editingId !== todo.id && (
              <div className="ios-todo-actions">
                <button
                  className="ios-action-btn"
                  onClick={(e) => handleEdit(todo, e)}
                  title="编辑"
                >
                  ✏️
                </button>
                <button
                  className="ios-action-btn ios-delete-btn"
                  onClick={(e) => handleDelete(todo.id, e)}
                  title="删除"
                >
                  🗑️
                </button>
              </div>
            )}
          </div>
        ))}
        </div>

        {/* 空状态 */}
        {filteredAndSortedTodos.length === 0 && (
          <div className="ios-empty-state">
            <div className="ios-empty-icon">📝</div>
            <p className="ios-empty-title">暂无任务</p>
            <p className="ios-empty-description">
              添加第一个任务开始你的工作吧！
            </p>
          </div>
        )}
      </div>

      {/* iOS 浮动添加按钮 */}
      {editingId === null && newTodoTitle.trim() && (
        <button className="ios-add-btn" onClick={handleCreate}>
          +
        </button>
      )}
    </div>
  );
}
