# PomodoroFlow-Rs 项目状态

**最后更新**: 2025-12-23

## 项目概述

PomodoroFlow-Rs 是一个基于 **Tauri + React + Rust** 开发的番茄钟桌面应用，提供专注计时和本地任务管理功能。

## 核心功能

## 最近更新 (v0.1.0)

### 🎯 重大修复 (2025-12-23)

**问题 1: 设置保存后不生效**
- **原因**: `saveUserConfig` 只保存到数据库，没有更新运行时 `PomodoroService`
- **修复**: 添加 `update_pomodoro_config` 方法，现在保存配置后会立即更新计时器配置

**问题 2: 新创建的任务总是出现在"已完成"标签**
- **根本原因**: 多个bug组合导致
  - 前端创建任务时硬编码 `status: "todo"`，没有传递当前标签页状态
  - 后端返回数据时覆盖了前端设置的正确状态
  - 前端过滤逻辑中"done"标签页返回 truthy 字符串而不是布尔值
  - 后端序列化问题（`TodoStatus` 枚举没有正确的序列化配置）
- **修复**: 
  - 前端现在传递 `activeTab` 作为 `initialStatus`
  - 后端接受可选的 `status` 参数
  - 前端强制保留前端设置的状态不被后端覆盖
  - 为 `TodoStatus` 和 `Todo` 添加 `#[serde(rename_all = "snake_case")]`
  - 修复前端过滤逻辑的布尔返回值问题

**当前行为**:
- ✅ 在"待办"标签创建任务 → 状态为 "todo" → 只出现在"待办"标签
- ✅ 在"进行中"标签创建任务 → 状态为 "in_progress" → 只出现在"进行中"标签
- ✅ 在"已完成"标签创建任务 → 状态为 "done" → 只出现在"已完成"标签

### ✅ 已实现

1. **番茄钟模块**
   - 倒计时显示（MM:SS 格式）
   - 三种周期：工作(25分钟) / 短休息(5分钟) / 长休息(15分钟)
   - 开始/暂停/重置/跳过控制
   - 系统通知提醒
   - 周期自动切换
   - 可配置时长
   - **修复：配置保存后立即生效**

2. **待办事项模块**
   - 创建/编辑/删除任务
   - 任务状态管理（待办/进行中/已完成）
   - 任务搜索和筛选
   - 本地 SQLite 数据持久化
   - **新增：三标签页视图**（待办/进展中/已完成）
   - **修复：在当前标签页创建任务会自动设置对应状态**

3. **用户配置**
   - 番茄钟时长配置
   - 主题切换（浅色/深色/系统）
   - 通知设置
   - 配置持久化

4. **用户界面**
   - 响应式设计
   - 键盘快捷键支持
   - 加载状态指示
   - 错误提示系统
   - **新增：iOS 18 风格分段控件**

## 技术栈

### 前端
- **框架**: React 18 + TypeScript
- **构建**: Vite
- **状态管理**: Zustand
- **样式**: 自定义 CSS（亮色/暗色主题）

### 后端
- **框架**: Tauri 1.0+
- **语言**: Rust (2021 Edition)
- **数据库**: SQLite (rusqlite)
- **异步**: Tokio

## 项目结构

```
pomoflow-rs/
├── src/                    # React 前端
│   ├── components/         # UI 组件
│   ├── stores/            # Zustand 状态管理
│   ├── hooks/             # 自定义钩子
│   └── styles/            # CSS 样式
├── src-tauri/             # Tauri 后端
│   ├── src/
│   │   ├── main.rs        # 应用入口
│   │   └── commands/      # Tauri 命令
│   └── Cargo.toml
└── src/                   # Rust 核心库
    ├── core/              # 业务逻辑
    ├── storage/           # 数据库层
    └── async_utils/       # 异步工具
```

## Tauri 命令 API

### 番茄钟命令
- `start_pomodoro()` - 启动计时
- `pause_pomodoro()` - 暂停计时
- `reset_pomodoro()` - 重置计时
- `skip_pomodoro_phase()` - 跳过当前阶段
- `get_pomodoro_session()` - 获取当前会话
- `update_pomodoro_config(config)` - 更新配置

### 待办事项命令
- `get_todos()` - 获取所有任务
- `create_todo(title, description, status)` - 创建任务（可指定初始状态）
- `update_todo(id, updates)` - 更新任务
- `delete_todo(id)` - 删除任务
- `toggle_todo_status(id)` - 切换状态
- `get_todo_stats()` - 获取统计

### 配置命令
- `get_user_config()` - 获取用户配置
- `save_user_config(config)` - 保存配置

## 数据库设计

### 核心表

**todos** - 待办事项
```sql
CREATE TABLE todos (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);
```

**user_config** - 用户配置
```sql
CREATE TABLE user_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    pomodoro_work_duration INTEGER DEFAULT 1500,
    pomodoro_short_break_duration INTEGER DEFAULT 300,
    pomodoro_long_break_duration INTEGER DEFAULT 900,
    pomodoro_cycles_until_long_break INTEGER DEFAULT 4,
    notifications_enabled BOOLEAN DEFAULT 1,
    sound_enabled BOOLEAN DEFAULT 1,
    theme TEXT DEFAULT 'light'
);
```

**pomodoro_sessions** - 番茄钟会话记录
```sql
CREATE TABLE pomodoro_sessions (
    id INTEGER PRIMARY KEY,
    phase TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL,
    completed_at TIMESTAMP NOT NULL,
    cycle_count INTEGER NOT NULL
);
```

## 性能指标

- **启动时间**: < 5 秒
- **内存占用**: < 150 MB
- **UI 响应**: < 100ms
- **数据库操作**: < 10ms

## 开发命令

```bash
# 开发模式
npm run tauri:dev

# 构建前端
npm run build

# 运行测试
npm test

# Tauri 构建
npm run tauri build
```

## 已知问题

无关键问题。应用功能完整，性能稳定。

## 未来计划

- [ ] 番茄钟统计页面
- [ ] 任务优先级和标签
- [ ] 数据导入/导出
- [ ] 自定义通知音效
