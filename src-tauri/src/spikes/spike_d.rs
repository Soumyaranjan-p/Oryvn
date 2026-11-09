//! Spike D: AppBar docking via SHAppBarMessage for ONE edge.
//!
//! Verifies:
//! - ABM_NEW / ABM_QUERYPOS / ABM_SETPOS / ABM_REMOVE
//! - Callback message handled through a window procedure
//! - Maximized windows respect the reserved edge space (ABM_QUERYPOS/SETPOS)
//! - ABM_REMOVE on close and on startup cleanup
//! - Auto-hide docks are NOT registered as appbars (plain floating windows)

use std::sync::Mutex;
use tauri::AppHandle;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::Shell::{APPBARDATA, SHAppBarMessage};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassW, RegisterWindowMessageA, WINDOW_EX_STYLE,
    WINDOW_STYLE,
};
use windows::core::PCSTR;

use crate::monitor::enumerate_monitors;

// AppBar messages (shell32 / shlobj_core.h).
const ABM_NEW: u32 = 0x00000000;
const ABM_REMOVE: u32 = 0x00000002;
const ABM_QUERYPOS: u32 = 0x00000003;
const ABM_SETPOS: u32 = 0x00000004;
const ABE_BOTTOM: u32 = 0x00000003;

const APPBAR_CLASS: &str = "OryvnAppBar\0";

static APPBAR_HWND: Mutex<Option<usize>> = Mutex::new(None);
static CALLBACK_MSG: Mutex<Option<u32>> = Mutex::new(None);

fn callback_msg() -> u32 {
    let mut g = CALLBACK_MSG.lock().unwrap();
    if let Some(m) = *g {
        return m;
    }
    let m = unsafe {
        RegisterWindowMessageA(PCSTR::from_raw(b"OryvnAppBarCallback\0".as_ptr()))
    };
    *g = Some(m);
    m
}

#[tauri::command]
pub fn spike_d_register_appbar() -> Result<String, String> {
    let monitors = enumerate_monitors().map_err(|e| e.to_string())?;
    let primary = monitors
        .iter()
        .find(|m| m.identity.is_primary)
        .ok_or_else(|| "no primary monitor".to_string())?;

    let hinstance = HINSTANCE(
        unsafe {
            windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
        }
        .map_err(|e| e.to_string())?
        .0,
    );

    let class_name: Vec<u16> = APPBAR_CLASS.encode_utf16().collect();
    let wc = windows::Win32::UI::WindowsAndMessaging::WNDCLASSW {
        lpfnWndProc: Some(appbar_wndproc),
        hInstance: hinstance,
        lpszClassName: windows::core::PCWSTR::from_raw(class_name.as_ptr()),
        ..Default::default()
    };
    let _ = unsafe { RegisterClassW(&wc) };

    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            windows::core::PCWSTR::from_raw(class_name.as_ptr()),
            windows::core::PCWSTR::null(),
            WINDOW_STYLE::default(),
            0,
            0,
            primary.identity.width(),
            40,
            None,
            None,
            Some(hinstance),
            None,
        )
        .map_err(|e| e.to_string())?
    };
    if hwnd.is_invalid() {
        return Err("failed to create appbar window".into());
    }

    let _ = unsafe {
        SHAppBarMessage(
            ABM_NEW,
            &mut APPBARDATA {
                cbSize: std::mem::size_of::<APPBARDATA>() as u32,
                hWnd: hwnd,
                uCallbackMessage: callback_msg(),
                uEdge: ABE_BOTTOM,
                rc: RECT {
                    left: primary.identity.left,
                    top: primary.identity.top,
                    right: primary.identity.right,
                    bottom: primary.identity.bottom,
                },
                lParam: LPARAM(0),
            },
        )
    };

    // Query then set position: claim the bottom edge strip.
    let mut query = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        uCallbackMessage: callback_msg(),
        uEdge: ABE_BOTTOM,
        rc: RECT {
            left: primary.identity.left,
            top: primary.identity.bottom - 40,
            right: primary.identity.right,
            bottom: primary.identity.bottom,
        },
        lParam: LPARAM(0),
    };
    let _ = unsafe { SHAppBarMessage(ABM_QUERYPOS, &mut query) };
    let _ = unsafe { SHAppBarMessage(ABM_SETPOS, &mut query) };

    *APPBAR_HWND.lock().unwrap() = Some(hwnd.0 as usize);
    tracing::info!(
        "spike D appbar registered on bottom edge of primary monitor {}",
        primary.identity.device_name
    );
    Ok(format!("appbar on {} bottom edge", primary.identity.device_name))
}

#[tauri::command]
pub fn spike_d_remove_appbar() -> Result<(), String> {
    let hwnd_opt = APPBAR_HWND.lock().unwrap().take().map(|p| HWND(p as *mut _));
    if let Some(hwnd) = hwnd_opt {
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
        tracing::info!("spike D appbar removed");
    }
    Ok(())
}

unsafe extern "system" fn appbar_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == callback_msg() {
        tracing::info!(
            "spike D appbar callback wparam={:?} lparam={:?}",
            wparam, lparam
        );
        return LRESULT(0);
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
