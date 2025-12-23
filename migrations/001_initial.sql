-- PomodoroFlow-Rs 数据库初始化脚本
-- 版本: 0.1.0
-- 描述: 创建应用所需的所有表、索引和触发器

-- ============================================================================
-- 用户配置表
-- 存储用户的 GitHub 配置、番茄钟设置和界面偏好
-- ============================================================================
CREATE TABLE user_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    github_token_encrypted TEXT NOT NULL,
    github_username TEXT NOT NULL,
    selected_project_owner TEXT,
    selected_project_repo TEXT,
    selected_project_number INTEGER,
    pomodoro_work_duration INTEGER DEFAULT 1500,
    pomodoro_short_break_duration INTEGER DEFAULT 300,
    pomodoro_long_break_duration INTEGER DEFAULT 900,
    pomodoro_cycles_until_long_break INTEGER DEFAULT 4,
    notifications_enabled BOOLEAN DEFAULT 1,
    sound_enabled BOOLEAN DEFAULT 1,
    system_notifications BOOLEAN DEFAULT 1,
    theme TEXT DEFAULT 'light',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 待办事项表
-- 存储用户的任务信息
-- ============================================================================
CREATE TABLE todos (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL CHECK (length(title) > 0 AND length(title) <= 200),
    description TEXT CHECK (description IS NULL OR length(description) <= 5000),
    status TEXT NOT NULL CHECK (status IN ('todo', 'in_progress', 'done')) DEFAULT 'todo',
    github_issue_id INTEGER,
    github_project_id INTEGER,
    github_issue_number INTEGER,
    sync_pending BOOLEAN DEFAULT 1,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP NULL
);

-- ============================================================================
-- 番茄钟会话记录表
-- 存储历史番茄钟会话数据，用于统计和分析
-- ============================================================================
CREATE TABLE pomodoro_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    phase TEXT NOT NULL CHECK (phase IN ('work', 'short_break', 'long_break')),
    duration_seconds INTEGER NOT NULL CHECK (duration_seconds > 0),
    completed_at TIMESTAMP NOT NULL,
    cycle_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 同步队列表
-- 存储离线操作，网络恢复后批量处理
-- ============================================================================
CREATE TABLE sync_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type TEXT NOT NULL CHECK (operation_type IN ('create', 'update', 'delete')),
    table_name TEXT NOT NULL,
    record_id TEXT NOT NULL,
    payload TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'synced', 'failed', 'skipped')),
    retry_count INTEGER DEFAULT 0,
    error_message TEXT,
    priority INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    synced_at TIMESTAMP NULL
);

-- ============================================================================
-- 网络状态记录表
-- 记录网络连接历史，用于分析离线时间
-- ============================================================================
CREATE TABLE network_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    is_online BOOLEAN NOT NULL,
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    duration_seconds INTEGER
);

-- ============================================================================
-- 应用日志表
-- 存储关键操作和错误日志（简化版）
-- ============================================================================
CREATE TABLE app_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    level TEXT NOT NULL CHECK (level IN ('debug', 'info', 'warn', 'error')),
    module TEXT NOT NULL,
    message TEXT NOT NULL,
    context TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- 索引优化
-- ============================================================================

-- 用户配置相关索引
CREATE INDEX idx_user_config_updated ON user_config(updated_at);

-- 待办事项查询优化索引
CREATE INDEX idx_todos_status ON todos(status);
CREATE INDEX idx_todos_sync_pending ON todos(sync_pending);
CREATE INDEX idx_todos_created_at ON todos(created_at);
CREATE INDEX idx_todos_updated_at ON todos(updated_at);
CREATE INDEX idx_todos_deleted_at ON todos(deleted_at);
CREATE INDEX idx_todos_github_issue ON todos(github_issue_id);

-- 番茄钟会话查询索引
CREATE INDEX idx_pomodoro_sessions_date ON pomodoro_sessions(completed_at);
CREATE INDEX idx_pomodoro_sessions_phase ON pomodoro_sessions(phase);

-- 同步队列处理索引
CREATE INDEX idx_sync_queue_status ON sync_queue(status);
CREATE INDEX idx_sync_queue_created_at ON sync_queue(created_at);
CREATE INDEX idx_sync_queue_priority ON sync_queue(priority DESC, created_at);

-- 网络历史分析索引
CREATE INDEX idx_network_history_date ON network_history(recorded_at);

-- 应用日志查询索引
CREATE INDEX idx_app_logs_level ON app_logs(level);
CREATE INDEX idx_app_logs_module ON app_logs(module);
CREATE INDEX idx_app_logs_created_at ON app_logs(created_at);

-- ============================================================================
-- 视图创建
-- ============================================================================

-- 活跃待办事项视图（已删除的不显示）
CREATE VIEW todos_active AS
SELECT *
FROM todos
WHERE deleted_at IS NULL;

-- 待同步任务视图
CREATE VIEW todos_pending_sync AS
SELECT *
FROM todos
WHERE sync_pending = 1
  AND deleted_at IS NULL;

-- 今日番茄钟会话视图
CREATE VIEW pomodoro_sessions_today AS
SELECT *
FROM pomodoro_sessions
WHERE DATE(completed_at) = DATE('now');

-- ============================================================================
-- 触发器
-- ============================================================================

-- 待办事项更新时自动更新 updated_at 字段
CREATE TRIGGER update_todos_updated_at
AFTER UPDATE ON todos
FOR EACH ROW
WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE todos
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

-- 用户配置更新时自动更新 updated_at 字段
CREATE TRIGGER update_user_config_updated_at
AFTER UPDATE ON user_config
FOR EACH ROW
WHEN NEW.id = 1 AND NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE user_config
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = 1;
END;

-- 删除待办事项时设置 deleted_at 字段
-- NOTE: 触发器中的 INSERT 语句会导致 execute_batch() 错误，暂时禁用
-- CREATE TRIGGER soft_delete_todos
-- AFTER UPDATE ON todos
-- FOR EACH ROW
-- WHEN NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL
-- BEGIN
--     INSERT INTO sync_queue (operation_type, table_name, record_id, payload)
--     VALUES ('delete', 'todos', NEW.id, json_object('id', NEW.id, 'deleted_at', NEW.deleted_at));
-- END;

-- 同步队列入队时自动设置优先级
-- CREATE TRIGGER set_sync_queue_priority
-- AFTER INSERT ON sync_queue
-- FOR EACH ROW
-- WHEN NEW.priority = 0
-- BEGIN
--     UPDATE sync_queue SET priority = CASE
--         WHEN NEW.operation_type = 'delete' THEN 10
--         WHEN NEW.operation_type = 'update' THEN 5
--         ELSE 1
--     END WHERE id = NEW.id;
-- END;

-- ============================================================================
-- 初始化数据
-- ============================================================================

-- 插入默认用户配置
INSERT INTO user_config (
    id,
    github_token_encrypted,
    github_username,
    pomodoro_work_duration,
    pomodoro_short_break_duration,
    pomodoro_long_break_duration,
    pomodoro_cycles_until_long_break,
    notifications_enabled,
    sound_enabled,
    system_notifications,
    theme
) VALUES (
    1,
    '',
    '',
    1500,
    300,
    900,
    4,
    1,
    1,
    1,
    'light'
);

-- ============================================================================
-- 注释说明
-- ============================================================================

/*
表结构说明:

1. user_config
   - 应用级配置，只有一条记录 (id = 1)
   - github_token_encrypted: 加密存储的 GitHub PAT
   - 项目选择: owner + repo + number 三字段组合

2. todos
   - 任务主表，使用 UUID 作为主键
   - status: todo(待办), in_progress(进行中), done(已完成)
   - sync_pending: 是否需要同步到 GitHub
   - github_*: GitHub 相关字段，方便快速查询

3. pomodoro_sessions
   - 番茄钟会话历史，用于统计和分析
   - phase: work(工作), short_break(短休息), long_break(长休息)
   - cycle_count: 完成的番茄钟周期数

4. sync_queue
   - 离线操作队列
   - priority: 优先级，数字越大优先级越高
   - status: pending(待处理), synced(已同步), failed(失败), skipped(跳过)

5. network_history
   - 网络状态历史，记录在线/离线时间

6. app_logs
   - 简化版日志表，避免使用外部日志库

索引说明:
- 为常用查询字段添加索引
- 复合索引用于多条件查询
- 按时间倒序索引用于最新数据查询

触发器说明:
- 自动更新 updated_at 字段
- 软删除机制
- 自动设置同步队列优先级

视图说明:
- todos_active: 排除已删除的任务
- todos_pending_sync: 需要同步的任务
- pomodoro_sessions_today: 今日会话记录
*/
