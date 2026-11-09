//! Tray icon + menu event handler.
//! Tray menu is configured in tauri.conf.json (tauri-build 2.7 doesn't yet
//! validate the `tray` root key; the menu below is handled programmatically
//! when the config schema is updated).
//!
//! For Phase 1, this is a stub that registers the tray event listener.

use tauri::App;

pub fn setup_tray(_app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    // Tray menu is defined in tauri.conf.json under the `tray` key.
    // tauri-build 2.7.x doesn't validate that key yet; once upgraded,
    // menu item events arrive via app.tray_on_menu_event().
    tracing::info!("tray module initialized (menu config in tauri.conf.json)");
    Ok(())
}
