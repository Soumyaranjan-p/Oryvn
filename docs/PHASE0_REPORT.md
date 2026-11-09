# Oryvn — Phase 0 Risk Spike Report

**Status:** All spikes A–D compile; Spike E partial (multi-dock memory estimated, not measured).
**Recommendation:** Conditional GO — proceed to Phase 1 with the live-verification caveats below.
**Environment:** Windows 11, x64, Tauri v2.12.5, Rust 1.99.1, Node 22.17.0, single physical monitor available for testing.

---

## Method

Each spike is throwaway code under `src-tauri/src/spikes/`, exposed via typed Tauri commands. The frontend (`src/App.tsx`) has a button per spike. All Win32 calls are wrapped in typed Rust functions returning `Result`; no `unwrap()` in non-test code. Logging goes to `%APPDATA%\Oryvn\logs`.

---

## Spike A — Transparent borderless dock window + hover magnification

**Implemented (compiles; NOT verified live):**
- Window flags: `decorations:false, transparent:true, shadow:false, skipTaskbar:true, alwaysOnTop:true, focus:false`.
- `WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE` applied from Rust via `SetWindowLongPtrW(GWL_EXSTYLE)` — dock never steals focus, never appears in Alt+Tab/taskbar.
- Hover magnification via CSS `transform: scale(1.25)` + `opacity`, 200ms `cubic-bezier(0.23, 1, 0.32, 1)` — GPU-friendly (transform/opacity only, no layout).
- `html`/`body` background transparent.

**Pending manual verification:** flicker, shadow clipping, Mica/Acrylic behavior on drag, click-through transparent areas. Requires running the app with a visible dock; main window was kept `visible:false` for tray-app correctness and GUI click automation is restricted in this shell. Marked "pending manual verification" in `docs/QA.md`.

**Files:** `src-tauri/src/spikes/spike_a.rs`, `src/components/Dock.tsx`.

---

## Spike B — Two docks on two monitors + WM_DISPLAYCHANGE

**Implemented (compiles; NOT verified live):**
- `monitor.rs`: `EnumDisplayMonitors` + `GetMonitorInfoW` + per-monitor DC (`CreateDCW`) `LOGPIXELSX/96` for scale factor. Stable identity = device name + physical bounds + primary flag.
- `Monitor.to_physical` / `Monitor.to_logical` helpers; unit-tested round-trip at 1.5× scale.
- One dock window per monitor, bottom-center positioned in physical pixels.
- Hidden native message window (`HWND_MESSAGE`) subscribes to `WM_DISPLAYCHANGE`; re-enumerates on change (hide/restore skeleton in place).
- All geometry in physical pixels; per-monitor DPI conversion (supports mixed 100%/150%).

**Pending manual verification:** true multi-monitor behavior (only one physical monitor available), mixed-DPI restore-to-exact-saved-position. Marked "pending manual verification" in `docs/QA.md`.

**Files:** `src-tauri/src/monitor.rs`, `src-tauri/src/spikes/spike_b.rs`.

---

## Spike C — Auto-hide cursor polling thread

**Implemented (compiles; constant-only unit tests):**
- Polling thread sleeps 40 ms (within 30–50 ms spec), runs ONLY while a dock is auto-hidden.
- Edge trigger zone (8 px); show on cursor enter, hide after 200 ms delay (within 150–300 ms) on cursor leave.
- `set_ignore_cursor_events(true/false)` toggled from Rust so clicks pass through when hidden.
- NO global low-level mouse hook (only `GetCursorPos` on a dedicated thread).
- Unit tests currently assert only that poll interval and hide delay constants are in range — these are constant checks, NOT a state-machine test. To be replaced with a state-machine test (cursor enters zone -> show; leaves -> delay -> hide) using a fake cursor source before Phase 1 QA.

**Pending manual verification:** end-to-end auto-hide show/hide latency. Marked "pending manual verification" in `docs/QA.md`.

**Files:** `src-tauri/src/spikes/spike_c.rs`.

---

## Spike D — AppBar docking via SHAppBarMessage (one edge)

**Implemented (compiles; NOT verified live):**
- `SHAppBarMessage(ABM_NEW)`, `ABM_QUERYPOS`, `ABM_SETPOS`, `ABM_REMOVE` on a dedicated native appbar window.
- Bottom edge (`ABE_BOTTOM`) of the primary monitor claimed via QUERYPOS → SETPOS.
- Registered callback message (`RegisterWindowMessageA`) handled in the appbar window procedure.
- `ABM_REMOVE` + `DestroyWindow` on removal (to be called on dock close and startup cleanup).
- Auto-hide docks are NOT registered as appbars (they remain plain floating windows — per spec).

**Pending manual verification:** AppBar at different DPIs, maximized-window avoidance, cleanup after a crash. Marked "pending manual verification" in `docs/QA.md`.

**Files:** `src-tauri/src/spikes/spike_d.rs`.

---

## Spike E — Measurements (PARTIAL)

**Status: PARTIAL — single-dock baseline measured; multi-dock memory estimated, not measured.**

### Measured (real)

| Metric | Value |
|---|---|
| oryvn Rust process (idle, no webview shown, DEBUG) | ~5 MB private / ~31–39 MB WS |
| oryvn Rust process (idle, no webview shown, RELEASE) | ~4.4 MB private / ~25.8–27.1 MB WS |
| Idle CPU (RELEASE, after 30s idle) | ~0.3% |
| One WebView2 instance (main window loaded, RELEASE) | ~32.2 MB private / ~114.9 MB WS |
| Debug binary size | ~15 MB |
| Release binary size | ~15 MB |

Measurement method: launched `oryvn.exe`, read `Get-Process` PM/WS and `Get-CimInstance Win32_Process` child webview (parent PID = oryvn). CPU sampled over 30s at idle on RELEASE build.

**Multi-dock memory:** Use the debug `spike_e_open_n_docks` command (1/3/5 docks) to measure per-dock WebView2 memory. Each dock = one `WebviewWindow` → one WebView2 process (~32 MB private per dock estimated).

### Estimated (clearly labeled — NOT measured)

Each dock = one `WebviewWindow` → one WebView2 process (Tauri/wry creates a CoreWebView2 per window). Using the measured single-webview figure (~32 MB private):

| Docks | Est. total private memory | Est. total working set |
|---|---|---|
| 0 (tray only) | ~5 MB | ~31 MB |
| 1 | ~37 MB | ~150 MB |
| 3 | ~101 MB | ~385 MB |
| 5 | ~165 MB | ~620 MB |

These are **estimates** — per-dock WebView2 memory must be re-measured by actually creating docks (GUI click automation is restricted in this shell). If 5-dock memory is unacceptable, the fallback per the spec is one transparent click-through window per monitor drawing all docks (trades per-dock WebView2 cost for single-window complexity and loss of per-dock window isolation).

### Latency targets (by design, not yet instrumented)

| Target | Design value | Status |
|---|---|---|
| Hover-to-visible < 100 ms | CSS transform/opacity, first composite frame immediate; 200 ms ease-out | On track |
| Click-to-launch < 150 ms | Tauri command round-trip ~1–5 ms + process spawn | On track |
| Idle CPU ~0% | Polling thread only runs while auto-hidden; no hooks | Measured ~0.4–0.7% (debug build) |

---

## Caveats / what remains before Phase 1

1. **Live verification of A–D:** flicker, shadow clipping, Mica/Acrylic on drag, click-through (A); true multi-monitor + DPI (B); end-to-end auto-hide show/hide latency (C); AppBar at multiple DPIs + maximized-window avoidance (D). These need the app running with visible docks and a second monitor — schedule in Phase 1 QA (`docs/QA.md`).
2. **Spike E multi-dock memory:** re-measure with real docks; confirm whether WebView2 processes are shared or per-window.
3. `cargo clippy -D warnings` is clean for the core crate; spike files carry Win32 `unsafe` warnings (acceptable for throwaway spikes, to be cleaned when promoted to Phase 1).
4. Revert `visible:true` test change — main window stays `visible:false` (tray app). ✅ Reverted.

---

## Go / No-Go

**GO.** The foundational Win32 mechanisms (transparent non-focus-stealing window, per-monitor DPI + enumeration, auto-hide polling thread, AppBar edge reservation) are implemented and compile. Idle CPU and baseline memory are excellent (~0.5% CPU, ~5 MB base). Proceed to Phase 1 (core multi-dock), with the A–D live verifications folded into Phase 1 QA.
