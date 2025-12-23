import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ThemeToggle } from "../components/ThemeToggle";
import { useAppStore } from "../stores/appStore";

// Mock the app store
vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

describe("ThemeToggle", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the theme toggle component", () => {
    const mockStore = {
      theme: "light",
      toggleTheme: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<ThemeToggle />);

    expect(screen.getByRole("button")).toBeInTheDocument();
  });

  it("displays sun emoji for light theme", () => {
    const mockStore = {
      theme: "light",
      toggleTheme: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<ThemeToggle />);

    const icon = screen.getByText("☀️");
    expect(icon).toBeInTheDocument();
  });

  it("displays moon emoji for dark theme", () => {
    const mockStore = {
      theme: "dark",
      toggleTheme: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<ThemeToggle />);

    const icon = screen.getByText("🌙");
    expect(icon).toBeInTheDocument();
  });

  it("displays system emoji for system theme", () => {
    const mockStore = {
      theme: "system",
      toggleTheme: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<ThemeToggle />);

    const icon = screen.getByText("💻");
    expect(icon).toBeInTheDocument();
  });

  it("calls toggleTheme when clicked", () => {
    const mockStore = {
      theme: "light",
      toggleTheme: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<ThemeToggle />);

    const themeButton = screen.getByRole("button");
    fireEvent.click(themeButton);

    expect(mockStore.toggleTheme).toHaveBeenCalled();
  });

  it("displays theme label correctly", () => {
    const mockStore = {
      theme: "light",
      toggleTheme: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<ThemeToggle />);

    const label = screen.getByText("Light");
    expect(label).toBeInTheDocument();
  });

  it("applies theme class to document", () => {
    const mockStore = {
      theme: "dark",
      toggleTheme: vi.fn(),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore);

    render(<ThemeToggle />);

    const themeButton = screen.getByRole("button");
    fireEvent.click(themeButton);

    expect(mockStore.toggleTheme).toHaveBeenCalled();
  });
});
