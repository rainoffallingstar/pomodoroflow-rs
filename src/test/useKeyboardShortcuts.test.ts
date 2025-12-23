import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useKeyboardShortcuts } from "../hooks/useKeyboardShortcuts";

// Mock Tauri API
vi.mock("@tauri-apps/api/tauri", () => ({
  invoke: vi.fn(),
}));

describe("useKeyboardShortcuts", () => {
  let mockInvoke: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    vi.clearAllMocks();
    vi.clearAllTimers();

    // Get the mocked invoke function
    const { invoke } = require("@tauri-apps/api/tauri");
    mockInvoke = invoke as ReturnType<typeof vi.fn>;
  });

  afterEach(() => {
    // Clean up event listeners
    window.removeEventListener("keydown", expect.any(Function));
  });

  it("should register Ctrl+Space to toggle pomodoro", () => {
    const { unmount } = renderHook(() => useKeyboardShortcuts());

    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: " ",
        code: "Space",
        ctrlKey: true,
      });
      window.dispatchEvent(event);
    });

    expect(mockInvoke).toHaveBeenCalledWith("toggle_pomodoro");

    unmount();
  });

  it("should register Ctrl+N to focus new task input", () => {
    const { unmount } = renderHook(() => useKeyboardShortcuts());

    // Create a mock input element
    const mockInput = document.createElement("input");
    mockInput.id = "new-task-input";
    document.body.appendChild(mockInput);

    const focusSpy = vi.spyOn(mockInput, "focus");

    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: "n",
        code: "KeyN",
        ctrlKey: true,
      });
      window.dispatchEvent(event);
    });

    expect(focusSpy).toHaveBeenCalled();

    focusSpy.mockRestore();
    document.body.removeChild(mockInput);
    unmount();
  });

  it("should register Ctrl+R to reset pomodoro", () => {
    const { unmount } = renderHook(() => useKeyboardShortcuts());

    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: "r",
        code: "KeyR",
        ctrlKey: true,
      });
      window.dispatchEvent(event);
    });

    expect(mockInvoke).toHaveBeenCalledWith("reset_pomodoro");

    unmount();
  });

  it("should register Ctrl+, to open settings", () => {
    const { unmount } = renderHook(() => useKeyboardShortcuts());

    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: ",",
        code: "Comma",
        ctrlKey: true,
      });
      window.dispatchEvent(event);
    });

    expect(mockInvoke).toHaveBeenCalledWith("open_settings");

    unmount();
  });

  it("should register Cmd+Space on macOS to toggle pomodoro", () => {
    const { unmount } = renderHook(() => useKeyboardShortcuts());

    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: " ",
        code: "Space",
        metaKey: true,
      });
      window.dispatchEvent(event);
    });

    expect(mockInvoke).toHaveBeenCalledWith("toggle_pomodoro");

    unmount();
  });

  it("should not trigger shortcuts without Ctrl or Cmd", () => {
    const { unmount } = renderHook(() => useKeyboardShortcuts());

    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: "n",
        code: "KeyN",
        ctrlKey: false,
        metaKey: false,
      });
      window.dispatchEvent(event);
    });

    expect(mockInvoke).not.toHaveBeenCalled();

    unmount();
  });

  it("should not trigger shortcuts with wrong key", () => {
    const { unmount } = renderHook(() => useKeyboardShortcuts());

    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: "x",
        code: "KeyX",
        ctrlKey: true,
      });
      window.dispatchEvent(event);
    });

    expect(mockInvoke).not.toHaveBeenCalled();

    unmount();
  });

  it("should prevent default behavior for shortcuts", () => {
    const { unmount } = renderHook(() => useKeyboardShortcuts());

    const preventDefaultSpy = vi.fn();

    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: " ",
        code: "Space",
        ctrlKey: true,
        preventDefault: preventDefaultSpy,
      } as any);
      window.dispatchEvent(event);
    });

    expect(preventDefaultSpy).toHaveBeenCalled();

    unmount();
  });

  it("should clean up event listeners on unmount", () => {
    const addEventListenerSpy = vi.spyOn(window, "addEventListener");
    const removeEventListenerSpy = vi.spyOn(window, "removeEventListener");

    const { unmount } = renderHook(() => useKeyboardShortcuts());

    expect(addEventListenerSpy).toHaveBeenCalledWith(
      "keydown",
      expect.any(Function),
    );

    unmount();

    expect(removeEventListenerSpy).toHaveBeenCalledWith(
      "keydown",
      expect.any(Function),
    );

    addEventListenerSpy.mockRestore();
    removeEventListenerSpy.mockRestore();
  });
});
