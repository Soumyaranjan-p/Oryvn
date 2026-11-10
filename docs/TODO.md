# Phase 0 Risk Spikes — COMPLETE (conditional GO)
# Phase 1 Core Multi-Dock — BACKEND IMPLEMENTED, FRONTEND NOT STARTED

## Phase 0 — ALL DONE
- [x] Spike A: compiles; live verification pending
- [x] Spike B: compiles; live verification pending
- [x] Spike C: compiles; state-machine tests pass (35 total tests)
- [x] Spike D: compiles; live verification pending
- [x] Spike E: RELEASE build measured (~0.3% CPU idle, ~4.4 MB base / ~32 MB per webview)
- [x] docs/PHASE0_REPORT.md corrected (compile-only vs live, partial Spike E)
- [x] Debug N-docks command (`spike_e_open_n_docks`) behind #[cfg(debug_assertions)] — absent from release
- [x] Scale detection switched to GetDpiForMonitor (MDT_EFFECTIVE_DPI)
- [x] Spike C state-machine tests with fake clock
- [x] docs/QA.md created with pending manual verification entries
- [x] WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE verified in code (no TOOLWORLD typo)
- [x] DockWindowHost create/destroy leak-detection tests added
- [x] Appbar remove/cleanup safe-when-empty tests added

## Phase 1 — CORE MULTI-DOCK

### Backend (IMPLEMENTED)
- [x] Pure data model (`dock_model.rs`): DockConfig, DockItem, ItemType, AppConfig
- [x] Config persistence (`config.rs`): versioned, atomic write, corrupt-config recovery
- [x] DockWindowHost layer (`dock_host.rs`): manages WebviewWindow per dock, WS_EX_TOOLWINDOW|NOACTIVATE
- [x] Monitor enumeration + GetDpiForMonitor (`monitor.rs`)
- [x] Icon extraction + disk cache (`icons.rs`)
- [x] Launcher (`launcher.rs`): apps, folders, files, URLs, AUMID
- [x] Running-app tracking (`windows_tracker.rs`): EnumWindows + process queries
- [x] AppBar docking (`appbar.rs`): ABM_NEW/QUERYPOS/SETPOS/REMOVE
- [x] Auto-hide (`autohide.rs`): polling thread, state machine, unit-tested
- [x] Tray event handler (`tray.rs`)
- [x] All commands registered in main.rs
- [x] Unit tests: 35 passing (config, DPI, autohide state machine, appbar, dock host leak checks)

### Frontend (NOT STARTED)
- [ ] Settings window (lazy, destroyed on close)
- [ ] Dock list / per-dock editor UI
- [ ] Drag-and-drop items onto dock
- [ ] Reorder items by dragging
- [ ] Running-app indicators
- [ ] Per-dock window rendering (Dock.tsx loads ?dock=<id>)
- [ ] Live preview in settings

### Live Verification (PENDING — await user test results)
- [ ] Spike A: flicker, shadow clipping, Mica/Acrylic on drag, click-through, focus/Alt+Tab
- [ ] Spike B: multi-monitor, mixed-DPI restore-to-saved-position
- [ ] Spike C: end-to-end auto-hide show/hide latency
- [ ] Spike D: AppBar at different DPIs, maximized-window avoidance, crash cleanup
- [ ] Spike E multi-dock: measure 1/3/5 dock memory via spike_e_open_n_docks

**Status: Backend builds clean (debug + release), 35 tests pass. Waiting for live test results before starting Phase 1 frontend.**
