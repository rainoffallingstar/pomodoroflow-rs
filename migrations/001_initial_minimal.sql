-- 最小化迁移文件，只包含表结构，不包含 INSERT 语句

-- 用户配置表
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

-- 待办事项表
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

-- 番茄钟会话记录表
CREATE TABLE pomodoro_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    phase TEXT NOT NULL CHECK (phase IN ('work', 'short_break', 'long_break')),
    duration_seconds INTEGER NOT NULL CHECK (duration_seconds > 0),
    completed_at TIMESTAMP NOT NULL,
    cycle_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 同步队列表
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

-- 网络状态记录表
CREATE TABLE network_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    is_online BOOLEAN NOT NULL,
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    duration_seconds INTEGER
);

-- 应用日志表
CREATE TABLE app_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    level TEXT NOT NULL CHECK (level IN ('debug', 'info', 'warn', 'error')),
    module TEXT NOT NULL,
    message TEXT NOT NULL,
    context TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_user_config_updated ON user_config(updated_at);
CREATE INDEX idx_todos_status ON todos(status);
CREATE INDEX idx_todos_sync_pending ON todos(sync_pending);
CREATE INDEX idx_todos_created_at ON todos(created_at);
CREATE INDEX idx_todos_updated_at ON todos(updated_at);
CREATE INDEX idx_todos_deleted_at ON todos(deleted_at);
CREATE INDEX idx_todos_github_issue ON todos(github_issue_id);
CREATE INDEX idx_pomodoro_sessions_date ON pomodoro_sessions(completed_at);
CREATE INDEX idx_pomodoro_sessions_phase ON pomodoro_sessions(phase);
CREATE INDEX idx_sync_queue_status ON sync_queue(status);
CREATE INDEX idx_sync_queue_created_at ON sync_queue(created_at);
CREATE INDEX idx_sync_queue_priority ON sync_queue(priority DESC, created_at);
CREATE INDEX idx_network_history_date ON network_history(recorded_at);
CREATE INDEX idx_app_logs_level ON app_logs(level);
CREATE INDEX idx_app_logs_module ON app_logs(module);
CREATE INDEX idx_app_logs_created_at ON app_logs(created_at);

-- NOTE: 视图和触发器暂时移除，避免 execute_batch() 中的 SELECT/UPDATE 语句导致错误
-- 将在后续版本中重新添加
