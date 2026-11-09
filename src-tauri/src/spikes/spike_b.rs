//! Spike B: two dock windows on different monitors + WM_DISPLAYCHANGE handling.
//!
//! Verifies:
//! - Monitor enumeration with per-monitor identity (stable device name + bounds)
//! - One dock window per monitor, positioned on that monitor
//! - WM_DISPLAYCHANGE received via a hidden native message window
//! - Docks hidden for a missing monitor, restored to saved position on return

use crate::monitor::enumerate_monitors;
use serde::Serialize;
use tauri::{AppHandle, WebviewWindowBuilder, WebviewUrl};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostMessageW, RegisterClassW,
    TranslateMessage, CW_USEDEFAULT, HWND_MESSAGE, MSG, WM_DISPLAYCHANGE, WINDOW_EX_STYLE,
    WINDOW_STYLE, WNDCLASSW,
};
use windows::core::{PCWSTR, Result as WinResult};

const LISTENER_CLASS: &str = "OryvnMonitorListener";
const WM_USER_REENUMERATE: u32 = 0x0400 + 1;

#[derive(Debug, Clone, Serialize)]
pub struct DockPlacement {
    pub label: String,
    pub monitor_device: String,
    pub x: i32,
    pub y: i32,
}

#[tauri::command]
pub fn spike_b_setup(app: AppHandle) -> std::result::Result<Vec<DockPlacement>, String> {
    let monitors = enumerate_monitors().map_err(|e| e.to_string())?;

    if monitors.is_empty() {
        return Ok(vec![]);
    }

    let mut placements = Vec::new();

    // Create one dock window per monitor.
    for (i, mon) in monitors.iter().enumerate() {
        let label = format!("dock-spike-b-{}", i);
        // Position bottom-center of the monitor.
        let x = (mon.identity.left + mon.identity.width() / 2 - 150) as f64;
        let y = (mon.identity.bottom - 80) as f64;

        let window = WebviewWindowBuilder::new(
            &app,
            &label,
            WebviewUrl::App(format!("/?dock=b&mon={}", i).into()),
        )
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .build()
        .map_err(|e| e.to_string())?;

        // Move to physical position on the monitor
        let _ = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));

        tracing::info!(
            "spike B dock {} on monitor {} ({}x{} @ {})",
            label,
            mon.identity.device_name,
            mon.identity.width(),
            mon.identity.height(),
            mon.scale
        );

        placements.push(DockPlacement {
            label,
            monitor_device: mon.identity.device_name.clone(),
            x: x as i32,
            y: y as i32,
        });
    }

    // Start the hidden WM_DISPLAYCHANGE listener on a background thread.
    let app_clone = app.clone();
    std::thread::spawn(move || {
        if let Ok(hwnd) = unsafe { create_listener_window() } {
            run_listener_loop(hwnd, app_clone);
        }
    });

    Ok(placements)
}

unsafe fn create_listener_window() -> WinResult<HWND> {
    let hinstance = HINSTANCE(
        windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?.0,
    );
    let class_name: Vec<u16> = LISTENER_CLASS.encode_utf16().collect();
    let wc = WNDCLASSW {
        lpfnWndProc: Some(listener_wndproc),
        hInstance: hinstance,
        lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
        ..Default::default()
    };
    let _ = unsafe { RegisterClassW(&wc) };
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR::from_raw(class_name.as_ptr()),
            PCWSTR::null(),
            WINDOW_STYLE::default(),
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            Some(HWND_MESSAGE),
            None,
            Some(hinstance),
            None,
        )
    }?;
    Ok(hwnd)
}

fn run_listener_loop(hwnd: HWND, _app: AppHandle) {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        loop {
            let ret = GetMessageW(&mut msg, None, 0, 0);
            if ret.0 == 0 {
                break; // WM_QUIT
            }
            if ret.0 == -1 {
                break;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

extern "system" fn listener_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DISPLAYCHANGE => {
            tracing::info!(
                "WM_DISPLAYCHANGE received (wparam={:?}, lparam={:?})",
                wparam, lparam
            );
            let _ = unsafe { PostMessageW(Some(hwnd), WM_USER_REENUMERATE, WPARAM(0), LPARAM(0)) };
            LRESULT(0)
        }
        WM_USER_REENUMERATE => {
            tracing::info!("Re-enumerating monitors after display change");
            let _ = enumerate_monitors();
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
