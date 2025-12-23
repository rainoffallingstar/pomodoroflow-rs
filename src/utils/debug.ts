// 调试工具
// 提供结构化的调试日志、性能监控和错误追踪功能

import { invoke } from "@tauri-apps/api/tauri";

// 检查是否在开发环境
const isDevelopment = process.env.NODE_ENV === "development";

// 调试日志级别
export enum LogLevel {
  DEBUG = "DEBUG",
  INFO = "INFO",
  WARN = "WARN",
  ERROR = "ERROR",
}

// 日志条目接口
interface LogEntry {
  timestamp: string;
  level: LogLevel;
  component: string;
  action: string;
  message: string;
  data?: any;
  stack?: string;
}

// 性能测量接口
interface PerformanceMeasurement {
  name: string;
  startTime: number;
  endTime?: number;
  duration?: number;
}

/**
 * 调试日志记录器
 * 提供结构化的日志记录，支持开发/生产环境切换
 */
class DebugLogger {
  private logs: LogEntry[] = [];
  private maxLogs = 1000; // 最大日志条数
  private performanceMeasurements: Map<string, PerformanceMeasurement> =
    new Map();
  private enabled: boolean;

  constructor() {
    this.enabled = isDevelopment;
    this.setupGlobalErrorHandling();
  }

  /**
   * 设置全局错误处理
   */
  private setupGlobalErrorHandling() {
    if (!this.enabled) return;

    // 捕获未处理的Promise错误
    window.addEventListener("unhandledrejection", (event) => {
      this.error("Global", "Unhandled Promise Rejection", event.reason);
    });

    // 捕获全局错误
    window.addEventListener("error", (event) => {
      this.error("Global", "Global Error", {
        message: event.message,
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno,
        error: event.error,
      });
    });

    // 捕获React错误边界错误（如果使用React）
    if ((window as any).React) {
      const originalConsoleError = console.error;
      console.error = (...args) => {
        this.error("React", "React Error", args);
        originalConsoleError.apply(console, args);
      };
    }
  }

  /**
   * 记录日志
   */
  log(component: string, action: string, data?: any) {
    if (!this.enabled) return;

    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level: LogLevel.INFO,
      component,
      action,
      message: `${component}: ${action}`,
      data,
    };

    this.addLogEntry(entry);
    console.log(`[${component}] ${action}`, data || "");
  }

  /**
   * 记录调试信息
   */
  debug(component: string, action: string, data?: any) {
    if (!this.enabled) return;

    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level: LogLevel.DEBUG,
      component,
      action,
      message: `${component}: ${action}`,
      data,
    };

    this.addLogEntry(entry);
    console.debug(`[${component}] ${action}`, data || "");
  }

  /**
   * 记录警告
   */
  warn(component: string, action: string, data?: any) {
    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level: LogLevel.WARN,
      component,
      action,
      message: `${component}: ${action}`,
      data,
    };

    this.addLogEntry(entry);
    console.warn(`[${component}] ${action}`, data || "");
  }

  /**
   * 记录错误
   */
  error(component: string, action: string, error: Error | any) {
    const entry: LogEntry = {
      timestamp: new Date().toISOString(),
      level: LogLevel.ERROR,
      component,
      action,
      message: error instanceof Error ? error.message : String(error),
      data: error instanceof Error ? { name: error.name } : error,
      stack: error instanceof Error ? error.stack : undefined,
    };

    this.addLogEntry(entry);
    console.error(`[${component}] ${action}`, error);

    // 在生产环境中发送错误到后端
    if (!isDevelopment) {
      this.reportError(entry).catch(() => {
        // 静默失败，避免无限循环
      });
    }
  }

  /**
   * 开始性能测量
   */
  startMeasurement(name: string) {
    if (!this.enabled) return;

    const measurement: PerformanceMeasurement = {
      name,
      startTime: performance.now(),
    };

    this.performanceMeasurements.set(name, measurement);
    this.debug("Performance", `Start measurement: ${name}`);
  }

  /**
   * 结束性能测量并记录结果
   */
  endMeasurement(name: string) {
    if (!this.enabled) return;

    const measurement = this.performanceMeasurements.get(name);
    if (!measurement) {
      this.warn("Performance", `Measurement not found: ${name}`);
      return;
    }

    measurement.endTime = performance.now();
    measurement.duration = measurement.endTime - measurement.startTime;

    this.log("Performance", `Measurement completed: ${name}`, {
      duration: measurement.duration,
      durationMs: measurement.duration.toFixed(2) + "ms",
    });

    this.performanceMeasurements.delete(name);
  }

  /**
   * 获取所有日志
   */
  getLogs(): LogEntry[] {
    return [...this.logs];
  }

  /**
   * 清除日志
   */
  clearLogs() {
    this.logs = [];
  }

  /**
   * 导出日志
   */
  exportLogs(): string {
    return JSON.stringify(this.logs, null, 2);
  }

  /**
   * 启用/禁用调试
   */
  setEnabled(enabled: boolean) {
    this.enabled = enabled;
  }

  /**
   * 检查是否启用
   */
  isEnabled(): boolean {
    return this.enabled;
  }

  /**
   * 添加日志条目（内部方法）
   */
  private addLogEntry(entry: LogEntry) {
    this.logs.push(entry);

    // 限制日志数量
    if (this.logs.length > this.maxLogs) {
      this.logs = this.logs.slice(-this.maxLogs);
    }
  }

  /**
   * 报告错误到后端（生产环境）
   */
  private async reportError(entry: LogEntry) {
    try {
      await invoke("log_error", {
        message: entry.message,
        component: entry.component,
        action: entry.action,
        data: entry.data,
        stack: entry.stack,
        timestamp: entry.timestamp,
      });
    } catch (error) {
      // 静默失败，避免无限循环
      console.warn("Failed to report error to backend:", error);
    }
  }
}

/**
 * 应用状态监控器
 * 监控应用关键状态变化
 */
class AppStateMonitor {
  private logger: DebugLogger;
  private stateHistory: Map<string, any[]> = new Map();
  private maxStateHistory = 50;

  constructor(logger: DebugLogger) {
    this.logger = logger;
  }

  /**
   * 记录状态变化
   */
  logStateChange(
    storeName: string,
    action: string,
    prevState: any,
    nextState: any,
  ) {
    if (!this.logger.isEnabled()) return;

    // 记录状态变化摘要
    const changes = this.detectChanges(prevState, nextState);

    this.logger.debug("State", `${storeName}.${action}`, {
      changes,
      prevState: this.sanitizeState(prevState),
      nextState: this.sanitizeState(nextState),
    });

    // 保存状态历史
    this.saveStateHistory(storeName, nextState);
  }

  /**
   * 检测状态变化
   */
  private detectChanges(prev: any, next: any): string[] {
    const changes: string[] = [];

    if (prev === next) return changes;

    if (typeof prev === "object" && typeof next === "object") {
      const allKeys = new Set([
        ...Object.keys(prev || {}),
        ...Object.keys(next || {}),
      ]);

      for (const key of allKeys) {
        if (prev[key] !== next[key]) {
          changes.push(key);
        }
      }
    } else {
      changes.push("root");
    }

    return changes;
  }

  /**
   * 清理状态（移除敏感信息）
   */
  private sanitizeState(state: any): any {
    if (!state || typeof state !== "object") return state;

    const sanitized: any = Array.isArray(state) ? [] : {};
    const sensitiveKeys = ["token", "password", "secret", "key", "credential"];

    for (const [key, value] of Object.entries(state)) {
      if (
        sensitiveKeys.some((sensitive) => key.toLowerCase().includes(sensitive))
      ) {
        sanitized[key] = "***REDACTED***";
      } else if (value && typeof value === "object") {
        sanitized[key] = this.sanitizeState(value);
      } else {
        sanitized[key] = value;
      }
    }

    return sanitized;
  }

  /**
   * 保存状态历史
   */
  private saveStateHistory(storeName: string, state: any) {
    if (!this.stateHistory.has(storeName)) {
      this.stateHistory.set(storeName, []);
    }

    const history = this.stateHistory.get(storeName)!;
    history.push({
      timestamp: new Date().toISOString(),
      state: this.sanitizeState(state),
    });

    // 限制历史记录数量
    if (history.length > this.maxStateHistory) {
      history.shift();
    }
  }

  /**
   * 获取状态历史
   */
  getStateHistory(storeName: string): any[] {
    return this.stateHistory.get(storeName) || [];
  }
}

/**
 * Tauri事件监控器
 * 监控Tauri事件系统
 */
class TauriEventMonitor {
  private logger: DebugLogger;
  private eventCounts: Map<string, number> = new Map();

  constructor(logger: DebugLogger) {
    this.logger = logger;
  }

  /**
   * 记录Tauri事件
   */
  logEvent(eventName: string, payload?: any) {
    if (!this.logger.isEnabled()) return;

    // 更新事件计数
    const count = (this.eventCounts.get(eventName) || 0) + 1;
    this.eventCounts.set(eventName, count);

    this.logger.debug("TauriEvent", eventName, {
      payload,
      count,
      timestamp: new Date().toISOString(),
    });
  }

  /**
   * 获取事件统计
   */
  getEventStats(): Record<string, number> {
    const stats: Record<string, number> = {};
    for (const [event, count] of this.eventCounts) {
      stats[event] = count;
    }
    return stats;
  }
}

// 创建全局调试实例
export const debug = new DebugLogger();
export const appStateMonitor = new AppStateMonitor(debug);
export const tauriEventMonitor = new TauriEventMonitor(debug);

// 导出工具函数 - 使用 perfUtils 避免与全局 performance 冲突
export const perfUtils = {
  /**
   * 测量函数执行时间
   */
  measure<T>(name: string, fn: () => T): T {
    debug.startMeasurement(name);
    try {
      const result = fn();
      debug.endMeasurement(name);
      return result;
    } catch (error) {
      debug.endMeasurement(name);
      throw error;
    }
  },

  /**
   * 异步测量函数执行时间
   */
  async measureAsync<T>(name: string, fn: () => Promise<T>): Promise<T> {
    debug.startMeasurement(name);
    try {
      const result = await fn();
      debug.endMeasurement(name);
      return result;
    } catch (error) {
      debug.endMeasurement(name);
      throw error;
    }
  },
};

// 开发环境辅助函数
if (isDevelopment) {
  // 将调试工具暴露到全局，方便在浏览器控制台中使用
  (window as any).__DEBUG__ = {
    debug,
    appStateMonitor,
    tauriEventMonitor,
    perfUtils: performance,
    getLogs: () => debug.getLogs(),
    exportLogs: () => debug.exportLogs(),
    clearLogs: () => debug.clearLogs(),
    enableDebug: () => debug.setEnabled(true),
    disableDebug: () => debug.setEnabled(false),
  };

  console.log("🔧 调试工具已启用，可通过 window.__DEBUG__ 访问");
}
