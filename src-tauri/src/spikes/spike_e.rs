//! Spike E: debug-only helper to open N docks (1/3/5) for memory measurement.
//! Entirely absent from release builds (gated on `debug_assertions`).
//!
//! This lets the user measure per-dock WebView2 memory (private bytes + working set)
//! with 1, 3 and 5 docks, and re-measure idle CPU after 30s idle on a RELEASE build.

use tauri::{AppHandle, WebviewWindow, WebviewWindowBuilder, WebviewUrl};

/// Open N dock windows for memory measurement. Debug-only.
#[cfg(debug_assertions)]
#[tauri::command]
pub fn spike_e_open_n_docks(app: AppHandle, count: u32) -> Result<Vec<String>, String> {
    let mut labels = Vec::new();
    for i in 0..count {
        let label = format!("dock-spike-e-{}", i);
        let _window: WebviewWindow = WebviewWindowBuilder::new(
            &app,
            &label,
            WebviewUrl::App(format!("/?dock=e&n={}", i).into()),
        )
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()
        .map_err(|e| e.to_string())?;

        let hwnd = _window.hwnd().map_err(|e| e.to_string())?;
        crate::window_styles::apply_dock_styles(hwnd).map_err(|e| e.to_string())?;
        crate::window_styles::schedule_style_reapply(_window.clone(), &label);

        tracing::info!("spike E opened dock {}", label);
        labels.push(label);
    }
    Ok(labels)
}

#[cfg(debug_assertions)]
fn apply_non_focus_styles(hwnd: windows::Win32::Foundation::HWND) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let current = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE,
        ) as u32;
        let updated = current | 0x0000_0080 | 0x0800_0000; // TOOLWINDOW | NOACTIVATE
        windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
            hwnd,
            windows::Win32::UI::WindowsAndMessaging::GWL_EXSTYLE,
            updated as isize,
        );
    }
    Ok(())
}
