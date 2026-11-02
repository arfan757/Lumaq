# Contributing to Lumaq

We welcome contributions to Lumaq! Whether you're fixing bugs, improving documentation, or adding support for additional platforms (Linux MPRIS, macOS NowPlaying), this guide helps you get started.

---

## 📋 Development Guidelines

- **No Remote Assets:** The frontend runs under strict CSP (`connect-src 'none'`). All fonts and icons must be local or inline SVGs.
- **Platform Separation:** Platform-specific media session code resides in `src-tauri/src/media/`. Windows uses `windows.rs`; other platforms implement the `MediaSource` trait defined in `src-tauri/src/media/mod.rs`.
- **Thread Safety:** WinRT objects must remain inside the dedicated GSMTC thread. Avoid sharing non-`Send` types across Tokio tasks.
- **Conventional Commits:** Use standard commit prefixes:
  - `feat: ...` for new features
  - `fix: ...` for bug fixes
  - `docs: ...` for documentation
  - `test: ...` for tests
  - `refactor: ...` for code refactoring
  - `style: ...` for styling / formatting changes

---

## 🧪 Testing Checklist

Before submitting a Pull Request, ensure all tests pass:

1. **Frontend typecheck & tests:**
   ```bash
   npm run build
   npm test
   ```

2. **Backend unit tests:**
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   ```

3. **Format & Lint:**
   ```bash
   cargo fmt --check --manifest-path src-tauri/Cargo.toml
   npm run lint
   ```

---

## 💡 Submitting Issues

When reporting issues, please provide:
1. Windows version (e.g. Windows 11 23H2).
2. Media player & version (e.g. Spotify Desktop 1.2.x).
3. Log output from `C:\Users\<username>\AppData\Local\com.lumaq.desktop\logs` if applicable.

