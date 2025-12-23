import '../styles/App.css'

interface LoadingSpinnerProps {
  size?: 'small' | 'medium' | 'large'
  text?: string
  fullScreen?: boolean
}

export function LoadingSpinner({ size = 'medium', text, fullScreen = false }: LoadingSpinnerProps) {
  const sizeMap = {
    small: 'w-8 h-8',
    medium: 'w-12 h-12',
    large: 'w-16 h-16'
  }

  const textSizeMap = {
    small: 'text-sm',
    medium: 'text-base',
    large: 'text-lg'
  }

  const spinner = (
    <div className="flex flex-col items-center justify-center gap-3">
      <div className={`loading-spinner ${sizeMap[size]}`}></div>
      {text && (
        <p className={`loading-text ${textSizeMap[size]} text-secondary`}>
          {text}
        </p>
      )}
    </div>
  )

  if (fullScreen) {
    return (
      <div className="loading-screen">
        {spinner}
      </div>
    )
  }

  return spinner
}
