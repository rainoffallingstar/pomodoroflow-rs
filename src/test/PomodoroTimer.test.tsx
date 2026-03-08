import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { PomodoroTimer } from "../components/PomodoroTimer";
import { useAppStore } from "../stores/appStore";

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

const baseStore = {
  startPomodoro: vi.fn(),
  pausePomodoro: vi.fn(),
  resetPomodoro: vi.fn(),
  skipPomodoroPhase: vi.fn(),
  getSelectedTodo: vi.fn(() => null),
  userConfig: null,
};

describe("PomodoroTimer", () => {
  it("renders timer and phase", () => {
    vi.mocked(useAppStore).mockReturnValue({
      ...baseStore,
      pomodoroSession: {
        phase: "work",
        duration: 1500,
        remaining: 1500,
        is_running: false,
        cycle_count: 0,
      },
    } as any);

    render(<PomodoroTimer />);

    expect(screen.getByText("Pomodoro Timer")).toBeInTheDocument();
    expect(screen.getByText("25:00")).toBeInTheDocument();
    expect(screen.getByText("Work")).toBeInTheDocument();
  });

  it("disables start when running", () => {
    vi.mocked(useAppStore).mockReturnValue({
      ...baseStore,
      pomodoroSession: {
        phase: "work",
        duration: 1500,
        remaining: 1499,
        is_running: true,
        cycle_count: 0,
      },
    } as any);

    render(<PomodoroTimer />);
    expect(screen.getByRole("button", { name: /Start/i })).toBeDisabled();
  });

  it("calls start on click", () => {
    const startPomodoro = vi.fn();
    vi.mocked(useAppStore).mockReturnValue({
      ...baseStore,
      startPomodoro,
      pomodoroSession: {
        phase: "work",
        duration: 1500,
        remaining: 1500,
        is_running: false,
        cycle_count: 0,
      },
    } as any);

    render(<PomodoroTimer />);
    fireEvent.click(screen.getByRole("button", { name: /Start/i }));
    expect(startPomodoro).toHaveBeenCalledTimes(1);
  });

  it("renders progress helper element", () => {
    vi.mocked(useAppStore).mockReturnValue({
      ...baseStore,
      pomodoroSession: {
        phase: "work",
        duration: 1500,
        remaining: 750,
        is_running: false,
        cycle_count: 0,
      },
    } as any);

    render(<PomodoroTimer />);
    const progressElement = document.querySelector(".timer-progress") as HTMLElement;
    expect(progressElement).toBeInTheDocument();
    expect(progressElement.style.width).toBe("50%");
  });
});
