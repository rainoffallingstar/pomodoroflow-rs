import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { PomodoroTimer } from "../components/PomodoroTimer";
import { useAppStore } from "../stores/appStore";

// Mock the store
vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

describe("PomodoroTimer", () => {
  it("renders the timer component", () => {
    const mockStore = {
      pomodoroSession: {
        phase: "work",
        duration: 1500,
        remaining: 1500,
        is_running: false,
        cycle_count: 0,
      },
      startPomodoro: vi.fn(),
      pausePomodoro: vi.fn(),
      resetPomodoro: vi.fn(),
      skipPomodoroPhase: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<PomodoroTimer />);

    expect(screen.getByText("Pomodoro Timer")).toBeInTheDocument();
    expect(screen.getByText("25:00")).toBeInTheDocument();
    expect(screen.getByText("Work")).toBeInTheDocument();
  });

  it("displays the correct time format", () => {
    const mockStore = {
      pomodoroSession: {
        phase: "work",
        duration: 1500,
        remaining: 900,
        is_running: false,
        cycle_count: 0,
      },
      startPomodoro: vi.fn(),
      pausePomodoro: vi.fn(),
      resetPomodoro: vi.fn(),
      skipPomodoroPhase: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<PomodoroTimer />);

    expect(screen.getByText("15:00")).toBeInTheDocument();
  });

  it("shows different phases correctly", () => {
    const mockStore = {
      pomodoroSession: {
        phase: "short_break",
        duration: 300,
        remaining: 300,
        is_running: false,
        cycle_count: 1,
      },
      startPomodoro: vi.fn(),
      pausePomodoro: vi.fn(),
      resetPomodoro: vi.fn(),
      skipPomodoroPhase: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<PomodoroTimer />);

    expect(screen.getByText("Short Break")).toBeInTheDocument();
  });

  it("disables start button when running", () => {
    const mockStore = {
      pomodoroSession: {
        phase: "work",
        duration: 1500,
        remaining: 1499,
        is_running: true,
        cycle_count: 0,
      },
      startPomodoro: vi.fn(),
      pausePomodoro: vi.fn(),
      resetPomodoro: vi.fn(),
      skipPomodoroPhase: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<PomodoroTimer />);

    const startButton = screen.getByText("Start");
    expect(startButton).toBeDisabled();
  });

  it("enables start button when not running", () => {
    const mockStore = {
      pomodoroSession: {
        phase: "work",
        duration: 1500,
        remaining: 1500,
        is_running: false,
        cycle_count: 0,
      },
      startPomodoro: vi.fn(),
      pausePomodoro: vi.fn(),
      resetPomodoro: vi.fn(),
      skipPomodoroPhase: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<PomodoroTimer />);

    const startButton = screen.getByText("Start");
    expect(startButton).not.toBeDisabled();
  });

  it("calls startPomodoro when start button is clicked", () => {
    const mockStore = {
      pomodoroSession: {
        phase: "work",
        duration: 1500,
        remaining: 1500,
        is_running: false,
        cycle_count: 0,
      },
      startPomodoro: vi.fn(),
      pausePomodoro: vi.fn(),
      resetPomodoro: vi.fn(),
      skipPomodoroPhase: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<PomodoroTimer />);

    const startButton = screen.getByText("Start");
    fireEvent.click(startButton);

    expect(mockStore.startPomodoro).toHaveBeenCalledTimes(1);
  });

  it("displays progress percentage", () => {
    const mockStore = {
      pomodoroSession: {
        phase: "work",
        duration: 1500,
        remaining: 750,
        is_running: false,
        cycle_count: 0,
      },
      startPomodoro: vi.fn(),
      pausePomodoro: vi.fn(),
      resetPomodoro: vi.fn(),
      skipPomodoroPhase: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<PomodoroTimer />);

    // Progress should be 50% (750 / 1500)
    const progressElement = document.querySelector(
      ".timer-progress",
    ) as HTMLElement;
    expect(progressElement).toBeInTheDocument();
    expect(progressElement.style.width).toBe("50%");
  });
});
