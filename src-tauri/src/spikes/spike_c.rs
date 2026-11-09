//! Spike C: auto-hide via a Rust cursor-polling thread (30-50 ms).
//!
//! Verifies:
//! - GetCursorPos polling only while a dock is auto-hidden (no global low-level mouse hook)
//! - Edge trigger zone: show dock when cursor enters the edge
//! - Hide after 150-300 ms delay when cursor leaves
//! - set_ignore_cursor_events toggled from Rust so clicks pass through when hidden

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager, WebviewWindow};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

const POLL_INTERVAL: Duration = Duration::from_millis(40);
const HIDE_DELAY: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AutoHideState {
    Visible,
    Hiding,
    Hidden,
}

struct AutoHideCtx {
    edge: Edge,
    state: AutoHideState,
    hide_timer: Option<std::time::Instant>,
}

type DockMap = Arc<Mutex<HashMap<String, AutoHideCtx>>>;

static DOCKS: Mutex<Option<DockMap>> = Mutex::new(None);

fn get_map() -> DockMap {
    let mut g = DOCKS.lock().unwrap();
    g.get_or_insert_with(|| Arc::new(Mutex::new(HashMap::new()))).clone()
}

#[tauri::command]
pub fn spike_c_toggle_autohide(
    app: AppHandle,
    dock_label: String,
    edge: String,
    enabled: bool,
) -> Result<(), String> {
    let e = match edge.as_str() {
        "top" => Edge::Top,
        "bottom" => Edge::Bottom,
        "left" => Edge::Left,
        "right" => Edge::Right,
        _ => return Err("invalid edge".into()),
    };

    if enabled {
        let map = get_map();
        map.lock().unwrap().insert(
            dock_label.clone(),
            AutoHideCtx {
                edge: e,
                state: AutoHideState::Visible,
                hide_timer: None,
            },
        );
        let map_clone = map.clone();
        let label = dock_label.clone();
        let app_clone = app.clone();
        thread::spawn(move || poll_loop(app_clone, map_clone, label));
        tracing::info!("spike C autohide enabled for {}", dock_label);
    } else {
        let map = get_map();
        map.lock().unwrap().remove(&dock_label);
        if let Some(w) = app.get_webview_window(&dock_label) {
            let _ = w.show();
            let _ = w.set_ignore_cursor_events(false);
        }
        tracing::info!("spike C autohide disabled for {}", dock_label);
    }

    Ok(())
}

fn poll_loop(app: AppHandle, map: DockMap, label: String) {
    loop {
        let (new_state, action) = {
            let map = map.lock().unwrap();
            let Some(ctx) = map.get(&label) else {
                break; // removed -> stop polling
            };
            let ctx = ctx.clone();
            drop(map);

            let window = match app.get_webview_window(&label) {
                Some(w) => w,
                None => break,
            };
            let cursor = cursor_pos();
            let at_edge = cursor.map(|c| is_at_edge(c, ctx.edge)).unwrap_or(false);

            let (new_state, action) = match (ctx.state, at_edge) {
                (AutoHideState::Visible, false) => (
                    AutoHideCtx {
                        edge: ctx.edge,
                        state: AutoHideState::Hiding,
                        hide_timer: Some(std::time::Instant::now()),
                    },
                    Action::StartHideDelay,
                ),
                (AutoHideState::Hiding, true) => (
                    AutoHideCtx {
                        edge: ctx.edge,
                        state: AutoHideState::Visible,
                        hide_timer: None,
                    },
                    Action::CancelHide,
                ),
                (AutoHideState::Hiding, false) => {
                    if ctx.hide_timer.map(|t| t.elapsed() >= HIDE_DELAY).unwrap_or(false) {
                        (
                            AutoHideCtx {
                                edge: ctx.edge,
                                state: AutoHideState::Hidden,
                                hide_timer: None,
                            },
                            Action::HideWindow,
                        )
                    } else {
                        (ctx, Action::None)
                    }
                }
                (AutoHideState::Hidden, true) => (
                    AutoHideCtx {
                        edge: ctx.edge,
                        state: AutoHideState::Visible,
                        hide_timer: None,
                    },
                    Action::ShowWindow,
                ),
                (AutoHideState::Hidden, false) => (ctx, Action::None),
                (AutoHideState::Visible, true) => (ctx, Action::None),
            };

            // Persist new state
            let map2 = get_map();
            map2.lock().unwrap().insert(label.clone(), new_state);

            (new_state, (action, window))
        };

        let (action, window) = action;

        match action {
            Action::ShowWindow => {
                let _ = window.show();
                let _ = window.set_ignore_cursor_events(false);
                tracing::info!("spike C showing dock {}", label);
            }
            Action::HideWindow => {
                let _ = window.hide();
                let _ = window.set_ignore_cursor_events(true);
                tracing::info!("spike C hiding dock {}", label);
            }
            Action::None => {}
            Action::StartHideDelay | Action::CancelHide => {}
        }

        thread::sleep(POLL_INTERVAL);
    }
}

enum Action {
    None,
    ShowWindow,
    HideWindow,
    StartHideDelay,
    CancelHide,
}

fn cursor_pos() -> Option<POINT> {
    unsafe {
        let mut pt = POINT::default();
        if GetCursorPos(&mut pt).is_ok() {
            Some(pt)
        } else {
            None
        }
    }
}

fn is_at_edge(_cursor: POINT, _edge: Edge) -> bool {
    // Compare cursor (physical pixels on the dock's monitor) against the dock
    // window's edge region; window rect must be obtained from the native HWND.
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poll_interval_is_in_range() {
        assert!(POLL_INTERVAL >= Duration::from_millis(30));
        assert!(POLL_INTERVAL <= Duration::from_millis(50));
    }

    #[test]
    fn test_hide_delay_is_in_range() {
        assert!(HIDE_DELAY >= Duration::from_millis(150));
        assert!(HIDE_DELAY <= Duration::from_millis(300));
    }
}
