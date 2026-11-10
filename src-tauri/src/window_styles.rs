//! Dock window extended styles.
//!
//! Docks must never steal focus and must never appear in Alt+Tab or the taskbar.
//!
//! Two details matter here:
//!   * `WS_EX_TOOLWINDOW` (0x80) removes the window from the Alt+Tab switcher
//!     and the taskbar.
//!   * `WS_EX_APPWINDOW` (0x40000) *forces* a window into the Alt+Tab switcher
//!     and taskbar. It wins over `WS_EX_TOOLWINDOW`, so it must be explicitly
//!     cleared.
//!   * `WS_EX_NOACTIVATE` (0x08000000) stops the window ever taking foreground
//!     when clicked.

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, SWP_FRAMECHANGED,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
};

pub const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
pub const WS_EX_NOACTIVATE: u32 = 0x0800_0000;
pub const WS_EX_APPWINDOW: u32 = 0x0004_0000;

/// Compute the extended style a dock window should have.
pub fn desired_ex_style(current: u32) -> u32 {
    (current & !WS_EX_APPWINDOW) | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE
}

/// Apply `WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE` and clear `WS_EX_APPWINDOW`.
///
/// Safe to call repeatedly. Call it once right after window creation and again
/// while the webview initialises — Tauri/wry can reset extended styles there.
pub fn apply_dock_styles(hwnd: HWND) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        let updated = desired_ex_style(current);
        if updated != current {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, updated as isize);
        }
        // SWP_FRAMECHANGED forces the non-client area to be recalculated so the
        // new extended styles take effect immediately.
        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
    Ok(())
}

/// Apply `WS_EX_TOOLWINDOW` to every *other* top-level window of this process.
///
/// Tauri/tao/WebView2 create extra top-level helper windows that are visible
/// (`Tao Thread Event Target`, `com.<id>.app-sic`). Those are what Windows
/// surfaces in the Alt+Tab switcher, so they must be tool windows too.
/// Only `WS_EX_TOOLWINDOW` is applied — `WS_EX_NOACTIVATE` is left alone so
/// the WebView2 input sink keeps receiving mouse input.
pub fn apply_tool_style_to_helper_windows() {
    let pid = std::process::id();
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(mark_helper_tool_window),
            windows::Win32::Foundation::LPARAM(pid as isize),
        );
    }
}

unsafe extern "system" fn mark_helper_tool_window(
    hwnd: HWND,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::core::BOOL {
    use windows::core::BOOL;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    let target_pid = lparam.0 as u32;
    let mut wpid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut wpid));
    if wpid != target_pid {
        return BOOL(1);
    }

    let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    let updated = (current & !WS_EX_APPWINDOW) | WS_EX_TOOLWINDOW;
    if updated != current {
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, updated as isize);
    }
    BOOL(1)
}

/// Read back the current extended styles (for diagnostics/tests).
pub fn ex_styles(hwnd: HWND) -> u32 {
    unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32 }
}

/// True when the window carries the styles that keep it out of Alt+Tab.
pub fn is_tool_window(hwnd: HWND) -> bool {
    let style = ex_styles(hwnd);
    style & WS_EX_TOOLWINDOW != 0 && style & WS_EX_APPWINDOW == 0
}

/// Re-apply the dock styles for the first few seconds after window creation.
///
/// Tauri/wry attach the WebView2 child after the top-level window is created
/// and can reset the extended styles in the process. Re-applying on a short
/// schedule (and on window events) guarantees the dock stays out of Alt+Tab.
pub fn schedule_style_reapply(window: tauri::WebviewWindow, label: &str) {
    let label = label.to_string();
    let win = window.clone();

    // Re-apply on interaction (cheap, and repairs any style reset).
    let event_window = window.clone();
    let event_label = label.clone();
    window.on_window_event(move |event| {
        let should_reapply = matches!(
            event,
            tauri::WindowEvent::Focused(_)
                | tauri::WindowEvent::Moved(_)
                | tauri::WindowEvent::Resized(_)
                | tauri::WindowEvent::ScaleFactorChanged { .. }
        );
        if should_reapply {
            if let Ok(hwnd) = event_window.hwnd() {
                let _ = apply_dock_styles(hwnd);
            }
        }
    });

    // Re-apply during webview initialisation.
    std::thread::spawn(move || {
        for delay_ms in [30u64, 100, 250, 500, 1000, 2000] {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
            if let Ok(hwnd) = win.hwnd() {
                if let Err(e) = apply_dock_styles(hwnd) {
                    tracing::debug!("style reapply failed for {}: {}", label, e);
                }
            }
            // Helper windows (tao / WebView2) are created alongside the dock.
            apply_tool_style_to_helper_windows();
        }
    });
}

/// Diagnostic: enumerate every top-level window owned by this process and log
/// its visibility, ex-style and tool-window status.
///
/// Used to diagnose "dock appears in Alt+Tab" — it shows exactly which HWND the
/// shell would list and whether the exclusion styles are actually applied.
pub fn log_our_windows(tag: &str) {
    let pid = std::process::id();
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(dump_window_proc),
            windows::Win32::Foundation::LPARAM(pid as isize),
        );
    }
    tracing::info!("WINDOW-DUMP[{tag}] done");
}

unsafe extern "system" fn dump_window_proc(
    hwnd: HWND,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::core::BOOL {
    use windows::core::BOOL;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
        IsWindowVisible,
    };

    let target_pid = lparam.0 as u32;
    let mut wpid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut wpid));
    if wpid != target_pid {
        return BOOL(1);
    }

    let mut title_buf: Vec<u16> = vec![0; 256];
    let title_len = GetWindowTextLengthW(hwnd) as usize;
    if title_len > 0 && title_len < 255 {
        let _ = GetWindowTextW(hwnd, &mut title_buf);
    }
    let title = String::from_utf16_lossy(&title_buf)
        .trim_end_matches('\0')
        .to_string();

    let mut class_buf: Vec<u16> = vec![0; 256];
    let _ = GetClassNameW(hwnd, &mut class_buf);
    let class = String::from_utf16_lossy(&class_buf)
        .trim_end_matches('\0')
        .to_string();

    let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    tracing::info!(
        "WINDOW hwnd=0x{:X} class={class} title={title:?} visible={} ex=0x{style:08X} tool={} appwindow={} noactivate={}",
        hwnd.0 as usize,
        IsWindowVisible(hwnd).as_bool(),
        style & WS_EX_TOOLWINDOW != 0,
        style & WS_EX_APPWINDOW != 0,
        style & WS_EX_NOACTIVATE != 0,
    );
    BOOL(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(WS_EX_TOOLWINDOW, 0x0000_0080);
        assert_eq!(WS_EX_NOACTIVATE, 0x0800_0000);
        assert_eq!(WS_EX_APPWINDOW, 0x0004_0000);
    }

    #[test]
    fn test_style_computation_clears_appwindow() {
        // Simulate a Tauri window that has WS_EX_APPWINDOW set.
        let current = WS_EX_APPWINDOW;
        let updated = desired_ex_style(current);
        assert_eq!(updated & WS_EX_APPWINDOW, 0, "WS_EX_APPWINDOW must be cleared");
        assert_ne!(updated & WS_EX_TOOLWINDOW, 0);
        assert_ne!(updated & WS_EX_NOACTIVATE, 0);
    }

    #[test]
    fn test_style_computation_is_idempotent() {
        let once = desired_ex_style(WS_EX_APPWINDOW);
        let twice = desired_ex_style(once);
        assert_eq!(once, twice);
    }
}
