//! Tray icon + menu event handler.
//! Creates the system tray icon using Tauri v2's TrayIconBuilder and muda menu.
//! Wires "New Dock" to the DockWindowHost so clicking creates a real dock window.

use tauri::{App, AppHandle, Manager};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

use std::sync::atomic::{AtomicU32, Ordering};
use crate::dock_model::{Background, DockConfig, Edge, Orientation, Theme};

static DOCK_COUNTER: AtomicU32 = AtomicU32::new(0);

fn new_dock_config() -> DockConfig {
    let n = DOCK_COUNTER.fetch_add(1, Ordering::Relaxed);
    DockConfig {
        id: format!("dock-{}", n),
        name: "New Dock".into(),
        monitor_id: String::new(), // primary / auto
        edge: Edge::Bottom,
        x: 100,
        y: 100,
        width: 400,
        height: 60,
        icon_size: 32,
        gap: 8,
        scale: 1.0,
        theme: Theme::System,
        background: Background::Solid,
        auto_hide: false,
        collapse_to_button: false,
        hide_when_fullscreen: false,
        orientation: Orientation::Horizontal,
    }
}

pub fn setup_tray(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.app_handle().clone();

    let menu = Menu::new(&app_handle)?;

    let new_dock = MenuItem::with_id(&app_handle, "new-dock", "New Dock", true, None::<&str>)?;
    menu.append(&new_dock)?;

    let settings = MenuItem::with_id(&app_handle, "settings", "Settings", true, None::<&str>)?;
    menu.append(&settings)?;

    let pause = MenuItem::with_id(&app_handle, "pause", "Pause All Docks", true, None::<&str>)?;
    menu.append(&pause)?;

    let quit = MenuItem::with_id(&app_handle, "quit", "Quit", true, None::<&str>)?;
    menu.append(&quit)?;

    let app_handle_clone = app_handle.clone();
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .on_menu_event(move |_app, event| {
            let id = event.id();
            match id.as_ref() {
                "new-dock" => {
                    let cfg = new_dock_config();
                    let host = app_handle_clone.state::<crate::dock_host::DockWindowHost>();
                    let mut guard = host.0.lock().unwrap();
                    if let Err(e) = guard.create_dock_window(&app_handle_clone, &cfg) {
                        tracing::error!("Failed to create dock: {}", e);
                    } else {
                        tracing::info!("Tray: New Dock created ({})", cfg.id);
                    }
                }
                "settings" => tracing::info!("Tray: Settings requested"),
                "pause" => tracing::info!("Tray: Pause All Docks requested"),
                "quit" => {
                    tracing::info!("Tray: Quit requested");
                    std::process::exit(0);
                }
                _ => tracing::info!("Tray: unknown menu item {:?}", id),
            }
        })
        .build(&app_handle)?;

    Ok(())
}
