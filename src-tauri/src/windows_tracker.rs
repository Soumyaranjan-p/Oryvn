//! Running-app tracking via EnumWindows + process queries.
//! Groups multiple windows of one app.

use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, GetWindowThreadProcessId,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::core::{PCWSTR, PWSTR};

static TRACKER: Mutex<Option<AppTracker>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunningApp {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub window_count: u32,
    pub aumid: Option<String>,
}

pub struct AppTracker {
    apps: HashMap<u32, RunningApp>,
}

impl AppTracker {
    fn new() -> Self {
        Self {
            apps: HashMap::new(),
        }
    }

    pub fn refresh() -> Vec<RunningApp> {
        let mut tracker = AppTracker::new();
        tracker.enumerate_windows();
        let apps: Vec<RunningApp> = tracker.apps.values().cloned().collect();
        // Store in static for reuse
        let _ = TRACKER.lock().unwrap().insert(tracker);
        apps
    }

    fn enumerate_windows(&mut self) {
        unsafe {
            let _ = EnumWindows(Some(enum_proc), LPARAM(&mut self.apps as *mut HashMap<u32, RunningApp> as isize));
        }
    }

    fn add_window(&mut self, hwnd: HWND) {
        unsafe {
            if !IsWindowVisible(hwnd).as_bool() {
                return;
            }
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return;
            }
            let entry = self
                .apps
                .entry(pid)
                .or_insert_with(|| RunningApp {
                    pid,
                    name: window_title(hwnd).unwrap_or_default(),
                    path: process_path(pid),
                    window_count: 0,
                    aumid: None,
                });
            entry.window_count += 1;
            if entry.name.is_empty() {
                entry.name = window_title(hwnd).unwrap_or_default();
            }
        }
    }
}

unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    let apps = &mut *(lparam.0 as *mut HashMap<u32, RunningApp>);
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return windows::core::BOOL(1);
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return windows::core::BOOL(1);
        }
        let entry = apps.entry(pid).or_insert_with(|| RunningApp {
            pid,
            name: window_title(hwnd).unwrap_or_default(),
            path: process_path(pid),
            window_count: 0,
            aumid: None,
        });
        entry.window_count += 1;
        if entry.name.is_empty() {
            entry.name = window_title(hwnd).unwrap_or_default();
        }
    }
    windows::core::BOOL(1) // continue enumeration
}

fn window_title(hwnd: HWND) -> Option<String> {
    unsafe {
        let len = GetWindowTextLengthW(hwnd) as usize;
        if len == 0 {
            return None;
        }
        let mut buf: Vec<u16> = vec![0; len + 1];
        let copied = GetWindowTextW(hwnd, &mut buf);
        if copied == 0 {
            return None;
        }
        String::from_utf16(&buf[..copied as usize]).ok()
    }
}

fn process_path(pid: u32) -> Option<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut size: u32 = 0;
        // First call to get required size (NULL name)
        let _ = QueryFullProcessImageNameW(
            h,
            windows::Win32::System::Threading::PROCESS_NAME_FORMAT(0),
            PWSTR::null(),
            &mut size,
        );
        if size == 0 {
            return None;
        }
        let mut buf: Vec<u16> = vec![0; size as usize];
        let ok = QueryFullProcessImageNameW(
            h,
            windows::Win32::System::Threading::PROCESS_NAME_FORMAT(0),
            PWSTR::from_raw(buf.as_mut_ptr()),
            &mut size,
        );
        if ok.is_ok() {
            // Trim to actual length
            String::from_utf16(&buf[..size as usize]).ok()
        } else {
            None
        }
    }
}

/// Get all currently running apps with visible windows.
#[tauri::command]
pub fn windows_tracker_list() -> Result<Vec<RunningApp>, String> {
    Ok(AppTracker::refresh())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_running_app_serializes() {
        let app = RunningApp {
            pid: 1234,
            name: "notepad.exe".into(),
            path: Some("C:\\Windows\\notepad.exe".into()),
            window_count: 1,
            aumid: None,
        };
        let json = serde_json::to_string(&app).unwrap();
        let back: RunningApp = serde_json::from_str(&json).unwrap();
        assert_eq!(back.pid, app.pid);
    }
}
