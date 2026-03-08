import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { act } from "react";
import { SettingsPanel } from "../components/SettingsPanel";
import { useAppStore } from "../stores/appStore";

vi.mock("../stores/appStore", () => ({
  useAppStore: vi.fn(),
}));

describe("SettingsPanel", () => {
  const baseConfig = {
    github_token_encrypted: "github_pat_xxxxxxxxxxxx",
    github_username: "octocat",
    last_sync_cursor: null,
    selected_project_owner: "acme",
    selected_project_repo: "pomoflow-rs",
    selected_project_number: 1,
    pomodoro_work_duration: 1500,
    pomodoro_short_break_duration: 300,
    pomodoro_long_break_duration: 900,
    pomodoro_cycles_until_long_break: 4,
    notifications_enabled: true,
    sound_enabled: true,
    theme: "light",
  };

  it("runs github sync check in dry-run mode", async () => {
    const user = userEvent.setup();
    const mockStore = {
      userConfig: baseConfig,
      saveUserConfig: vi.fn(async () => {}),
      loadUserConfig: vi.fn(async () => {}),
      runGithubSync: vi.fn(async () => ({
        dry_run: true,
        pending_items: 1,
        supported_items: 1,
        unsupported_items: 0,
        invalid_items: 0,
        target: { owner: "acme", repo: "pomoflow-rs", project_number: 1 },
        errors: [],
      })),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore as any);
    render(<SettingsPanel />);

    await act(async () => {
      await user.click(screen.getByRole("button", { name: "检查 GitHub 同步队列" }));
    });

    expect(mockStore.runGithubSync).toHaveBeenCalledWith(true);
  });

  it("runs github sync in real mode", async () => {
    const user = userEvent.setup();
    const mockStore = {
      userConfig: baseConfig,
      saveUserConfig: vi.fn(async () => {}),
      loadUserConfig: vi.fn(async () => {}),
      runGithubSync: vi.fn(async () => ({
        dry_run: false,
        pending_items: 1,
        supported_items: 1,
        unsupported_items: 0,
        invalid_items: 0,
        target: { owner: "acme", repo: "pomoflow-rs", project_number: 1 },
        errors: [],
      })),
    };

    vi.mocked(useAppStore).mockReturnValue(mockStore as any);
    render(<SettingsPanel />);

    await act(async () => {
      await user.click(screen.getByRole("button", { name: "执行 GitHub 同步" }));
    });

    expect(mockStore.runGithubSync).toHaveBeenCalledWith(false);
  });
});
