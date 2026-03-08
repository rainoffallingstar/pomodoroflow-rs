# Repository Guidelines

## Project Structure & Module Organization
This repository is a Tauri desktop app with a React/TypeScript frontend and Rust backend/core logic.

- `src/`: React app (`components/`, `stores/`, `hooks/`, `styles/`, tests in `src/test/`).
- `src-tauri/`: Tauri host application and command handlers (`src/commands/`).
- `src/` (Rust modules): shared core/domain logic (`core/`, `storage/`, `async_utils/`, `utils/`) exposed by `src/lib.rs`.
- `migrations/`: SQLite schema migrations.
- `docs/`: architecture, requirements, setup, and active context notes.

## Build, Test, and Development Commands
- `npm install`: install frontend and Tauri JS tooling.
- `npm run dev`: run Vite frontend only.
- `npm run tauri:dev`: run full desktop app (frontend + Tauri backend).
- `npm run build`: type-check and build frontend assets.
- `npm run tauri:build`: build desktop bundles.
- `npm run test` or `npm run test:run`: run Vitest suite.
- `npm run test:coverage`: generate V8 coverage (text/json/html).
- `cargo test`: run Rust unit/integration tests.
- `cargo clippy --all-targets --all-features`: Rust lint pass before PRs.

## Coding Style & Naming Conventions
- TypeScript/React: 2-space indentation, functional components, `PascalCase` components (`PomodoroTimer.tsx`), `camelCase` hooks/utilities (`useKeyboardShortcuts.ts`).
- Rust: follow `rustfmt` defaults (4 spaces), `snake_case` modules/functions, `CamelCase` types.
- Keep command handlers thin in `src-tauri/src/commands/`; move business rules into Rust core modules.
- Use descriptive CSS file names aligned to component names (for example `TodoList.css`).

## Testing Guidelines
- Frontend tests use Vitest + Testing Library in `src/test/`, with file names `*.test.ts` or `*.test.tsx`.
- Rust tests follow current patterns like `*_test.rs` under module directories.
- Add/adjust tests for new behavior, especially timer state transitions, todo status changes, and store updates.
- Run `npm run test:coverage` and `cargo test` before opening a PR.

## Commit & Pull Request Guidelines
- Follow Conventional Commit style seen in history (for example `feat: ...`).
- Keep commits focused and atomic; separate UI, core logic, and migration changes when possible.
- PRs should include: summary, rationale, test evidence (commands run), and linked issue/task.
- Include screenshots or short recordings for UI/UX changes.
- Note migration or config impacts explicitly in the PR description.
