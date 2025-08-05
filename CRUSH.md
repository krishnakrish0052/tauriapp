# CRUSH.md

Project: MockMate Desktop (Tauri v2 + Rust 2021 + minimal JS)

Build/Run/Test
- Dev app: npm run dev (starts Tauri dev with hot reload)
- Build app: npm run build (tauri build)
- Preview static (if used): npm run preview
- Rust check: cargo check --manifest-path src-tauri/Cargo.toml
- Rust build: cargo build --manifest-path src-tauri/Cargo.toml
- Rust tests (all): cargo test --manifest-path src-tauri/Cargo.toml
- Rust single test: cargo test --manifest-path src-tauri/Cargo.toml <test_name>
- Lint Rust: cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
- Format Rust: cargo fmt --manifest-path src-tauri/Cargo.toml
- Node deps install: npm i

Code Style (Rust)
- Edition 2021; use anyhow for error propagation (anyhow::Result) and thiserror if custom errors are added.
- Prefer Result<T, anyhow::Error>; bubble with ?; avoid unwrap/expect in app code.
- Use tracing/log crate levels via log macros (error!, warn!, info!, debug!, trace!). No sensitive data in logs.
- Module imports: group std, external, then crate; use absolute crate paths; no wildcard imports.
- Formatting via rustfmt (cargo fmt); run clippy and fix lints before commit.
- Concurrency: use tokio (async/await); prefer spawn for background; handle JoinError.

Code Style (JS/TS frontend)
- Use ES modules; organize imports: node modules, then local; no default export unless required.
- Prefer TypeScript if added; strict types; avoid any; use async/await with try/catch and typed errors.
- Formatting via Prettier if introduced; keep functions small and pure.

Tauri/Project Conventions
- Commands live in src-tauri/src; keep API-safe, serialize with serde; avoid blocking in async.
- Network: use reqwest with timeouts; validate URLs; no secrets in repo; read env at runtime.
- Audio/websocket in dedicated modules; keep state in parking_lot Mutex/RwLock; document public fns via rustdoc.

Cursor/Copilot rules
- No Cursor/Copilot rule files detected. If added later (.cursor/rules/, .cursorrules, .github/copilot-instructions.md), mirror key rules here.
