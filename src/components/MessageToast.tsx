import { useEffect } from "react";
import { useAppStore } from "../stores/appStore";
import "../styles/MessageToast.css";

const ERROR_CODE_LABELS: Record<string, string> = {
  VALIDATION: "输入错误",
  INVALID_STATE: "状态错误",
  NOT_FOUND: "资源不存在",
  NETWORK: "网络错误",
  AUTH: "认证失败",
  PERMISSION: "权限不足",
  DATABASE: "数据库错误",
  INTERNAL: "内部错误",
};

function getErrorCodeLabel(code: string): string {
  return ERROR_CODE_LABELS[code] ?? "未知错误";
}

export function MessageToast() {
  const { error, errorCode, success, clearMessages } = useAppStore();

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
        <div className="message-toast-body">
          {type === "error" && errorCode && (
            <span className="message-toast-code" title={errorCode}>
              {getErrorCodeLabel(errorCode)}
            </span>
          )}
          <span className="message-toast-text">{message}</span>
        </div>
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
