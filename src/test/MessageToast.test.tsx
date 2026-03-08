import { describe, it, expect, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { MessageToast } from "../components/MessageToast";
import { useAppStore } from "../stores/appStore";

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

describe("MessageToast", () => {
  it("renders friendly error code label with message", () => {
    const clearMessages = vi.fn();
    vi.mocked(useAppStore).mockReturnValue({
      error: "请求超时",
      errorCode: "NETWORK",
      success: null,
      clearMessages,
    } as any);

    render(<MessageToast />);

    expect(screen.getByText("网络错误")).toBeInTheDocument();
    expect(screen.getByText("请求超时")).toBeInTheDocument();
  });

  it("falls back to unknown label for unmapped code", () => {
    vi.mocked(useAppStore).mockReturnValue({
      error: "未知异常",
      errorCode: "X_CUSTOM",
      success: null,
      clearMessages: vi.fn(),
    } as any);

    render(<MessageToast />);

    expect(screen.getByText("未知错误")).toBeInTheDocument();
  });

  it("calls clearMessages when close clicked", () => {
    const clearMessages = vi.fn();
    vi.mocked(useAppStore).mockReturnValue({
      error: "输入不合法",
      errorCode: "VALIDATION",
      success: null,
      clearMessages,
    } as any);

    render(<MessageToast />);
    fireEvent.click(screen.getByRole("button", { name: "关闭" }));
    expect(clearMessages).toHaveBeenCalledTimes(1);
  });
});
