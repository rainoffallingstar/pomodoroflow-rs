import { useState, useEffect } from 'react'
import '../styles/App.css'

export type ToastType = 'success' | 'error' | 'warning' | 'info'

interface ToastProps {
  message: string
  type?: ToastType
  duration?: number
  onClose?: () => void
}

export function Toast({ message, type = 'info', duration = 3000, onClose }: ToastProps) {
  const [isVisible, setIsVisible] = useState(true)

  useEffect(() => {
    const timer = setTimeout(() => {
      setIsVisible(false)
      setTimeout(() => {
        if (onClose) onClose()
      }, 300) // 等待动画结束
    }, duration)

    return () => clearTimeout(timer)
  }, [duration, onClose])

  const getTypeStyles = () => {
    switch (type) {
      case 'success':
        return 'bg-success-color text-white border-success-color'
      case 'error':
        return 'bg-error-color text-white border-error-color'
      case 'warning':
        return 'bg-warning-color text-white border-warning-color'
      case 'info':
      default:
        return 'bg-accent-color text-white border-accent-color'
    }
  }

  const getIcon = () => {
    switch (type) {
      case 'success':
        return '✅'
      case 'error':
        return '❌'
      case 'warning':
        return '⚠️'
      case 'info':
      default:
        return 'ℹ️'
    }
  }

  if (!isVisible) {
    return null
  }

  return (
    <div className={`toast ${getTypeStyles()}`}>
      <div className="toast-content">
        <span className="toast-icon">{getIcon()}</span>
        <span className="toast-message">{message}</span>
      </div>
      <button
        className="toast-close"
        onClick={() => {
          setIsVisible(false)
          if (onClose) setTimeout(onClose, 300)
        }}
        aria-label="关闭通知"
      >
        ✕
      </button>
    </div>
  )
}
