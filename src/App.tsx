import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { Header } from "./components/Header";
import { MainLayout } from "./components/MainLayout";
import { SettingsPanel } from "./components/SettingsPanel";
import { MessageToast } from "./components/MessageToast";
import { useAppStore } from "./stores/appStore";
import "./styles/App.css";

function App() {
  const [isInitializing, setIsInitializing] = useState(true);
  const [initializationError, setInitializationError] = useState<string | null>(
    null,
  );
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  const { theme, initializeApp, isLoading } = useAppStore();

  useEffect(() => {
    // 初始化应用状态
    initializeApp();

    // 监听初始化完成事件
    const unlistenComplete = listen("initialization-complete", () => {
      console.log("✅ 后端初始化完成");
      setIsInitializing(false);
    });

    // 监听初始化错误事件
    const unlistenError = listen("initialization-error", (event) => {
      console.error("❌ 后端初始化失败:", event.payload);
      setInitializationError(event.payload as string);
      setIsInitializing(false);
    });

    // 设置超时，防止事件丢失
    const timeoutId = setTimeout(() => {
      console.warn("⚠️ 初始化超时，强制显示界面");
      setIsInitializing(false);
    }, 10000);

    return () => {
      unlistenComplete.then((f) => f());
      unlistenError.then((f) => f());
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

      {/* 顶部导航栏 */}
      <Header onOpenSettings={() => setIsSettingsOpen(true)} />

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

      {/* 底部状态栏 */}
      <footer className="app-footer">
        <div className="footer-content">
          <span className="footer-text">
            PomodoroFlow-Rs v0.1.0 • 使用番茄工作法提高工作效率
          </span>
        </div>
      </footer>
    </div>
  );
}

export default App;
