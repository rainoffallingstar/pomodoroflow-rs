import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useKeyboardShortcuts } from "../hooks/useKeyboardShortcuts";
import { invoke } from "@tauri-apps/api/tauri";

vi.mock("@tauri-apps/api/tauri", () => ({
  invoke: vi.fn(),
}));

describe("useKeyboardShortcuts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("starts pomodoro on Ctrl+Space", () => {
    const { unmount } = renderHook(() => useKeyboardShortcuts());

    act(() => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", {
          key: " ",
          code: "Space",
          ctrlKey: true,
          cancelable: true,
        }),
      );
    });

    expect(vi.mocked(invoke)).toHaveBeenCalledWith("start_pomodoro", undefined);
    unmount();
  });

  it("focuses new task input on Ctrl+N", () => {
    const { unmount } = renderHook(() => useKeyboardShortcuts());

    const input = document.createElement("input");
    input.id = "new-task-input";
    document.body.appendChild(input);
    const focusSpy = vi.spyOn(input, "focus");

    act(() => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", {
          key: "n",
          code: "KeyN",
          ctrlKey: true,
        }),
      );
    });

    expect(focusSpy).toHaveBeenCalled();
    focusSpy.mockRestore();
    document.body.removeChild(input);
    unmount();
  });

  it("triggers reset command and opens settings callback", () => {
    const onOpenSettings = vi.fn();
    const { unmount } = renderHook(() => useKeyboardShortcuts(onOpenSettings));

    act(() => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "r", ctrlKey: true }),
      );
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: ",", ctrlKey: true }),
      );
    });

    expect(vi.mocked(invoke)).toHaveBeenCalledWith("reset_pomodoro", undefined);
    expect(onOpenSettings).toHaveBeenCalled();
    unmount();
  });

  it("does not trigger command for unrelated key", () => {
    const { unmount } = renderHook(() => useKeyboardShortcuts());

    act(() => {
      window.dispatchEvent(
        new KeyboardEvent("keydown", { key: "x", ctrlKey: true }),
      );
    });

    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
    unmount();
  });
});
