//! Auto-hide via a Rust cursor-polling thread (30–50 ms).
//! No global low-level mouse hook — only GetCursorPos on a dedicated thread.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::{POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

use crate::dock_host::DockWindowHost;

const POLL_INTERVAL: Duration = Duration::from_millis(40);
const HIDE_DELAY: Duration = Duration::from_millis(200);
const EDGE_TRIGGER_PX: i32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoHideState {
    Visible,
    Hiding,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoHideAction {
    None,
    ShowWindow,
    HideWindow,
    StartHideDelay,
    CancelHide,
}

#[derive(Debug, Clone, Copy)]
pub struct AutoHideCtx {
    pub edge: Edge,
    pub state: AutoHideState,
    pub hide_timer: Option<Instant>,
}

/// Pure state transition for the auto-hide state machine.
pub fn autohide_transition(
    ctx: AutoHideCtx,
    at_edge: bool,
    now: Instant,
) -> (AutoHideCtx, AutoHideAction) {
    match (ctx.state, at_edge) {
        (AutoHideState::Visible, false) => (
            AutoHideCtx {
                edge: ctx.edge,
                state: AutoHideState::Hiding,
                hide_timer: Some(now),
            },
            AutoHideAction::StartHideDelay,
        ),
        (AutoHideState::Hiding, true) => (
            AutoHideCtx {
                edge: ctx.edge,
                state: AutoHideState::Visible,
                hide_timer: None,
            },
            AutoHideAction::CancelHide,
        ),
        (AutoHideState::Hiding, false) => {
            if ctx.hide_timer.map(|t| now.duration_since(t) >= HIDE_DELAY).unwrap_or(false) {
                (
                    AutoHideCtx {
                        edge: ctx.edge,
                        state: AutoHideState::Hidden,
                        hide_timer: None,
                    },
                    AutoHideAction::HideWindow,
                )
            } else {
                (ctx, AutoHideAction::None)
            }
        }
        (AutoHideState::Hidden, true) => (
            AutoHideCtx {
                edge: ctx.edge,
                state: AutoHideState::Visible,
                hide_timer: None,
            },
            AutoHideAction::ShowWindow,
        ),
        (AutoHideState::Hidden, false) => (ctx, AutoHideAction::None),
        (AutoHideState::Visible, true) => (ctx, AutoHideAction::None),
    }
}

pub fn is_at_edge(cursor: POINT, edge: Edge, dock_rect: RECT) -> bool {
    match edge {
        Edge::Top => cursor.y >= dock_rect.top && cursor.y < dock_rect.top + EDGE_TRIGGER_PX,
        Edge::Bottom => {
            cursor.y > dock_rect.bottom - EDGE_TRIGGER_PX && cursor.y <= dock_rect.bottom
        }
        Edge::Left => cursor.x >= dock_rect.left && cursor.x < dock_rect.left + EDGE_TRIGGER_PX,
        Edge::Right => {
            cursor.x > dock_rect.right - EDGE_TRIGGER_PX && cursor.x <= dock_rect.right
        }
    }
}

type DockMap = Arc<Mutex<HashMap>>;

type HashMap = std::collections::HashMap<String, AutoHideCtx>;

static AUTOHIDE: Mutex<Option<DockMap>> = Mutex::new(None);

fn get_map() -> DockMap {
    let mut g = AUTOHIDE.lock().unwrap();
    g.get_or_insert_with(|| Arc::new(Mutex::new(HashMap::new()))).clone()
}

#[tauri::command]
pub fn autohide_toggle(
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
        tracing::info!("autohide enabled for {}", dock_label);
    } else {
        let map = get_map();
        map.lock().unwrap().remove(&dock_label);
        if let Some(w) = app.get_webview_window(&dock_label) {
            let _ = w.show();
            let _ = w.set_ignore_cursor_events(false);
        }
        tracing::info!("autohide disabled for {}", dock_label);
    }

    Ok(())
}

fn poll_loop(app: AppHandle, map: DockMap, label: String) {
    loop {
        let (action, window) = {
            let map = map.lock().unwrap();
            let Some(ctx) = map.get(&label) else {
                break;
            };
            let ctx = (*ctx).clone();
            drop(map);

            let window = match app.get_webview_window(&label) {
                Some(w) => w,
                None => break,
            };
            let cursor = cursor_pos();
            let at_edge = cursor
                .map(|c| {
                    let dock_rect = RECT::default();
                    is_at_edge(c, ctx.edge, dock_rect)
                })
                .unwrap_or(false);

            let (new_state, action) = autohide_transition(ctx, at_edge, Instant::now());

            let map2 = get_map();
            map2.lock().unwrap().insert(label.clone(), new_state);

            (action, window)
        };

        match action {
            AutoHideAction::ShowWindow => {
                let _ = window.show();
                let _ = window.set_ignore_cursor_events(false);
                tracing::info!("autohide showing dock {}", label);
            }
            AutoHideAction::HideWindow => {
                let _ = window.hide();
                let _ = window.set_ignore_cursor_events(true);
                tracing::info!("autohide hiding dock {}", label);
            }
            AutoHideAction::None => {}
            AutoHideAction::StartHideDelay | AutoHideAction::CancelHide => {}
        }

        thread::sleep(POLL_INTERVAL);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(state: AutoHideState, timer: Option<Instant>) -> AutoHideCtx {
        AutoHideCtx {
            edge: Edge::Bottom,
            state,
            hide_timer: timer,
        }
    }

    #[test]
    fn test_visible_cursor_leaves_edge_starts_hiding() {
        let now = Instant::now();
        let (new_ctx, action) = autohide_transition(ctx(AutoHideState::Visible, None), false, now);
        assert_eq!(new_ctx.state, AutoHideState::Hiding);
        assert_eq!(action, AutoHideAction::StartHideDelay);
    }

    #[test]
    fn test_hiding_cursor_returns_cancels() {
        let now = Instant::now();
        let (new_ctx, action) = autohide_transition(ctx(AutoHideState::Hiding, Some(now)), true, now);
        assert_eq!(new_ctx.state, AutoHideState::Visible);
        assert_eq!(action, AutoHideAction::CancelHide);
    }

    #[test]
    fn test_hiding_delay_elapsed_hides() {
        let now = Instant::now();
        let timer = now - Duration::from_millis(250);
        let (new_ctx, action) = autohide_transition(ctx(AutoHideState::Hiding, Some(timer)), false, now);
        assert_eq!(new_ctx.state, AutoHideState::Hidden);
        assert_eq!(action, AutoHideAction::HideWindow);
    }

    #[test]
    fn test_hidden_cursor_enters_shows() {
        let now = Instant::now();
        let (new_ctx, action) = autohide_transition(ctx(AutoHideState::Hidden, None), true, now);
        assert_eq!(new_ctx.state, AutoHideState::Visible);
        assert_eq!(action, AutoHideAction::ShowWindow);
    }

    #[test]
    fn test_poll_interval_in_range() {
        assert!(POLL_INTERVAL >= Duration::from_millis(30));
        assert!(POLL_INTERVAL <= Duration::from_millis(50));
    }

    #[test]
    fn test_hide_delay_in_range() {
        assert!(HIDE_DELAY >= Duration::from_millis(150));
        assert!(HIDE_DELAY <= Duration::from_millis(300));
    }
}
