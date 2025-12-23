import { useAppStore } from "../stores/appStore";
import "../styles/App.css";

export function StatsPanel() {
  const { todos, pomodoroSession } = useAppStore();

  // 计算任务统计数据
  const totalTasks = todos.length;
  const pendingTasks = todos.filter((todo) => todo.status === "todo").length;
  const inProgressTasks = todos.filter(
    (todo) => todo.status === "in_progress",
  ).length;
  const completedTasks = todos.filter((todo) => todo.status === "done").length;

  // 计算番茄钟统计数据（简化版本）
  const totalPomodoros = pomodoroSession?.cycle_count || 0;
  const todayPomodoros = pomodoroSession?.cycle_count || 0; // 简化：暂时使用总数
  const totalFocusTime = (pomodoroSession?.cycle_count || 0) * 25; // 25分钟每个番茄钟
  const todayFocusTime = (pomodoroSession?.cycle_count || 0) * 25; // 简化

  // 计算任务完成率
  const completionRate =
    totalTasks > 0 ? Math.round((completedTasks / totalTasks) * 100) : 0;

  // 计算平均番茄钟效率
  const efficiencyRate =
    totalPomodoros > 0
      ? Math.min(100, Math.round((completedTasks / totalPomodoros) * 100))
      : 0;

  const formatTime = (minutes: number) => {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    if (hours > 0) {
      return `${hours}h ${mins}m`;
    }
    return `${mins}m`;
  };

  return (
    <div className="card">
      <div className="card-header">
        <h2 className="card-title">统计数据</h2>
        <p className="text-sm text-tertiary mt-1">今日工作概览</p>
      </div>

      <div className="stats-grid">
        {/* 番茄钟统计 */}
        <div className="stat-item">
          <div className="stat-value text-accent">{todayPomodoros}</div>
          <div className="stat-label">今日番茄钟</div>
        </div>

        <div className="stat-item">
          <div className="stat-value text-accent">
            {formatTime(todayFocusTime)}
          </div>
          <div className="stat-label">专注时间</div>
        </div>

        <div className="stat-item">
          <div className="stat-value text-accent">{totalPomodoros}</div>
          <div className="stat-label">总番茄钟</div>
        </div>

        <div className="stat-item">
          <div className="stat-value text-accent">
            {formatTime(totalFocusTime)}
          </div>
          <div className="stat-label">总专注时间</div>
        </div>

        {/* 任务统计 */}
        <div className="stat-item">
          <div className="stat-value text-success">{completedTasks}</div>
          <div className="stat-label">已完成任务</div>
        </div>

        <div className="stat-item">
          <div className="stat-value text-warning">{inProgressTasks}</div>
          <div className="stat-label">进行中任务</div>
        </div>

        <div className="stat-item">
          <div className="stat-value text-error">{pendingTasks}</div>
          <div className="stat-label">待办任务</div>
        </div>

        <div className="stat-item">
          <div className="stat-value text-primary">{totalTasks}</div>
          <div className="stat-label">总任务数</div>
        </div>

        {/* 效率统计 */}
        <div className="stat-item">
          <div className="stat-value text-success">{completionRate}%</div>
          <div className="stat-label">任务完成率</div>
        </div>

        <div className="stat-item">
          <div className="stat-value text-accent">{efficiencyRate}%</div>
          <div className="stat-label">番茄钟效率</div>
        </div>

        <div className="stat-item">
          <div className="stat-value text-primary">
            {totalTasks > 0 ? Math.round((completedTasks / totalTasks) * 5) : 0}
            /5
          </div>
          <div className="stat-label">今日评分</div>
        </div>

        <div className="stat-item">
          <div className="stat-value text-tertiary">
            {new Date().toLocaleDateString("zh-CN", {
              month: "short",
              day: "numeric",
            })}
          </div>
          <div className="stat-label">日期</div>
        </div>
      </div>

      {/* 进度条 */}
      <div className="mt-6">
        <div className="flex justify-between items-center mb-2">
          <span className="text-sm font-medium text-primary">今日进度</span>
          <span className="text-sm text-tertiary">
            {completedTasks}/{totalTasks} 任务完成
          </span>
        </div>
        <div className="w-full bg-bg-tertiary rounded-full h-2">
          <div
            className="bg-success-color h-2 rounded-full transition-all duration-300"
            style={{ width: `${completionRate}%` }}
          ></div>
        </div>
      </div>

      {/* 提示信息 */}
      <div className="mt-4 p-3 bg-bg-tertiary rounded-lg">
        <div className="flex items-start gap-2">
          <div className="text-warning">💡</div>
          <div className="text-sm">
            {completionRate >= 80 ? (
              <span className="text-success">
                太棒了！继续保持高效工作节奏。
              </span>
            ) : completionRate >= 50 ? (
              <span className="text-primary">
                不错！今天已经完成了一半以上的任务。
              </span>
            ) : totalTasks === 0 ? (
              <span className="text-tertiary">
                还没有任务，开始添加第一个任务吧！
              </span>
            ) : (
              <span className="text-warning">
                加油！专注于当前最重要的任务。
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
