//! AppBar docking via SHAppBarMessage.
//! Registers a reserved edge so maximized windows don't cover the dock.
//! Auto-hide docks are NOT registered as appbars.

use tauri::AppHandle;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Shell::{APPBARDATA, SHAppBarMessage};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassW, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSW,
};
use windows::core::{PCSTR, PCWSTR};

use crate::monitor::{enumerate_monitors, Monitor};

const ABM_NEW: u32 = 0x00000000;
const ABM_REMOVE: u32 = 0x00000002;
const ABM_QUERYPOS: u32 = 0x00000003;
const ABM_SETPOS: u32 = 0x00000004;
const ABE_BOTTOM: u32 = 0x00000003;

static APPBAR_HWND: std::sync::Mutex<Option<isize>> = std::sync::Mutex::new(None);

fn register_appbar(edge: u32, monitor: &Monitor) -> Result<HWND, String> {
    let hinstance: HINSTANCE = HINSTANCE(
        unsafe {
            windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
        }
        .map_err(|e| e.to_string())?
        .0,
    );

    let class_name: Vec<u16> = "OryvnAppBar\0".encode_utf16().collect();
    let wc = WNDCLASSW {
        lpfnWndProc: Some(appbar_wndproc),
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
            0,
            0,
            monitor.identity.width(),
            40,
            None,
            None,
            Some(hinstance),
            None,
        )
    }
    .map_err(|e| e.to_string())?;

    let mut abd = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        uCallbackMessage: 0,
        uEdge: edge,
        rc: RECT {
            left: monitor.identity.left,
            top: monitor.identity.top,
            right: monitor.identity.right,
            bottom: monitor.identity.bottom,
        },
        lParam: LPARAM(0),
    };
    let _ = unsafe { SHAppBarMessage(ABM_NEW, &mut abd) };

    let mut query = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        uCallbackMessage: 0,
        uEdge: edge,
        rc: match edge {
            ABE_BOTTOM => RECT {
                left: monitor.identity.left,
                top: monitor.identity.bottom - 40,
                right: monitor.identity.right,
                bottom: monitor.identity.bottom,
            },
            _ => RECT {
                left: monitor.identity.left,
                top: monitor.identity.top,
                right: monitor.identity.right,
                bottom: monitor.identity.top + 40,
            },
        },
        lParam: LPARAM(0),
    };
    let _ = unsafe { SHAppBarMessage(ABM_QUERYPOS, &mut query) };
    let _ = unsafe { SHAppBarMessage(ABM_SETPOS, &mut query) };

    *APPBAR_HWND.lock().unwrap() = Some(hwnd.0 as isize);
    Ok(hwnd)
}

/// Register a bottom-edge appbar on the primary monitor.
#[tauri::command]
pub fn register_bottom_appbar() -> Result<(), String> {
    let monitors = enumerate_monitors().map_err(|e| e.to_string())?;
    let primary = monitors
        .iter()
        .find(|m| m.identity.is_primary)
        .ok_or_else(|| "no primary monitor".to_string())?;
    let _hwnd = register_appbar(ABE_BOTTOM, primary)?;
    tracing::info!("appbar registered on bottom edge");
    Ok(())
}

/// Remove the appbar and destroy its window.
#[tauri::command]
pub fn remove_appbar() -> Result<(), String> {
    let opt = APPBAR_HWND.lock().unwrap().take().map(|p| HWND(p as *mut _));
    if let Some(hwnd) = opt {
        let mut abd = APPBARDATA {
            cbSize: std::mem::size_of::<APPBARDATA>() as u32,
            hWnd: hwnd,
            uCallbackMessage: 0,
            uEdge: 0,
            rc: RECT::default(),
            lParam: LPARAM(0),
        };
        let _ = unsafe { SHAppBarMessage(ABM_REMOVE, &mut abd) };
        let _ = unsafe { windows::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd) };
        tracing::info!("appbar removed");
    }
    Ok(())
}

/// Call on startup to remove any stale appbar left by a crash.
pub fn startup_cleanup() -> Result<(), String> {
    remove_appbar()
}

extern "system" fn appbar_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appbar_constants() {
        assert_eq!(ABM_NEW, 0);
        assert_eq!(ABM_REMOVE, 2);
        assert_eq!(ABM_QUERYPOS, 3);
        assert_eq!(ABM_SETPOS, 4);
        assert_eq!(ABE_BOTTOM, 3);
    }
}
