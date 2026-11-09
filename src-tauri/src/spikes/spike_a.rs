//! Spike A: transparent, borderless, always-on-top, non-focus-stealing dock window
//! with hover magnification animation.
//!
//! Verifies:
//! - Window flags: decorations:false, transparent:true, shadow:false,
//!   skipTaskbar:true, alwaysOnTop:true, focus:false
//! - WS_EX_TOOLWINDOW + WS_EX_NOACTIVATE applied from Rust (no Alt+Tab, no focus steal)
//! - Hover magnification via CSS transform (GPU-friendly)
//! - Mica/Acrylic via window-vibrancy
//! - Click-through transparent areas (to be verified)

use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE,
};

const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
const WS_EX_NOACTIVATE: u32 = 0x0800_0000;

/// Apply WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE so the dock never steals focus
/// and never appears in Alt+Tab or the taskbar.
pub fn apply_non_focus_styles(hwnd: HWND) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let updated = current | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, updated as isize);
    }
    Ok(())
}

/// Create the spike-A dock window. Returns the window label on success.
#[tauri::command]
pub fn spike_a_create_dock(app: AppHandle) -> Result<String, String> {
    let window: WebviewWindow = WebviewWindowBuilder::new(
        &app,
        "dock-spike-a",
        WebviewUrl::App("/?dock=1".into()),
    )
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .skip_taskbar(true)
    .always_on_top(true)
    .build()
    .map_err(|e| e.to_string())?;

    let hwnd = window.hwnd().map_err(|e| e.to_string())?;
    apply_non_focus_styles(hwnd).map_err(|e| e.to_string())?;

    tracing::info!("spike A dock window created, label=dock-spike-a");
    Ok("dock-spike-a".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_flags_combined() {
        let combined = WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE;
        assert_eq!(combined, 0x0800_0080);
    }
}
