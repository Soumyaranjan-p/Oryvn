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

## Phase 1 module boundaries

### Data model (pure, decoupled from windows)
- `src-tauri/src/dock_model.rs` — `DockConfig`, `DockItem`, `ItemType`, `AppConfig`. Pure data, no window references.
- `src-tauri/src/config.rs` — `AppConfig` load/save/migrations/atomic write to `%APPDATA%\Oryvn\config.json`.

### DockWindowHost layer
- `src-tauri/src/dock_host.rs` — `DockWindowHost`: owns a `HashMap<String, WebviewWindow>` mapping dock id → Tauri window. Creates dock windows lazily (`decorations:false, transparent:true, shadow:false, skipTaskbar:true, alwaysOnTop:true, focus:false` + `WS_EX_TOOLWINDOW|WS_EX_NOACTIVATE`). Destroys on dock delete. Each dock window loads `/?dock=<id>`.
- The host is the ONLY place that creates/destroys dock windows. Commands delegate to it.

### Native services
- `src-tauri/src/monitor.rs` — monitor enumeration + `GetDpiForMonitor` (MDT_EFFECTIVE_DPI) scale factors.
- `src-tauri/src/icons.rs` — `SHGetFileInfo` / `IShellItemImageFactory` extraction, disk cache keyed by path+mtime, asset URLs to frontend.
- `src-tauri/src/launcher.rs` — `ShellExecuteEx` for apps/folders/URLs, `CreateProcess` for scripts with args.
- `src-tauri/src/windows_tracker.rs` — `EnumWindows` + process queries for running-app tracking; group multiple windows per app.
- `src-tauri/src/appbar.rs` — `SHAppBarMessage` wrapping (ABM_NEW/QUERYPOS/SETPOS/REMOVE), dedicated appbar window per edge.
- `src-tauri/src/autohide.rs` — cursor polling thread (30–50 ms) with edge trigger zone, show/hide delay, `set_ignore_cursor_events`.
- `src-tauri/src/tray.rs` — tray icon + menu (new dock, settings, pause all, quit).
- `src-tauri/src/notify_pipe.rs` — named pipe `\\.\pipe\oryvn` notification endpoint (Phase 3).

### Commands / events
- `src-tauri/src/commands.rs` — all Tauri command handlers (dock CRUD, item CRUD, launch, etc.).
- `src/tauri.ts` — typed TS contracts mirroring the Rust side.

### Frontend
- `src/components/Dock.tsx` — renders inside a dock window (`/?dock=<id>`): items, hover magnification, tooltip labels.
- `src/components/DockItem.tsx` — single item: icon, label, drag handle, context menu.
- `src/components/Settings.tsx` — settings window (lazy, destroyed on close): dock list, per-dock editor, live preview, theme editor.
- `src/App.tsx` — main (invisible) window; entry point for spikes/debug.

## Window model (Phase 1)
- One dock = one Tauri window sized to the dock. Created lazily by `DockWindowHost`.
- Dock window flags: `decorations:false`, `transparent:true`, `shadow:false`, `skipTaskbar:true`, `alwaysOnTop:true`, `focus:false`.
- `WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE` from Rust — never steals focus, never Alt+Tab/taskbar.
- Per-monitor DPI: physical pixels, scale via `Monitor.to_physical`/`to_logical`.
- Rounded corners, shadows, padding in CSS inside the transparent window; leave shadow padding. `html`/`body` transparent.
- Animate only `transform` and `opacity`. Spring-style hover magnification via GPU-friendly transforms.

## Config model (Phase 1)
```json
{
  "version": 1,
  "docks": [
    {
      "id": "...",
      "name": "Main",
      "monitorId": "\\\\.\\DISPLAY1",
      "edge": "bottom",
      "x": 100, "y": 1040,
      "iconSize": 32, "gap": 8, "scale": 1,
      "theme": "dark",
      "background": "solid",
      "autoHide": false,
      "collapseToButton": false,
      "hideWhenFullscreen": false,
      "orientation": "horizontal"
    }
  ],
  "items": { "<dockId>": [ { "id": "...", "type": "app", "label": "VS Code", "path": "...", "order": 0 } ] }
}
```

## Quality bar
- `cargo fmt`, `clippy -D warnings`, ESLint + Prettier, strict TypeScript.
- Docks never steal focus, never in Alt+Tab/taskbar, survive explorer restart, sleep/resume, display changes, RDP.
- Accessibility: keyboard nav inside dock, ARIA labels, high-contrast respect.
- Tests: config migration, geometry/DPI conversion, autohide state machine. Manual QA checklist in `docs/QA.md`.

## Phase status
- Phase 0: complete (conditional GO). Compile-verified; live A–D verification pending in QA.
- Phase 1: in progress.
