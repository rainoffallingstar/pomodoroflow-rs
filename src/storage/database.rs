//! SQLite 数据库操作

use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::core::error::{AppError, Result};
use crate::core::pomodoro::PomodoroPhase;
use crate::core::state::UserConfig;
use crate::core::todo::{NewTodo, Todo, TodoStatus, TodoUpdate};

/// Helper function to convert TodoStatus to database string
fn todo_status_to_db_string(status: &TodoStatus) -> &str {
    match status {
        TodoStatus::Todo => "todo",
        TodoStatus::InProgress => "in_progress",
        TodoStatus::Done => "done",
    }
}

/// 线程安全的数据库连接包装器
#[derive(Debug, Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

// 安全实现 Send + Sync，因为所有内部字段都是 Arc 包装的
unsafe impl Send for Database {}
unsafe impl Sync for Database {}

impl Database {
    /// 初始化数据库
    pub async fn init(path: &Path) -> Result<Self> {
        // 创建目录（如果不存在）
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::Io(e))?;
        }

        let conn = Connection::open(path).map_err(AppError::Database)?;

        // 简化初始化，避免复杂的 PRAGMA 和迁移
        // 只设置基本的 PRAGMA
        let _ = conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON;",
        );

        // 运行迁移
        Self::run_migrations(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 创建未初始化的数据库实例
    pub fn init_uninitialized() -> Self {
        // 创建一个内存数据库连接作为占位符
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");

        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    /// 安全地获取数据库连接锁
    fn get_conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("Failed to lock database")
    }

    /// 运行数据库迁移
    fn run_migrations(conn: &Connection) -> Result<()> {
        // 首先创建版本表
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#,
        )
        .map_err(AppError::Database)?;

        // 获取当前版本
        let current_version: i32 = conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        const CURRENT_SCHEMA_VERSION: i32 = 2;

        // 如果已经是最新版本，跳过迁移
        if current_version >= CURRENT_SCHEMA_VERSION {
            println!("✅ 数据库已是最新版本 (v{})，跳过迁移", current_version);
            return Ok(());
        }

        println!(
            "🔧 运行数据库迁移: v{} -> v{}",
            current_version, CURRENT_SCHEMA_VERSION
        );

        // 版本1：初始表结构
        if current_version < 1 {
            // 创建用户配置表
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS user_config (
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
                )
            "#,
            )
            .map_err(AppError::Database)?;

            // 创建待办事项表
            conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS todos (
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
            )
        "#).map_err(AppError::Database)?;

            // 创建番茄钟会话记录表
            conn.execute_batch(
                r#"
            CREATE TABLE IF NOT EXISTS pomodoro_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                phase TEXT NOT NULL CHECK (phase IN ('work', 'short_break', 'long_break')),
                duration_seconds INTEGER NOT NULL CHECK (duration_seconds > 0),
                completed_at TIMESTAMP NOT NULL,
                cycle_count INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#,
            )
            .map_err(AppError::Database)?;

            // 创建同步队列表
            conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS sync_queue (
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
            )
        "#).map_err(AppError::Database)?;

            // 创建网络状态记录表
            conn.execute_batch(
                r#"
            CREATE TABLE IF NOT EXISTS network_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                is_online BOOLEAN NOT NULL,
                recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                duration_seconds INTEGER
            )
        "#,
            )
            .map_err(AppError::Database)?;

            // 创建应用日志表
            conn.execute_batch(
                r#"
            CREATE TABLE IF NOT EXISTS app_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                level TEXT NOT NULL CHECK (level IN ('debug', 'info', 'warn', 'error')),
                module TEXT NOT NULL,
                message TEXT NOT NULL,
                context TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#,
            )
            .map_err(AppError::Database)?;

            // 手动插入默认用户配置
            conn.execute(
                "INSERT OR IGNORE INTO user_config (
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
                )",
                [],
            )
            .ok(); // 忽略错误

            // 更新版本号
            conn.execute(
                "INSERT OR REPLACE INTO schema_version (version) VALUES (1)",
                [],
            )
            .map_err(AppError::Database)?;

            println!("✅ 数据库迁移到版本1完成");
        }

        // 版本2：添加标签功能
        if current_version < 2 {
            // 创建标签表
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS tags (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,
                    color TEXT NOT NULL DEFAULT '#007AFF',
                    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
                )
            "#,
            )
            .map_err(AppError::Database)?;

            // 创建待办事项-标签关联表
            conn.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS todo_tags (
                    todo_id TEXT NOT NULL,
                    tag_id TEXT NOT NULL,
                    PRIMARY KEY (todo_id, tag_id),
                    FOREIGN KEY (todo_id) REFERENCES todos(id) ON DELETE CASCADE,
                    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
                )
            "#,
            )
            .map_err(AppError::Database)?;

            // 更新版本号
            conn.execute(
                "INSERT OR REPLACE INTO schema_version (version) VALUES (2)",
                [],
            )
            .map_err(AppError::Database)?;

            println!("✅ 数据库迁移到版本2完成（标签功能）");
        }

        Ok(())
    }

    // ========================================================================
    // 用户配置操作
    // ========================================================================

    /// 保存用户配置
    pub async fn save_user_config(&self, config: &UserConfig) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        let config = config.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;
            let tx = conn.transaction().map_err(AppError::Database)?;

            tx.execute(
                r#"
                INSERT OR REPLACE INTO user_config (
                    id, github_token_encrypted, github_username,
                    selected_project_owner, selected_project_repo, selected_project_number,
                    pomodoro_work_duration, pomodoro_short_break_duration,
                    pomodoro_long_break_duration, pomodoro_cycles_until_long_break,
                    notifications_enabled, sound_enabled, system_notifications, theme
                ) VALUES (
                    1, '', '', NULL, NULL, NULL,
                    ?1, ?2, ?3, ?4,
                    ?5, ?6, 0, ?7
                )
                "#,
                params![
                    config.pomodoro_work_duration,
                    config.pomodoro_short_break_duration,
                    config.pomodoro_long_break_duration,
                    config.pomodoro_cycles_until_long_break,
                    config.notifications_enabled as i32,
                    config.sound_enabled as i32,
                    &config.theme,
                ],
            )
            .map_err(AppError::Database)?;

            tx.commit().map_err(AppError::Database)?;

            Ok(())
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 加载用户配置
    pub async fn load_user_config(&self) -> Result<Option<UserConfig>> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

            let mut stmt = conn
                .prepare(
                    r#"
                SELECT
                    pomodoro_work_duration, pomodoro_short_break_duration,
                    pomodoro_long_break_duration, pomodoro_cycles_until_long_break,
                    notifications_enabled, sound_enabled, theme
                FROM user_config WHERE id = 1
                "#,
                )
                .map_err(AppError::Database)?;

            let config_iter = stmt
                .query_map([], |row| {
                    Ok(UserConfig {
                        pomodoro_work_duration: row.get("pomodoro_work_duration")?,
                        pomodoro_short_break_duration: row.get("pomodoro_short_break_duration")?,
                        pomodoro_long_break_duration: row.get("pomodoro_long_break_duration")?,
                        pomodoro_cycles_until_long_break: row
                            .get("pomodoro_cycles_until_long_break")?,
                        notifications_enabled: row.get("notifications_enabled")?,
                        sound_enabled: row.get("sound_enabled")?,
                        theme: row.get("theme")?,
                    })
                })
                .map_err(AppError::Database)?;

            for config_result in config_iter {
                return Ok(Some(config_result.map_err(AppError::Database)?));
            }

            Ok(None)
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    // ========================================================================
    // 任务操作
    // ========================================================================

    /// 创建新任务
    pub async fn create_todo(&self, new_todo: &NewTodo) -> Result<Todo> {
        let conn = Arc::clone(&self.conn);
        let new_todo = new_todo.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;
            let tx = conn.transaction().map_err(AppError::Database)?;

            let id = uuid::Uuid::new_v4().to_string();
            let now = Utc::now();

            tx.execute(
                r#"
                INSERT INTO todos (id, title, description, status, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                params![id, new_todo.title, new_todo.description, todo_status_to_db_string(&new_todo.status), now, now],
            )
            .map_err(AppError::Database)?;

            tx.commit().map_err(AppError::Database)?;

            // 返回创建的任务
            Ok(Todo {
                id,
                title: new_todo.title.clone(),
                description: new_todo.description.clone(),
                status: new_todo.status.clone(),
                created_at: now,
                updated_at: now,
            })
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 获取所有任务
    pub async fn get_all_todos(&self) -> Result<Vec<Todo>> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

            let mut stmt = conn
                .prepare("SELECT * FROM todos ORDER BY created_at DESC")
                .map_err(AppError::Database)?;

            let todo_iter = stmt
                .query_map([], |row| {
                    let status_str: String = row.get("status")?;
                    let status = match status_str.as_str() {
                        "todo" => TodoStatus::Todo,
                        "in_progress" => TodoStatus::InProgress,
                        "done" => TodoStatus::Done,
                        _ => TodoStatus::Todo,
                    };

                    Ok(Todo {
                        id: row.get("id")?,
                        title: row.get("title")?,
                        description: row.get("description")?,
                        status,
                        created_at: row.get("created_at")?,
                        updated_at: row.get("updated_at")?,
                    })
                })
                .map_err(AppError::Database)?;

            let mut todos = Vec::new();
            for todo_result in todo_iter {
                todos.push(todo_result.map_err(AppError::Database)?);
            }

            Ok(todos)
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 根据ID获取任务
    pub async fn get_todo_by_id(&self, id: &str) -> Result<Option<Todo>> {
        let conn = Arc::clone(&self.conn);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

            let mut stmt = conn
                .prepare("SELECT * FROM todos WHERE id = ?1")
                .map_err(AppError::Database)?;

            let mut rows = stmt.query(params![id]).map_err(AppError::Database)?;

            if let Some(row) = rows.next().map_err(AppError::Database)? {
                let status_str: String = row.get("status")?;
                let status = match status_str.as_str() {
                    "todo" => TodoStatus::Todo,
                    "in_progress" => TodoStatus::InProgress,
                    "done" => TodoStatus::Done,
                    _ => TodoStatus::Todo,
                };

                return Ok(Some(Todo {
                    id: row.get("id")?,
                    title: row.get("title")?,
                    description: row.get("description")?,
                    status,
                    created_at: row.get("created_at")?,
                    updated_at: row.get("updated_at")?,
                }));
            }

            Ok(None)
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 更新任务
    pub async fn update_todo(&self, id: &str, updates: &TodoUpdate) -> Result<Option<Todo>> {
        let conn = Arc::clone(&self.conn);
        let id = id.to_string();
        let updates = updates.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;
            let tx = conn.transaction().map_err(AppError::Database)?;

            // 分别更新每个字段
            if let Some(ref title) = updates.title {
                tx.execute(
                    "UPDATE todos SET title = ?, updated_at = ? WHERE id = ?",
                    params![title, Utc::now(), id],
                )
                .map_err(AppError::Database)?;
            }

            if let Some(ref description) = updates.description {
                // 处理 Option<Option<String>>
                let desc_value = if let Some(ref desc) = *description {
                    desc.as_str()
                } else {
                    ""
                };
                tx.execute(
                    "UPDATE todos SET description = ?, updated_at = ? WHERE id = ?",
                    params![desc_value, Utc::now(), id],
                )
                .map_err(AppError::Database)?;
            }

            if let Some(ref status) = updates.status {
                tx.execute(
                    "UPDATE todos SET status = ?, updated_at = ? WHERE id = ?",
                    params![status.to_string(), Utc::now(), id],
                )
                .map_err(AppError::Database)?;
            }

            tx.commit().map_err(AppError::Database)?;

            // 在同一个闭包内获取更新后的任务
            let mut stmt = conn
                .prepare("SELECT * FROM todos WHERE id = ?1")
                .map_err(AppError::Database)?;

            let mut rows = stmt.query(params![id]).map_err(AppError::Database)?;

            if let Some(row) = rows.next().map_err(AppError::Database)? {
                let status_str: String = row.get("status")?;
                let status = match status_str.as_str() {
                    "todo" => TodoStatus::Todo,
                    "in_progress" => TodoStatus::InProgress,
                    "done" => TodoStatus::Done,
                    _ => TodoStatus::Todo,
                };

                return Ok(Some(Todo {
                    id: row.get("id")?,
                    title: row.get("title")?,
                    description: row.get("description")?,
                    status,
                    created_at: row.get("created_at")?,
                    updated_at: row.get("updated_at")?,
                }));
            }

            Ok(None)
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 删除任务
    pub async fn delete_todo(&self, id: &str) -> Result<bool> {
        let conn = Arc::clone(&self.conn);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let mut conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;
            let tx = conn.transaction().map_err(AppError::Database)?;

            let rows_affected = tx
                .execute("DELETE FROM todos WHERE id = ?1", params![id])
                .map_err(AppError::Database)?;

            tx.commit().map_err(AppError::Database)?;

            Ok(rows_affected > 0)
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 永久删除任务
    pub async fn permanently_delete_todo(&self, id: &str) -> Result<bool> {
        let conn = Arc::clone(&self.conn);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

            let rows_affected = conn
                .execute("DELETE FROM todos WHERE id = ?1", params![id])
                .map_err(AppError::Database)?;

            Ok(rows_affected > 0)
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 获取需要同步的任务
    pub async fn get_pending_sync_todos(&self) -> Result<Vec<Todo>> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

            let mut stmt = conn
                .prepare("SELECT * FROM todos WHERE sync_pending = 1")
                .map_err(AppError::Database)?;

            let todo_iter = stmt
                .query_map([], |row| {
                    let status_str: String = row.get("status")?;
                    let status = match status_str.as_str() {
                        "todo" => TodoStatus::Todo,
                        "in_progress" => TodoStatus::InProgress,
                        "done" => TodoStatus::Done,
                        _ => TodoStatus::Todo,
                    };

                    Ok(Todo {
                        id: row.get("id")?,
                        title: row.get("title")?,
                        description: row.get("description")?,
                        status,
                        created_at: row.get("created_at")?,
                        updated_at: row.get("updated_at")?,
                    })
                })
                .map_err(AppError::Database)?;

            let mut todos = Vec::new();
            for todo_result in todo_iter {
                todos.push(todo_result.map_err(AppError::Database)?);
            }

            Ok(todos)
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 标记任务为已同步
    pub async fn mark_todo_synced(&self, id: &str) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        let id = id.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

            conn.execute(
                "UPDATE todos SET sync_pending = 0 WHERE id = ?1",
                params![id],
            )
            .map_err(AppError::Database)?;

            Ok(())
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    // ========================================================================
    // 同步队列操作
    // ========================================================================

    /// 添加到同步队列
    pub async fn add_to_sync_queue(
        &self,
        operation_type: &str,
        record_id: &str,
        payload: &serde_json::Value,
    ) -> Result<i64> {
        let conn = self.get_conn();

        conn.execute(
            r#"
            INSERT INTO sync_queue (operation_type, table_name, record_id, payload)
            VALUES (?1, 'todos', ?2, ?3)
            "#,
            params![operation_type, record_id, payload.to_string()],
        )
        .map_err(AppError::Database)?;

        Ok(conn.last_insert_rowid())
    }

    /// 获取待同步队列
    pub async fn get_pending_sync_queue(&self) -> Result<Vec<(i64, String, String, String)>> {
        let conn = self.get_conn();

        let mut stmt = conn.prepare(
            "SELECT id, operation_type, record_id, payload FROM sync_queue WHERE status = 'pending' ORDER BY priority DESC, created_at ASC"
        ).map_err(AppError::Database)?;

        let mut items = Vec::new();
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get("id")?,
                    row.get("operation_type")?,
                    row.get("record_id")?,
                    row.get("payload")?,
                ))
            })
            .map_err(AppError::Database)?;

        for item in rows {
            items.push(item.map_err(AppError::Database)?);
        }

        Ok(items)
    }

    /// 标记同步队列项为已同步
    pub async fn mark_sync_queue_synced(&self, id: i64) -> Result<()> {
        let conn = self.get_conn();

        conn.execute(
            "UPDATE sync_queue SET status = 'synced', synced_at = ?1 WHERE id = ?2",
            params![Utc::now(), id],
        )
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// 标记同步队列项为失败
    pub async fn mark_sync_queue_failed(&self, id: i64, error_message: &str) -> Result<()> {
        let conn = self.get_conn();

        conn.execute(
            "UPDATE sync_queue SET status = 'failed', error_message = ?1 WHERE id = ?2",
            params![error_message, id],
        )
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// 清理已同步的队列项（保留最新的100条）
    pub async fn cleanup_sync_queue(&self) -> Result<usize> {
        let conn = self.get_conn();

        // 获取需要清理的记录数
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM sync_queue WHERE status = 'synced'")
            .map_err(AppError::Database)?;

        let count: i64 = stmt
            .query_row([], |row| row.get(0))
            .map_err(AppError::Database)?;

        if count <= 100 {
            return Ok(0);
        }

        let to_delete = count - 100;

        conn.execute(
            "DELETE FROM sync_queue WHERE status = 'synced' AND id IN (
                SELECT id FROM sync_queue WHERE status = 'synced' ORDER BY synced_at ASC LIMIT ?
            )",
            params![to_delete],
        )
        .map_err(AppError::Database)?;

        Ok(to_delete as usize)
    }

    // ========================================================================
    // 番茄钟会话操作
    // ========================================================================

    /// 记录番茄钟会话
    pub async fn record_pomodoro_session(
        &self,
        phase: PomodoroPhase,
        duration_seconds: u32,
        cycle_count: u32,
    ) -> Result<()> {
        let conn = self.get_conn();

        let phase_str = match phase {
            PomodoroPhase::Work => "work",
            PomodoroPhase::ShortBreak => "short_break",
            PomodoroPhase::LongBreak => "long_break",
        };

        conn.execute(
            "INSERT INTO pomodoro_sessions (phase, duration_seconds, completed_at, cycle_count) VALUES (?1, ?2, ?3, ?4)",
            params![phase_str, duration_seconds, Utc::now(), cycle_count]
        ).map_err(AppError::Database)?;

        Ok(())
    }

    /// 获取今日番茄钟会话
    pub async fn get_today_pomodoro_sessions(&self) -> Result<Vec<(PomodoroPhase, u32, u32)>> {
        let conn = self.get_conn();

        let mut stmt = conn.prepare(
            "SELECT phase, duration_seconds, cycle_count FROM pomodoro_sessions WHERE DATE(completed_at) = DATE('now') ORDER BY completed_at DESC"
        ).map_err(AppError::Database)?;

        let mut sessions = Vec::new();
        let rows = stmt
            .query_map([], |row| {
                let phase_str: String = row.get("phase")?;
                let phase = match phase_str.as_str() {
                    "work" => PomodoroPhase::Work,
                    "short_break" => PomodoroPhase::ShortBreak,
                    "long_break" => PomodoroPhase::LongBreak,
                    _ => PomodoroPhase::Work,
                };

                Ok((phase, row.get("duration_seconds")?, row.get("cycle_count")?))
            })
            .map_err(AppError::Database)?;

        for session in rows {
            sessions.push(session.map_err(AppError::Database)?);
        }

        Ok(sessions)
    }

    // ========================================================================
    // 标签操作
    // ========================================================================

    /// 创建标签
    pub async fn create_tag(&self, name: &str, color: &str) -> Result<(String, String)> {
        let conn = Arc::clone(&self.conn);
        let name = name.to_string();
        let color = color.to_string();
        
        tokio::task::spawn_blocking(move || {
            let mut conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;
            
            let id = uuid::Uuid::new_v4().to_string();
            let now = Utc::now();

            conn.execute(
                "INSERT INTO tags (id, name, color, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![id, name, color, now],
            )
            .map_err(AppError::Database)?;

            Ok((id, name))
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 获取所有标签
    pub async fn get_all_tags(&self) -> Result<Vec<(String, String, String)>> {
        let conn = Arc::clone(&self.conn);
        
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

            let mut stmt = conn
                .prepare("SELECT id, name, color FROM tags ORDER BY created_at DESC")
                .map_err(AppError::Database)?;

            let mut tags = Vec::new();
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get("id")?,
                        row.get("name")?,
                        row.get("color")?,
                    ))
                })
                .map_err(AppError::Database)?;

            for tag_result in rows {
                tags.push(tag_result.map_err(AppError::Database)?);
            }

            Ok(tags)
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 删除标签
    pub async fn delete_tag(&self, id: &str) -> Result<bool> {
        let conn = Arc::clone(&self.conn);
        let id = id.to_string();
        
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

            let rows_affected = conn
                .execute("DELETE FROM tags WHERE id = ?1", params![id])
                .map_err(AppError::Database)?;

            Ok(rows_affected > 0)
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 为待办事项添加标签
    pub async fn add_tag_to_todo(&self, todo_id: &str, tag_id: &str) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        let todo_id = todo_id.to_string();
        let tag_id = tag_id.to_string();
        
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;
            
            // 使用 INSERT OR IGNORE 避免重复
            conn.execute(
                "INSERT OR IGNORE INTO todo_tags (todo_id, tag_id) VALUES (?1, ?2)",
                params![todo_id, tag_id],
            )
            .map_err(AppError::Database)?;

            Ok(())
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 从待办事项移除标签
    pub async fn remove_tag_from_todo(&self, todo_id: &str, tag_id: &str) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        let todo_id = todo_id.to_string();
        let tag_id = tag_id.to_string();
        
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

            conn.execute(
                "DELETE FROM todo_tags WHERE todo_id = ?1 AND tag_id = ?2",
                params![todo_id, tag_id],
            )
            .map_err(AppError::Database)?;

            Ok(())
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }

    /// 获取待办事项的所有标签
    pub async fn get_todo_tags(&self, todo_id: &str) -> Result<Vec<(String, String, String)>> {
        let conn = Arc::clone(&self.conn);
        let todo_id = todo_id.to_string();
        
        tokio::task::spawn_blocking(move || {
            let conn = conn
                .lock()
                .map_err(|e| AppError::Other(format!("Failed to lock database: {}", e)))?;

            let mut stmt = conn
                .prepare(
                    "SELECT t.id, t.name, t.color FROM tags t 
                     INNER JOIN todo_tags tt ON t.id = tt.tag_id 
                     WHERE tt.todo_id = ?1 
                     ORDER BY t.created_at DESC"
                )
                .map_err(AppError::Database)?;

            let mut tags = Vec::new();
            let rows = stmt
                .query_map(params![todo_id], |row| {
                    Ok((
                        row.get("id")?,
                        row.get("name")?,
                        row.get("color")?,
                    ))
                })
                .map_err(AppError::Database)?;

            for tag_result in rows {
                tags.push(tag_result.map_err(AppError::Database)?);
            }

            Ok(tags)
        })
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
    }
}
