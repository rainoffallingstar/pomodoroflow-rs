//! 待办事项数据模型和服务

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::error::{AppError, Result};

/// 任务状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Todo,
    InProgress,
    Done,
}

impl TodoStatus {
    /// 转换为人类可读的字符串
    pub fn to_string(&self) -> &'static str {
        match self {
            TodoStatus::Todo => "待办",
            TodoStatus::InProgress => "进行中",
            TodoStatus::Done => "已完成",
        }
    }

    /// 从字符串解析状态
    pub fn from_string(s: &str) -> Result<Self> {
        match s {
            "todo" => Ok(TodoStatus::Todo),
            "in_progress" => Ok(TodoStatus::InProgress),
            "done" => Ok(TodoStatus::Done),
            _ => Err(AppError::Validation(format!("未知的状态: {}", s))),
        }
    }
}

/// 标签实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
}

impl Tag {
    /// 创建新标签
    pub fn new(name: String, color: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            color,
            created_at: Utc::now(),
        }
    }
}

/// 任务实体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TodoStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Todo {
    /// 创建新的任务
    pub fn new(title: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            status: TodoStatus::Todo,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新任务标题
    pub fn update_title(&mut self, title: String) {
        self.title = title;
        self.updated_at = Utc::now();
    }

    /// 更新任务描述
    pub fn update_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    /// 更新任务状态
    pub fn update_status(&mut self, status: TodoStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// 切换任务状态
    pub fn toggle_status(&mut self) {
        self.status = match self.status {
            TodoStatus::Todo => TodoStatus::InProgress,
            TodoStatus::InProgress => TodoStatus::Done,
            TodoStatus::Done => TodoStatus::Todo,
        };
        self.updated_at = Utc::now();
    }

    /// 检查任务是否完成
    pub fn is_done(&self) -> bool {
        matches!(self.status, TodoStatus::Done)
    }
}

/// 创建新任务时的数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTodo {
    pub title: String,
    pub description: Option<String>,
    pub status: TodoStatus,
}

impl Default for NewTodo {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: None,
            status: TodoStatus::Todo,
        }
    }
}

impl NewTodo {
    /// 验证数据有效性
    pub fn validate(&self) -> Result<()> {
        if self.title.trim().is_empty() {
            return Err(AppError::Validation("任务标题不能为空".to_string()));
        }

        if self.title.len() > 200 {
            return Err(AppError::Validation(
                "任务标题不能超过 200 字符".to_string(),
            ));
        }

        if let Some(desc) = &self.description {
            if desc.len() > 5000 {
                return Err(AppError::Validation(
                    "任务描述不能超过 5000 字符".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// 更新任务时的数据结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TodoUpdate {
    pub title: Option<String>,
    pub description: Option<Option<String>>, // Some(None) 表示删除描述
    pub status: Option<TodoStatus>,
}

impl TodoUpdate {
    /// 创建空的更新
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置标题
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = Some(description);
        self
    }

    /// 设置状态
    pub fn with_status(mut self, status: TodoStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// 检查是否有任何更新
    pub fn has_updates(&self) -> bool {
        self.title.is_some() || self.description.is_some() || self.status.is_some()
    }
}

/// 任务筛选器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoFilter {
    pub status: Option<TodoStatus>,
    pub search: Option<String>,
    pub show_completed: bool,
    pub limit: Option<usize>,
}

impl Default for TodoFilter {
    fn default() -> Self {
        Self {
            status: None,
            search: None,
            show_completed: true,
            limit: None,
        }
    }
}

impl TodoFilter {
    /// 创建显示全部任务的筛选器
    pub fn all() -> Self {
        Self::default()
    }

    /// 创建只显示待办任务的筛选器
    pub fn pending() -> Self {
        Self {
            status: Some(TodoStatus::Todo),
            show_completed: false,
            ..Default::default()
        }
    }

    /// 创建显示已完成任务的筛选器
    pub fn completed() -> Self {
        Self {
            status: Some(TodoStatus::Done),
            show_completed: true,
            ..Default::default()
        }
    }

    /// 设置搜索关键词
    pub fn with_search(mut self, search: String) -> Self {
        self.search = Some(search);
        self
    }

    /// 设置状态筛选
    pub fn with_status(mut self, status: TodoStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// 设置结果限制
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// 应用筛选器到任务列表
    pub fn apply<'a>(&self, todos: &'a [Todo]) -> Vec<&'a Todo> {
        let mut filtered: Vec<&Todo> = todos.iter().collect();

        // 按状态筛选
        if let Some(status) = &self.status {
            filtered.retain(|t| t.status == *status);
        }

        // 隐藏已完成的任务
        if !self.show_completed {
            filtered.retain(|t| !t.is_done());
        }

        // 按关键词搜索
        if let Some(search) = &self.search {
            let search_lower = search.to_lowercase();
            filtered.retain(|t| {
                t.title.to_lowercase().contains(&search_lower)
                    || t.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&search_lower))
                        .unwrap_or(false)
            });
        }

        // 按更新时间倒序排序
        filtered.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        // 应用限制
        if let Some(limit) = self.limit {
            filtered.truncate(limit);
        }

        filtered
    }
}

/// 待办事项服务
#[derive(Debug)]
pub struct TodoService {
    // 服务会在应用层实现，这里只定义接口
}

/// 任务统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoStats {
    pub total: usize,
    pub todo: usize,
    pub in_progress: usize,
    pub done: usize,
}

impl TodoStats {
    /// 从任务列表计算统计信息
    pub fn from_todos(todos: &[Todo]) -> Self {
        let mut stats = Self {
            total: 0,
            todo: 0,
            in_progress: 0,
            done: 0,
        };

        for todo in todos {
            stats.total += 1;

            match todo.status {
                TodoStatus::Todo => stats.todo += 1,
                TodoStatus::InProgress => stats.in_progress += 1,
                TodoStatus::Done => stats.done += 1,
            }
        }

        stats
    }

    /// 计算完成率
    pub fn completion_rate(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.done as f32 / self.total as f32) * 100.0
        }
    }
}

/// 任务导出格式
#[derive(Debug, Serialize, Deserialize)]
pub struct TodoExport {
    pub version: String,
    pub exported_at: DateTime<Utc>,
    pub todos: Vec<Todo>,
    pub stats: TodoStats,
}

impl TodoExport {
    /// 导出任务列表
    pub fn export(todos: &[Todo]) -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            exported_at: Utc::now(),
            todos: todos.to_vec(),
            stats: TodoStats::from_todos(todos),
        }
    }
}
