# Repository Guidelines

## Project Structure & Module Organization
- Root crate: `tui-journal` (binary `tjournal`) with source in `src/`.
- UI and app logic live under `src/app/` (state, keymaps, editor, popups, themes).
- CLI parsing and subcommands are in `src/cli/`.
- Settings/config handling is in `src/settings/`; startup/logging in `src/main.rs` and `src/logging.rs`.
- Backend implementations are in `backend/src/` (JSON and SQLite), exposed as the `backend` library target from root `Cargo.toml`.
- Tests are split between integration tests in `backend/tests/` and app-level tests in `src/app/test/`.
- Assets/docs: `assets/` (gifs), `THEMES.md`, `README.md`.

## Build, Test, and Development Commands
- `cargo check` - fast compile check for default features.
- `cargo check --no-default-features -F json` - validate JSON-only build.
- `cargo check --no-default-features -F sqlite` - validate SQLite-only build.
- `cargo test` - run all tests.
- `cargo clippy --all-targets --all-features` - lint code before pushing.
- `cargo build --release` - produce optimized binary.
- `make cargo_check`, `make run_test`, `make clippy`, `make build-release` - convenience wrappers used by maintainers.

## Coding Style & Naming Conventions
- Rust edition: 2024, minimum Rust: `1.85.0`.
- Use standard Rust formatting (`cargo fmt`) and keep Clippy warnings addressed.
- Naming: `snake_case` for files/functions/modules, `PascalCase` for types/traits, `UPPER_SNAKE_CASE` for constants.
- Keep modules focused; place UI behavior under `src/app/ui/*` and backend/data access under `backend/src/*`.

## Testing Guidelines
- Prefer focused inline `#[cfg(test)]` unit tests next to pure logic modules.
- Use `src/app/test/*` for broader app-level behavior and state-transition tests.
- Keep backend persistence and storage behavior in integration tests under `backend/tests/*`.
- Name tests by behavior, e.g. `undo_redo_restores_previous_entry_state`.
- Do not modify production code for testability without explicit approval.
- Do not add new dependencies for tests without explicit approval.
- Keep test comments minimal; add them only when the fixture or setup is intentionally non-obvious.
