import { useState, useEffect } from "react";
import { useAppStore, UserConfig } from "../stores/appStore";

interface SettingsPanelProps {
  onClose?: () => void;
}

/**
 * iOS 18 风格设置面板
 * - 毛玻璃导航栏
 * - 分组列表样式
 * - Toggle 开关
 */
export function SettingsPanel({ onClose }: SettingsPanelProps) {
  const { userConfig, saveUserConfig, loadUserConfig, runGithubSync } = useAppStore();
  const [localConfig, setLocalConfig] = useState<UserConfig | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [isSyncChecking, setIsSyncChecking] = useState(false);
  const [isSyncRunning, setIsSyncRunning] = useState(false);
  const [message, setMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  useEffect(() => {
    if (userConfig) {
      setLocalConfig({ ...userConfig });
    }
  }, [userConfig]);

  const handleSave = async () => {
    if (!localConfig) return;
    setIsSaving(true);
    setMessage(null);

    try {
      await saveUserConfig(localConfig);
      await loadUserConfig();
      setMessage({ type: "success", text: "设置保存成功" });
      // 保存成功后延迟关闭面板
      setTimeout(() => {
        setMessage(null);
        onClose?.();
      }, 1500);
    } catch (error) {
      setMessage({
        type: "error",
        text: error instanceof Error ? error.message : "保存失败，请重试",
      });
      // 错误消息显示更长时间
      setTimeout(() => setMessage(null), 3000);
    } finally {
      setIsSaving(false);
    }
  };

  const handleSyncCheck = async () => {
    setIsSyncChecking(true);
    setMessage(null);
    try {
      const report = await runGithubSync(true);
      setMessage({
        type: "success",
        text: `同步检查完成：待处理 ${report.pending_items} 项，支持 ${report.supported_items} 项`,
      });
    } catch (error) {
      setMessage({
        type: "error",
        text: error instanceof Error ? error.message : "同步检查失败",
      });
    } finally {
      setIsSyncChecking(false);
    }
  };

  const handleSyncRun = async () => {
    setIsSyncRunning(true);
    setMessage(null);
    try {
      const report = await runGithubSync(false);
      const hasErrors = report.errors.length > 0;
      setMessage({
        type: hasErrors ? "error" : "success",
        text: hasErrors
          ? `同步完成，但有 ${report.errors.length} 条错误（已记录队列状态）`
          : `同步完成：处理 ${report.supported_items} 项`,
      });
    } catch (error) {
      setMessage({
        type: "error",
        text: error instanceof Error ? error.message : "执行同步失败",
      });
    } finally {
      setIsSyncRunning(false);
    }
  };

  const handleToggle = (field: keyof UserConfig) => {
    if (!localConfig) return;
    setLocalConfig({
      ...localConfig,
      [field]: !localConfig[field],
    });
  };

  const handleInputChange = (
    field: keyof UserConfig,
    value: string | number | boolean | null,
  ) => {
    if (!localConfig) return;
    setLocalConfig({
      ...localConfig,
      [field]: value,
    });
  };

  if (!localConfig) {
    return (
      <div className="ios-settings-modal">
        <div className="ios-settings-nav">
          <button className="ios-settings-back" onClick={onClose} />
          <span className="ios-settings-title">设置</span>
          <div style={{ width: "32px" }} />
        </div>
        <div className="ios-settings-content">
          <p className="ios-loading-text">加载中...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="ios-settings-modal">
      {/* iOS 导航栏 */}
      <div className="ios-settings-nav">
        <button className="ios-settings-back" onClick={onClose} />
        <span className="ios-settings-title">设置</span>
        <div style={{ width: "32px" }} />
      </div>

      <div className="ios-settings-content">
        {/* 番茄钟设置 */}
        <div className="ios-settings-section">
          <div className="ios-settings-section-title">番茄钟</div>
          <div className="ios-setting-group">
            <div className="ios-setting-item">
              <span className="ios-setting-label">工作时间</span>
              <input
                type="number"
                className="ios-setting-input"
                value={localConfig.pomodoro_work_duration / 60}
                onChange={(e) =>
                  handleInputChange(
                    "pomodoro_work_duration",
                    parseInt(e.target.value) * 60,
                  )
                }
                min="1"
                max="60"
              />
              <span className="ios-setting-unit">分钟</span>
            </div>
            <div className="ios-setting-item">
              <span className="ios-setting-label">短休息</span>
              <input
                type="number"
                className="ios-setting-input"
                value={localConfig.pomodoro_short_break_duration / 60}
                onChange={(e) =>
                  handleInputChange(
                    "pomodoro_short_break_duration",
                    parseInt(e.target.value) * 60,
                  )
                }
                min="1"
                max="30"
              />
              <span className="ios-setting-unit">分钟</span>
            </div>
            <div className="ios-setting-item">
              <span className="ios-setting-label">长休息</span>
              <input
                type="number"
                className="ios-setting-input"
                value={localConfig.pomodoro_long_break_duration / 60}
                onChange={(e) =>
                  handleInputChange(
                    "pomodoro_long_break_duration",
                    parseInt(e.target.value) * 60,
                  )
                }
                min="1"
                max="60"
              />
              <span className="ios-setting-unit">分钟</span>
            </div>
            <div className="ios-setting-item">
              <span className="ios-setting-label">循环次数</span>
              <input
                type="number"
                className="ios-setting-input"
                value={localConfig.pomodoro_cycles_until_long_break}
                onChange={(e) =>
                  handleInputChange(
                    "pomodoro_cycles_until_long_break",
                    parseInt(e.target.value),
                  )
                }
                min="1"
                max="10"
              />
              <span className="ios-setting-unit">次</span>
            </div>
          </div>
        </div>

        {/* 通知设置 */}
        <div className="ios-settings-section">
          <div className="ios-settings-section-title">通知</div>
          <div className="ios-setting-group">
            <div className="ios-setting-item">
              <span className="ios-setting-label">桌面通知</span>
              <div
                className={`ios-toggle ${localConfig.notifications_enabled ? "active" : ""}`}
                onClick={() => handleToggle("notifications_enabled")}
              />
            </div>
            <div className="ios-setting-item">
              <span className="ios-setting-label">声音提示</span>
              <div
                className={`ios-toggle ${localConfig.sound_enabled ? "active" : ""}`}
                onClick={() => handleToggle("sound_enabled")}
              />
            </div>
          </div>
        </div>

        {/* 主题设置 */}
        <div className="ios-settings-section">
          <div className="ios-settings-section-title">外观</div>
          <div className="ios-setting-group">
            <div className="ios-setting-item">
              <span className="ios-setting-label">主题</span>
              <select
                className="ios-setting-select"
                value={localConfig.theme}
                onChange={(e) => handleInputChange("theme", e.target.value)}
              >
                <option value="light">浅色</option>
                <option value="dark">深色</option>
                <option value="system">跟随系统</option>
              </select>
            </div>
          </div>
        </div>

        {/* GitHub Project 设置 */}
        <div className="ios-settings-section">
          <div className="ios-settings-section-title">GitHub Project</div>
          <div className="ios-setting-group">
            <div className="ios-setting-item">
              <span className="ios-setting-label">Username</span>
              <input
                type="text"
                className="ios-setting-input"
                value={localConfig.github_username ?? ""}
                onChange={(e) =>
                  handleInputChange(
                    "github_username",
                    e.target.value.trim(),
                  )
                }
                placeholder="your-username"
              />
            </div>
            <div className="ios-setting-item">
              <span className="ios-setting-label">Token</span>
              <input
                type="password"
                className="ios-setting-input"
                value={localConfig.github_token_encrypted ?? ""}
                onChange={(e) =>
                  handleInputChange(
                    "github_token_encrypted",
                    e.target.value.trim(),
                  )
                }
                placeholder="ghp_xxx or github_pat_xxx"
              />
            </div>
            <div className="ios-setting-item">
              <span className="ios-setting-label">Owner</span>
              <input
                type="text"
                className="ios-setting-input"
                value={localConfig.selected_project_owner ?? ""}
                onChange={(e) =>
                  handleInputChange(
                    "selected_project_owner",
                    e.target.value.trim() || null,
                  )
                }
                placeholder="your-org"
              />
            </div>
            <div className="ios-setting-item">
              <span className="ios-setting-label">Repo</span>
              <input
                type="text"
                className="ios-setting-input"
                value={localConfig.selected_project_repo ?? ""}
                onChange={(e) =>
                  handleInputChange(
                    "selected_project_repo",
                    e.target.value.trim() || null,
                  )
                }
                placeholder="your-repo"
              />
            </div>
            <div className="ios-setting-item">
              <span className="ios-setting-label">Project #</span>
              <input
                type="number"
                className="ios-setting-input"
                value={localConfig.selected_project_number ?? ""}
                onChange={(e) =>
                  handleInputChange(
                    "selected_project_number",
                    e.target.value ? parseInt(e.target.value, 10) : null,
                  )
                }
                min="1"
                placeholder="1"
              />
            </div>
          </div>
        </div>

        {/* 保存按钮 */}
        <button
          className={`ios-btn ${isSyncChecking || isSyncRunning ? "disabled" : ""}`}
          onClick={handleSyncCheck}
          disabled={isSyncChecking || isSyncRunning}
        >
          {isSyncChecking ? "检查中..." : "检查 GitHub 同步队列"}
        </button>

        <button
          className={`ios-btn ${isSyncChecking || isSyncRunning ? "disabled" : ""}`}
          onClick={handleSyncRun}
          disabled={isSyncChecking || isSyncRunning}
        >
          {isSyncRunning ? "同步中..." : "执行 GitHub 同步"}
        </button>

        <button
          className={`ios-btn ios-btn-primary ${isSaving ? "disabled" : ""}`}
          onClick={handleSave}
          disabled={isSaving}
        >
          {isSaving ? "保存中..." : "保存设置"}
        </button>

        {/* 状态消息 */}
        {message && (
          <div className={`ios-toast ${message.type} show`}>{message.text}</div>
        )}
      </div>
    </div>
  );
}
