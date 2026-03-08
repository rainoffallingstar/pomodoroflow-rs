import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { MainLayout } from "./components/MainLayout";
import { SettingsPanel } from "./components/SettingsPanel";
import { MessageToast } from "./components/MessageToast";
import { FloatingSettingsButton } from "./components/FloatingSettingsButton";
import { useAppStore } from "./stores/appStore";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import "./styles/App.css";

function App() {
  const [isInitializing, setIsInitializing] = useState(true);
  const [initializationError, setInitializationError] = useState<string | null>(
    null,
  );
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  const { theme, initializeApp, isLoading } = useAppStore();
  useKeyboardShortcuts(() => setIsSettingsOpen(true));

  useEffect(() => {
    let disposed = false;
    let cleanupComplete: (() => void) | null = null;
    let cleanupError: (() => void) | null = null;

    // 先订阅初始化事件，再启动初始化，避免事件先发后收导致的竞态。
    const setupInitialization = async () => {
      cleanupComplete = await listen("initialization-complete", () => {
        if (disposed) return;
        console.log("✅ 后端初始化完成");
        setIsInitializing(false);
      });

      cleanupError = await listen("initialization-error", (event) => {
        if (disposed) return;
        console.error("❌ 后端初始化失败:", event.payload);
        setInitializationError(event.payload as string);
        setIsInitializing(false);
      });

      if (typeof initializeApp === "function") {
        await initializeApp();
        if (!disposed) {
          setIsInitializing(false);
        }
      } else if (!disposed) {
        setIsInitializing(false);
      }
    };

    void setupInitialization();

    // 设置超时，防止事件丢失
    const timeoutId = setTimeout(() => {
      console.warn("⚠️ 初始化超时，强制显示界面");
      setIsInitializing(false);
    }, 10000);

    return () => {
      disposed = true;
      cleanupComplete?.();
      cleanupError?.();
      clearTimeout(timeoutId);
    };
  }, [initializeApp]);

  // 加载状态界面
  if (isInitializing || isLoading) {
    return (
      <div className="loading-screen">
        <div className="loading-content">
          <h1 className="loading-title">🍅 PomodoroFlow-Rs</h1>
          <p className="loading-text">正在初始化应用...</p>

          <div className="loading-spinner" />

          <div className="loading-hint">
            <p>
              <strong>提示：</strong> 应用正在后台初始化，这可能需要几秒钟时间。
              初始化完成后界面会自动刷新。
            </p>
          </div>
        </div>
      </div>
    );
  }

  // 错误状态界面
  if (initializationError) {
    return (
      <div className="error-screen">
        <div className="error-content">
          <h1 className="error-title">⚠️ 初始化错误</h1>
          <p className="error-text">应用启动时遇到问题</p>

          <div className="error-details">
            <h2>错误详情</h2>
            <pre>{initializationError}</pre>
            <p>请检查控制台获取更多信息，或重启应用。</p>
          </div>

          <button
            className="btn btn-primary"
            onClick={() => window.location.reload()}
          >
            重试
          </button>
        </div>
      </div>
    );
  }

  // 正常应用界面
  return (
    <div className={`app theme-${theme}`}>
      {/* 消息提示 */}
      <MessageToast />

      {/* 浮动设置按钮 */}
      <FloatingSettingsButton onClick={() => setIsSettingsOpen(true)} />

      {/* 主内容区域 */}
      <main className="app-main">
        <MainLayout />
      </main>

      {/* 设置面板（iOS 18 风格模态框） */}
      {isSettingsOpen && (
        <div
          className="modal-overlay"
          onClick={(e) => {
            // 只在点击遮罩层本身时关闭，不传递点击事件给子组件
            if (e.target === e.currentTarget) {
              setIsSettingsOpen(false);
            }
          }}
        >
          <SettingsPanel onClose={() => setIsSettingsOpen(false)} />
        </div>
      )}
    </div>
  );
}

export default App;
