# Oryvn Architecture

**App name (single source of truth):** `APP_NAME = "Oryvn"` — defined in Rust (`src-tauri/src/brand.rs`) and TS (`src/brand.ts`). Never hardcoded elsewhere.

## Stack
- Tauri v2, Rust (stable) backend, React + TypeScript + Vite + Tailwind CSS frontend.
- `windows` crate for Win32, `window-vibrancy` for Mica/Acrylic, `serde`/`serde_json` for config, `tokio` for async, `tracing` for logging.
- Target: Windows 10 22H2 / Windows 11, x64. WebView2 runtime.
- Config: JSON in `%APPDATA%\Oryvn\config.json` with `version` + migrations, atomic writes (temp + rename), never crash on corrupt config (backup + defaults).
- Logs: `%APPDATA%\Oryvn\logs`.
- Single instance (tauri single-instance plugin). Optional launch-at-login.

## Layering (invariant)
- **Rust owns all native work:** window creation/styling, monitor enumeration, AppBar docking, cursor polling for auto-hide, running-app tracking, icon extraction/caching, launching, config persistence, notification endpoint (named pipe).
- **React owns only rendering:** dock UI, animations, settings window, themes.
- **Frontend talks to Rust via Tauri commands and events only.** Typed command/event contracts live in one file on each side: `src-tauri/src/commands.rs` (Rust) and `src/tauri.ts` (TS). No direct FFI from JS.

## Window model
- One dock = one small Tauri window sized to the dock (never full-screen overlays). Docks created lazily. Settings window created only when opened, destroyed on close.
- Dock window flags: `decorations:false`, `transparent:true`, `shadow:false`, `skipTaskbar:true`, `alwaysOnTop:true`, `focus:false`.
- Apply `WS_EX_TOOLWINDOW` and `WS_EX_NOACTIVATE` from Rust so docks never steal focus or appear in Alt+Tab.
- Declare per-monitor DPI awareness v2 (manifest). All native geometry in physical pixels; convert via each monitor's scale factor.
- Rounded corners, shadows, padding drawn in CSS inside the transparent window; leave shadow padding so nothing clips. `html`/`body` background transparent.
- Animate only `transform` and `opacity`. No layout animation. Spring-style hover magnification via GPU-friendly transforms (`prefers-reduced-motion` respected).

## Per-phase layout (Phase 0 = risk spikes)
Phase 0 lives under `spikes/` (throwaway / feature-flagged). Each spike A–E is verified independently, measured, then reported before Phase 1 begins.

Phase 1+ module map (planned, not yet implemented):
- `src-tauri/src/main.rs` — Tauri entry, single-instance, menu/tray
- `src-tauri/src/brand.rs` — `APP_NAME` constant
- `src-tauri/src/config.rs` — config load/save/migrations/atomic write
- `src-tauri/src/monitor.rs` — monitor enumeration + DPI scale factors
- `src-tauri/src/dock_window.rs` — per-dock Tauri window creation
- `src-tauri/src/appbar.rs` — SHAppBarMessage wrapping
- `src-tauri/src/autohide.rs` — cursor polling thread + edge trigger
- `src-tauri/src/icons.rs` — SHGetFileInfo/IShellItemImageFactory extraction + disk cache
- `src-tauri/src/launcher.rs` — app/folder/URL/script launching
- `src-tauri/src/tray.rs` — tray icon + menu
- `src-tauri/src/notify_pipe.rs` — named pipe notification endpoint
- `src/tauri.ts` — typed TS command/event contracts
- `src/components/Dock.tsx` — dock rendering + hover magnification
- `src/components/Settings.tsx` — settings window
- `src/brand.ts` — `APP_NAME` constant

## Quality bar
- `cargo fmt`, `clippy -D warnings`, ESLint + Prettier, strict TypeScript.
- Docks never steal focus, never in Alt+Tab/taskbar, survive explorer restart, sleep/resume, display changes, RDP.
- Accessibility: keyboard nav inside dock, ARIA labels, high-contrast respect.
- Tests: config migration, geometry/DPI conversion, workflow runner. Manual QA checklist in `docs/QA.md`.
