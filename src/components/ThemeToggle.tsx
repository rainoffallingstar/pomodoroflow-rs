import { useAppStore } from '../stores/appStore'
import '../styles/ThemeToggle.css'

export function ThemeToggle() {
  const { theme, toggleTheme } = useAppStore()

  const getThemeIcon = () => {
    switch (theme) {
      case 'light':
        return '☀️'
      case 'dark':
        return '🌙'
      case 'system':
        return '💻'
      default:
        return '💻'
    }
  }

  const getThemeLabel = () => {
    switch (theme) {
      case 'light':
        return 'Light'
      case 'dark':
        return 'Dark'
      case 'system':
        return 'System'
      default:
        return 'System'
    }
  }

  return (
    <button
      className="theme-toggle"
      onClick={toggleTheme}
      title={`Current theme: ${getThemeLabel()}. Click to switch.`}
    >
      <span className="theme-icon">{getThemeIcon()}</span>
      <span className="theme-label">{getThemeLabel()}</span>
    </button>
  )
}
