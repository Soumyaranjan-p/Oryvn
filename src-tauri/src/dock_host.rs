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

        // WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE — never steals focus, never Alt+Tab/taskbar.
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        unsafe {
            let current = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                hwnd,
                windows::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE,
            ) as u32;
            let updated = current | 0x0000_0080 | 0x0800_0000;
            windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                hwnd,
                windows::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE,
                updated as isize,
            );
        }

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
