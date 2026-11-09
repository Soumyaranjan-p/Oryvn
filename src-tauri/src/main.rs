use tauri::generate_context;
use tracing_subscriber::prelude::*;

mod brand;
mod commands;
mod config;
mod monitor;
mod spikes;

fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let app_data = std::env::var("APPDATA").map_err(|_| "APPDATA not set")?;
    let app_dir = std::path::Path::new(&app_data).join(brand::APP_NAME);
    std::fs::create_dir_all(app_dir.join("logs"))?;
    let log_file = app_dir.join("logs").join("oryvn.log");
    let file = std::fs::File::create(log_file)?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
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
    tracing::info!("{} starting", brand::APP_NAME);

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {
            tracing::info!("second instance ignored");
        }))
        .invoke_handler(tauri::generate_handler![
            spikes::spike_a::spike_a_create_dock,
            spikes::spike_b::spike_b_setup,
            spikes::spike_c::spike_c_toggle_autohide
        ])
        .setup(|app| {
            tracing::info!("{} setup complete", brand::APP_NAME);
            let _ = app;
            Ok(())
        })
        .run(generate_context!())
        .expect("error while running tauri application");
}
