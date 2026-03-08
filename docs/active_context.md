# PomodoroFlow-Rs 项目状态

**最后更新**: 2026-03-07

## 项目概述

PomodoroFlow-Rs 是一个基于 **Tauri + React + Rust** 开发的番茄钟桌面应用，提供专注计时和本地任务管理功能。

## 2026-03 重构进度快照

- 已完成: 命令层错误契约统一为 `CommandResult { success, data, error, error_code }`，前端统一解析并保留错误码。
- 已完成: 番茄钟阶段自动切换由后端状态机负责，前端只消费状态；事件系统支持阶段边沿检测。
- 已完成: 配置保存链路支持运行时同步失败回滚，避免“落盘成功但运行态失败”。
- 已完成: `Todo` 重新接回 GitHub 关联字段（`github_issue_id/github_issue_number/github_project_id`），数据库读写一致。
- 已完成: 新增 GitHub 关联命令 `link_todo_github` 与 `clear_todo_github_link`，并接入前端 store action。
- 已完成: 用户配置新增 GitHub Project 三元组（`selected_project_owner/repo/number`）并打通设置面板与数据库持久化。
- 已完成: 新增 `get_github_sync_config` 命令，后端可直接读取完整 GitHub 同步配置。
- 已完成: 新增 `run_github_sync` 命令（dry-run），可读取并分析 `sync_queue`，输出支持/不支持/无效项统计。
- 已完成: 设置页增加“检查 GitHub 同步队列”按钮，前端可直接触发 dry-run 并展示结果。
- 已完成: 回归测试通过（Rust 核心 + 前端 Vitest）。
- 进行中: 真正的 GitHub API 同步（鉴权、拉取 Project item、双向冲突处理）尚未接入。

## 核心功能

## 最近更新 (v0.2.0)

### 🎨 UI 重构 - iOS 18 质感界面 (2025-12-23)

**界面设计更新**：
- ✨ 全新的 iOS 18 质感设计系统
- 🧊 毛玻璃卡片效果（backdrop-filter: blur(40px)）
- ⭕ 圆环进度条动画，220px 大尺寸，88px 超细字体
- 📱 左右双卡片布局：左侧番茄钟（固定400px），右侧 TodoList（弹性宽度）
- 🔘 浮动设置按钮（右上角 44x44），移除顶部 Header
- 📜 右侧列表卡片内滚动，三段式标签页和输入框固定

**布局优化**：
- 页面整体固定不滚动（overflow: hidden）
- 左右卡片独立，高度自适应一致（align-items: stretch）
- 右侧 TodoList：标签页、输入框固定，只有列表区域滚动
- 响应式设计：>1200px (1:1.5), 768-1200px (1:1), <768px (上下堆叠)

**新增文件**：
- `src/styles/iOS-theme.css` - iOS 18 设计系统
- `src/styles/iOS-layout.css` - 左右布局样式
- `src/styles/PomodoroCard.css` - 番茄钟卡片样式
- `src/styles/FloatingSettingsButton.css` - 浮动按钮样式

**移除内容**：
- 顶部 Header 组件
- 底部 Footer 状态栏
- 整体页面滚动

### 🎯 重大修复 (v0.1.0)

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
   - **新增：圆环进度条（SVG），Phase 颜色渐变**
   - **新增：居中布局，组件间距统一**
   - **修复：配置保存后立即生效**

2. **待办事项模块**
   - 创建/编辑/删除任务
   - 任务状态管理（待办/进行中/已完成）
   - 任务搜索和筛选
   - 本地 SQLite 数据持久化
   - **iOS 18 分组列表（Inset Grouped）风格**
   - **列表内容在卡片内滚动，头部固定**
   - **三段式标签页（待办/进行中/已完成）**
   - **在当前标签页创建任务会自动设置对应状态**

3. **用户配置**
   - 番茄钟时长配置
   - 主题切换（浅色/深色/系统）
   - 通知设置
   - 配置持久化
   - **iOS 风格设置面板（模态框）**

4. **用户界面**
   - **iOS 18 Design System**
   - **左右双卡片布局**
   - **毛玻璃效果**
   - **浮动设置按钮**
   - 响应式设计
   - 键盘快捷键支持
   - 加载状态指示
   - 错误提示系统

## 技术栈

### 前端
- **框架**: React 18 + TypeScript
- **构建**: Vite
- **状态管理**: Zustand
- **样式**: iOS 18 Design System
  - 变量系统：`--ios-bg-primary`, `--ios-text-primary` 等
  - 颜色：遵循 iOS 18 HIG 规范
  - 动画曲线：`--ios-spring`, `--ios-ease-out`
  - 边框半径：`--ios-radius-small/medium/large/xlarge`

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
- `set_todo_status(id, status)` - 显式设置状态
- `link_todo_github(id, issue_id, issue_number, project_id)` - 关联 GitHub 元数据
- `clear_todo_github_link(id)` - 清除 GitHub 关联
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

- `src-tauri` 全量测试在当前环境受系统依赖限制（缺少 `libsoup-2.4`）无法完整执行。
- 前端测试仍有 `ReactDOMTestUtils.act` 的依赖告警（来自测试栈），不影响当前功能正确性。

## 未来计划

- [ ] 番茄钟统计页面
- [ ] 任务优先级和标签
- [ ] 数据导入/导出
- [ ] 自定义通知音效
