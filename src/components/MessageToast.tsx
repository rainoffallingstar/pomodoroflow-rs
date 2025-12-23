import { useEffect } from "react";
import { useAppStore } from "../stores/appStore";
import "../styles/MessageToast.css";

export function MessageToast() {
  const { error, success, clearMessages } = useAppStore();

  // 自动清除消息
  useEffect(() => {
    if (error || success) {
      const timer = setTimeout(() => {
        clearMessages();
      }, 3000);
      return () => clearTimeout(timer);
    }
  }, [error, success, clearMessages]);

  if (!error && !success) {
    return null;
  }

  const message = error || success;
  const type = error ? "error" : "success";

  return (
    <div className={`message-toast message-toast-${type}`}>
      <div className="message-toast-content">
        <span className="message-toast-icon">
          {type === "error" ? "❌" : "✅"}
        </span>
        <span className="message-toast-text">{message}</span>
        <button
          className="message-toast-close"
          onClick={clearMessages}
          aria-label="关闭"
        >
          ✕
        </button>
      </div>
    </div>
  );
}
