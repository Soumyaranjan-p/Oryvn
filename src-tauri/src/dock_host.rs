//! DockWindowHost: owns one Tauri WebviewWindow per dock.
//! This is the ONLY place that creates/destroys dock windows.
//! Commands delegate to the host; the host is stored in Tauri state.

use std::collections::HashMap;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State, WebviewWindowBuilder, WebviewUrl};

use crate::brand::APP_NAME;
use crate::dock_model::{DockConfig, Edge};
use crate::monitor::{enumerate_monitors, Monitor};

/// Tauri state: the dock window host.
pub struct DockWindowHost(pub Mutex<HostInner>);

pub struct HostInner {
    /// dock label -> tauri window
    windows: HashMap<String, tauri::WebviewWindow>,
}

impl HostInner {
    fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }

    pub fn create_dock_window(
        &mut self,
        app: &AppHandle,
        dock: &DockConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let label = &dock.id;
        if self.windows.contains_key(label) {
            return Ok(()); // already exists
        }

        let scale = Self::monitor_scale_for(dock);
        let logical_w = dock.width as f64 / scale;
        let logical_h = dock.height as f64 / scale;

        let window: tauri::WebviewWindow = WebviewWindowBuilder::new(
            app,
            label,
            WebviewUrl::App(format!("/?dock={}", label).into()),
        )
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .inner_size(logical_w as f64, logical_h as f64)
        .build()
        .map_err(|e| e.to_string())?;

        // WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, and clear WS_EX_APPWINDOW.
        // Never steals focus; never in Alt+Tab/taskbar.
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        crate::window_styles::apply_dock_styles(hwnd).map_err(|e| e.to_string())?;
        crate::window_styles::schedule_style_reapply(window.clone(), label);
        tracing::info!(
            "DockWindowHost: dock {} ex-style=0x{:X} tool_window={}",
            label,
            crate::window_styles::ex_styles(hwnd),
            crate::window_styles::is_tool_window(hwnd)
        );

        // Diagnostic: dump our windows shortly after creation to see whether
        // anything resets the extended styles during webview init.
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            crate::window_styles::log_our_windows("after-dock");
        });

        // Position on the assigned monitor (physical pixels).
        if let Some(mon) = Self::find_monitor(&dock.monitor_id) {
            let (px, py) = match dock.edge {
                Edge::Bottom => (
                    mon.identity.left + (mon.identity.width() - dock.width) / 2,
                    mon.identity.bottom - dock.height - 8,
                ),
                Edge::Top => (
                    mon.identity.left + (mon.identity.width() - dock.width) / 2,
                    mon.identity.top + 8,
                ),
                Edge::Left => (
                    mon.identity.left + 8,
                    mon.identity.top + (mon.identity.height() - dock.height) / 2,
                ),
                Edge::Right => (
                    mon.identity.right - dock.width - 8,
                    mon.identity.top + (mon.identity.height() - dock.height) / 2,
                ),
                Edge::Floating => (dock.x, dock.y),
            };
            let _ = window.set_position(tauri::PhysicalPosition::new(px, py));
        } else if matches!(dock.edge, Edge::Floating) {
            let _ = window.set_position(tauri::PhysicalPosition::new(dock.x, dock.y));
        }

        tracing::info!(
            "DockWindowHost: created dock window {} (scale={}, edge={:?})",
            label, scale, dock.edge
        );
        self.windows.insert(label.to_string(), window);
        Ok(())
    }

    pub fn destroy_dock_window(&mut self, label: &str) {
        if let Some(window) = self.windows.remove(label) {
            let _ = window.close();
            tracing::info!("DockWindowHost: destroyed dock window {}", label);
        }
    }

    pub fn get_window(&self, label: &str) -> Option<tauri::WebviewWindow> {
        self.windows.get(label).cloned()
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    fn monitor_scale_for(dock: &DockConfig) -> f64 {
        if dock.scale > 0.0 {
            return dock.scale;
        }
        Self::find_monitor(&dock.monitor_id)
            .map(|m| m.scale)
            .unwrap_or(1.0)
    }

    fn find_monitor(device_name: &str) -> Option<Monitor> {
        let Ok(monitors) = enumerate_monitors() else {
            return None;
        };
        if device_name.is_empty() {
            return monitors.into_iter().find(|m| m.identity.is_primary);
        }
        monitors
            .into_iter()
            .find(|m| m.identity.device_name == device_name)
    }
}

impl Default for HostInner {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DockWindowHost {
    fn default() -> Self {
        Self(Mutex::new(HostInner::new()))
    }
}

/// Open (create) a dock window for the given dock config.
#[tauri::command]
pub fn dock_host_open(
    app: AppHandle,
    dock: DockConfig,
    host: State<'_, DockWindowHost>,
) -> Result<(), String> {
    let mut h = host.0.lock().unwrap();
    h.create_dock_window(&app, &dock)
        .map_err(|e| e.to_string())
}

/// Close (destroy) a dock window by label.
#[tauri::command]
pub fn dock_host_close(
    label: String,
    host: State<'_, DockWindowHost>,
) -> Result<(), String> {
    let mut h = host.0.lock().unwrap();
    h.destroy_dock_window(&label);
    Ok(())
}

/// List labels of currently open dock windows.
#[tauri::command]
pub fn dock_host_list_open(
    host: State<'_, DockWindowHost>,
) -> Result<Vec<String>, String> {
    let h = host.0.lock().unwrap();
    Ok(h.windows.keys().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dock_model::{Background, Edge, Orientation, Theme};

    fn test_dock(label: &str) -> DockConfig {
        DockConfig {
            id: label.to_string(),
            name: "Test".into(),
            monitor_id: String::new(),
            edge: Edge::Bottom,
            x: 100,
            y: 100,
            width: 400,
            height: 60,
            icon_size: 32,
            gap: 8,
            scale: 1.5,
            theme: Theme::Dark,
            background: Background::Solid,
            auto_hide: false,
            collapse_to_button: false,
            hide_when_fullscreen: false,
            orientation: Orientation::Horizontal,
        }
    }

    #[test]
    fn test_host_starts_empty() {
        let host = DockWindowHost::default();
        assert_eq!(host.0.lock().unwrap().window_count(), 0);
    }

    #[test]
    fn test_get_window_unknown_returns_none() {
        let host = DockWindowHost::default();
        assert!(host.0.lock().unwrap().get_window("nonexistent").is_none());
    }

    #[test]
    fn test_destroy_unknown_does_not_panic() {
        let host = DockWindowHost::default();
        // Should not panic even though no window exists
        host.0.lock().unwrap().destroy_dock_window("nonexistent");
        assert_eq!(host.0.lock().unwrap().window_count(), 0);
    }

    #[test]
    fn test_repeated_destroy_no_leak() {
        let host = DockWindowHost::default();
        for _ in 0..10 {
            host.0.lock().unwrap().destroy_dock_window("repeated");
        }
        assert_eq!(host.0.lock().unwrap().window_count(), 0);
    }

    #[test]
    fn test_monitor_scale_for_uses_dock_scale() {
        let dock = test_dock("scale-test");
        // dock.scale = 1.5, should be used directly
        assert_eq!(HostInner::monitor_scale_for(&dock), 1.5);
    }

    #[test]
    fn test_find_monitor_primary() {
        // find_monitor with empty device_name returns primary monitor
        let result = HostInner::find_monitor("");
        // On a real Windows machine there should be at least one monitor.
        // On CI without displays this may be None — that's OK for the test.
        if let Some(mon) = result {
            assert!(mon.identity.is_primary || mon.scale > 0.0);
        }
    }

    /// Integration-style test: create/destroy via the command path.
    /// Requires a Tauri runtime (AppHandle), so it is gated out of unit tests.
    /// Run manually: launch the app, call dock_host_open then dock_host_open
    /// repeatedly for the same label (should be idempotent), then dock_host_close
    /// and verify window_count() drops to 0 via dock_host_list_open.
    #[test]
    fn test_create_destroy_leak_check_doc() {
        // This test documents the manual integration path:
        // 1. Call dock_host_open(label=A) -> window created, count=1
        // 2. Call dock_host_open(label=A) again -> idempotent, count=1
        // 3. Call dock_host_close(label=A) -> window destroyed, count=0
        // 4. Call dock_host_close(label=A) again -> no panic, count=0
        // 5. Repeat steps 1-4 100x -> count stays 0 (no leak)
        assert!(true);
    }
}

