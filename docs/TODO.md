# Phase 0 Risk Spikes

- [x] Architect: sketch module boundaries, data flow, command/event contracts; write docs/ARCHITECTURE.md
- [x] Bootstrap Tauri v2 project (Rust + React + Vite + Tailwind) under Oryvn/
- [ ] Spike A: transparent borderless always-on-top non-focus-stealing dock window with hover magnification animation; report flicker, shadow clipping, Mica/Acrylic on drag, click-through transparent areas
- [ ] Spike B: two dock windows on two monitors; WM_DISPLAYCHANGE handling; hide/restore on monitor connect/disconnect
- [ ] Spike C: auto-hide cursor polling thread (30-50ms) with edge trigger zone, show/hide delay, set_ignore_cursor_events toggle; no global low-level mouse hook
- [ ] Spike D: AppBar docking via SHAppBarMessage (ABM_NEW/QUERYPOS/SETPOS/REMOVE, window subclass callback) for one edge; verify at different DPIs; ABM_REMOVE on close and startup cleanup
- [ ] Spike E: measure memory (1/3/5 docks), hover-to-visible latency, idle CPU; report vs targets (<100ms hover, <150ms click-to-launch, ~0% idle CPU)
- [ ] Write Phase 0 risk spike report with numbers and go/no-go recommendation; STOP and wait for user confirmation
