# Oryvn — Manual QA Checklist

All items below are **pending manual verification** unless marked otherwise.
Run these on a real Windows 10 22H2 / 11 machine with WebView2 runtime installed.

## Phase 0 — Risk Spike Verification

### Spike A: Transparent borderless dock window + hover magnification
| # | Check | Status | Notes |
|---|---|---|---|
| A1 | Window is borderless (`decorations:false`) | pending manual verification | |
| A2 | Window is transparent (`transparent:true`) | pending manual verification | |
| A3 | Window is always-on-top (`alwaysOnTop:true`) | pending manual verification | |
| A4 | Window does not appear in taskbar (`skipTaskbar:true`) | pending manual verification | |
| A5 | Window does not steal focus (`focus:false`, `WS_EX_NOACTIVATE`) | pending manual verification | Clicking the dock should not activate it / steal foreground |
| A6 | Window does not appear in Alt+Tab (`WS_EX_TOOLWINDOW`) | pending manual verification | Press Alt+Tab, confirm dock absent |
| A7 | No flicker on show/hide or drag | pending manual verification | |
| A8 | Shadow does not clip dock content | pending manual verification | Confirm shadow padding leaves room |
| A9 | Mica/Acrylic background on drag | pending manual verification | Confirm vibrancy behaves acceptably when dragged |
| A10 | Clicks pass through transparent areas | pending manual verification | Click behind the dock on transparent regions |
| A11 | Hover magnification animation works | pending manual verification | CSS `transform: scale()` + `opacity` |
| A12 | `html`/`body` background is transparent | pending manual verification | |

### Spike B: Multi-monitor + WM_DISPLAYCHANGE
| # | Check | Status | Notes |
|---|---|---|---|
| B1 | One dock per monitor, each on correct monitor | pending manual verification | Requires 2+ monitors |
| B2 | Per-monitor DPI scale applied correctly | pending manual verification | `GetDpiForMonitor` / MDT_EFFECTIVE_DPI |
| B3 | Mixed 100%/150% DPI geometry correct | pending manual verification | Physical pixel conversion per monitor |
| B4 | `WM_DISPLAYCHANGE` hides dock for missing monitor | pending manual verification | Disconnect a monitor |
| B5 | Dock restores to exact saved position on return | pending manual verification | Reconnect monitor |
| B6 | Stable monitor identity (device name + bounds) | pending manual verification | |

### Spike C: Auto-hide cursor polling
| # | Check | Status | Notes |
|---|---|---|---|
| C1 | Polling thread runs only while auto-hidden | pending manual verification | |
| C2 | Dock shows when cursor enters edge trigger zone | pending manual verification | 8 px zone |
| C3 | Dock hides after 150–300 ms delay when cursor leaves | pending manual verification | 200 ms default |
| C4 | `set_ignore_cursor_events` toggled correctly | pending manual verification | Clicks pass through when hidden |
| C5 | No global low-level mouse hook used | pending manual verification | `GetCursorPos` only |
| C6 | Auto-hide show/hover-to-visible latency < 100 ms | pending manual verification | Measure end-to-end |

### Spike D: AppBar docking
| # | Check | Status | Notes |
|---|---|---|---|
| D1 | `ABM_NEW` registers appbar on one edge | pending manual verification | Bottom edge of primary monitor |
| D2 | `ABM_QUERYPOS` / `ABM_SETPOS` claim edge space | pending manual verification | Maximized windows should not cover dock |
| D3 | Callback message handled via window procedure | pending manual verification | `RegisterWindowMessageA` |
| D4 | `ABM_REMOVE` on dock close | pending manual verification | |
| D5 | `ABM_REMOVE` on startup cleanup | pending manual verification | |
| D6 | Works at different DPIs (100%, 150%, 200%) | pending manual verification | |
| D7 | Cleanup after crash (stale appbar removed on restart) | pending manual verification | Kill process, restart, confirm no ghost appbar |
| D8 | Auto-hide docks NOT registered as appbars | pending manual verification | They remain plain floating windows |

### Spike E: Measurements
| # | Check | Status | Notes |
|---|---|---|---|
| E1 | Idle CPU ~0% on RELEASE build after 30s idle | pending manual measurement | |
| E2 | Memory with 1 dock | pending manual measurement | Use debug N-docks command |
| E3 | Memory with 3 docks | pending manual measurement | Use debug N-docks command |
| E4 | Memory with 5 docks | pending manual measurement | Use debug N-docks command |
| E5 | Hover-to-visible latency < 100 ms | pending manual measurement | |
| E6 | Click-to-launch < 150 ms | pending manual measurement | |

---

## Phase 1 — Core Multi-Dock

| # | Check | Status | Notes |
|---|---|---|---|
| P1.1 | Create / delete / duplicate docks | pending | |
| P1.2 | Dock config persisted (id, name, monitor, edge, position, etc.) | pending | |
| P1.3 | Dock survives explorer.exe restart | pending | |
| P1.4 | Dock survives sleep/resume | pending | |
| P1.5 | Dock survives display change / RDP | pending | |
| P1.6 | Items: apps, folders, files, URLs, separators | pending | |
| P1.7 | Drag-and-drop files/folders onto dock to add | pending | |
| P1.8 | Reorder items by dragging | pending | |
| P1.9 | Icon extraction + disk cache (keyed by path+mtime) | pending | |
| P1.10 | Running-app indicators + click-to-focus/launch | pending | |
| P1.11 | Tray menu: new dock, settings, pause all, quit | pending | |
| P1.12 | Settings window: dock list, per-dock options, live preview, theme editor | pending | |
| P1.13 | Single instance (second launch is no-op) | pending | |
| P1.14 | Launch-at-login toggle | pending | |
| P1.15 | Config: versioned, atomic writes, corrupt-config recovery | pending | |
| P1.16 | Keyboard navigation inside dock + ARIA labels | pending | |
| P1.17 | `prefers-reduced-motion` respected | pending | |
| P1.18 | Windows high-contrast respected | pending | |

---

## Environment notes

- Target: Windows 10 22H2 / Windows 11, x64, WebView2 runtime.
- App data: `%APPDATA%\Oryvn\config.json`, logs: `%APPDATA%\Oryvn\logs`.
- App name single source of truth: `APP_NAME = "Oryvn"` (`src-tauri/src/brand.rs`, `src/brand.ts`).
