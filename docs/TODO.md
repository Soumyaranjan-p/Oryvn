# Phase 0 Risk Spikes — COMPLETE (conditional GO)
# Phase 1 Core Multi-Dock — IMPLEMENTED

## Phase 0 — ALL DONE
- [x] Spike A: compiles; live verification pending
- [x] Spike B: compiles; live verification pending
- [x] Spike C: compiles; state-machine tests pass (27 total tests)
- [x] Spike D: compiles; live verification pending
- [x] Spike E: RELEASE build measured (~0.3% CPU idle, ~4.4 MB base / ~32 MB per webview)
- [x] docs/PHASE0_REPORT.md corrected (compile-only vs live, partial Spike E)
- [x] Debug N-docks command added (`spike_e_open_n_docks`)
- [x] Scale detection switched to GetDpiForMonitor (MDT_EFFECTIVE_DPI)
- [x] Spike C state-machine tests with fake clock
- [x] docs/QA.md created with pending manual verification entries

## Phase 1 — CORE MULTI-DOCK (IMPLEMENTED)
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
- [x] Frontend: tauri.ts contracts, Dock.tsx component, brand.ts

## NEXT STEPS (not yet done)
- [ ] Frontend UI to create/delete/duplicate docks from settings
- [ ] Settings window (lazy, destroyed on close)
- [ ] Drag-and-drop items onto dock, reorder by dragging
- [ ] Running-app indicators wired to frontend
- [ ] Per-dock window rendering (Dock.tsx loads ?dock=<id>)
- [ ] Live multi-dock memory measurement (use spike_e_open_n_docks)
- [ ] Live A–D verification on real hardware with 2+ monitors

**Status: Build green, 27 tests pass, release build measured. Ready for Phase 1 frontend work.**
