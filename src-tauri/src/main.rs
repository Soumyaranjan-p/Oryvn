use tauri::{generate_context, Manager, State};
use tracing_subscriber::prelude::*;

mod brand;
mod commands;
mod config;
mod dock_host;
mod dock_model;
mod icons;
mod launcher;
mod monitor;
mod spikes;
mod tray;
mod appbar;
mod autohide;
mod window_styles;
mod windows_tracker;

fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let app_data = std::env::var("APPDATA").map_err(|_| "APPDATA not set")?;
    let log_dir = std::path::Path::new(&app_data).join(crate::brand::APP_NAME).join("logs");
    std::fs::create_dir_all(&log_dir)?;
    let log_file = log_dir.join("oryvn.log");
    let file = std::fs::File::create(log_file)?;
    // Default to INFO when RUST_LOG is unset — `EnvFilter::from_default_env()`
    // silently falls back to ERROR-only, which produced an empty log.
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_writer(file),
        )
        .init();
    Ok(())
}

fn main() {
    let _ = init_logging();
    tracing::info!("{} starting", crate::brand::APP_NAME);

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {
            tracing::info!("second instance ignored");
        }))
        .plugin(tauri_plugin_notification::init())
        .manage(crate::dock_host::DockWindowHost::default())
        .invoke_handler(tauri::generate_handler![
            // spikes (debug / throwaway)
            crate::spikes::spike_a::spike_a_create_dock,
            crate::spikes::spike_b::spike_b_setup,
            #[cfg(debug_assertions)]
            crate::spikes::spike_e::spike_e_open_n_docks,
            // phase 1 commands
            crate::commands::dock_list,
            crate::commands::dock_create,
            crate::commands::dock_update,
            crate::commands::dock_delete,
            crate::commands::dock_duplicate,
            crate::commands::dock_items,
            crate::commands::item_add,
            crate::commands::item_remove,
            crate::commands::item_reorder,
            crate::commands::item_extract_icon,
            crate::commands::app_launch,
            crate::commands::apps_running,
            crate::commands::config_get,
            crate::commands::config_save,
            crate::commands::config_reset,
            crate::dock_host::dock_host_open,
            crate::dock_host::dock_host_close,
            crate::dock_host::dock_host_list_open,
            // autohide + appbar (phase 1 native)
            crate::autohide::autohide_toggle,
            crate::appbar::register_bottom_appbar,
            crate::appbar::remove_appbar,
        ])
        .setup(|app| {
            tracing::info!("{} setup complete", crate::brand::APP_NAME);
            // Startup cleanup: remove any stale appbars from a previous crash.
            let _ = crate::appbar::startup_cleanup();
            // The main (hidden) window must never appear in Alt+Tab either.
            if let Some(main_window) = app.get_webview_window("main") {
                if let Ok(hwnd) = main_window.hwnd() {
                    let _ = crate::window_styles::apply_dock_styles(hwnd);
                }
            }
            let _ = crate::tray::setup_tray(app);
            crate::window_styles::apply_tool_style_to_helper_windows();
            crate::window_styles::log_our_windows("startup");
            Ok(())
        })
        .run(generate_context!())
        .expect("error while running tauri application");
}
