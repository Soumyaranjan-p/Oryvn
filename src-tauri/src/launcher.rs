//! Launching: apps (.exe, UWP via AUMID), folders, files, URLs, scripts.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::ShellExecuteExW;
use windows::Win32::UI::Shell::SHELLEXECUTEINFOW;
use windows::core::PCWSTR;

/// Launch target descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchTarget {
    pub kind: LaunchKind,
    pub path: Option<String>,
    pub aumid: Option<String>,
    pub url: Option<String>,
    pub args: Vec<String>,
    pub elevated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LaunchKind {
    App,
    Folder,
    File,
    Url,
}

/// Launch a target with the user's own permissions (no elevation unless requested).
#[tauri::command]
pub fn item_launch(_app: AppHandle, target: LaunchTarget) -> Result<(), String> {
    match target.kind {
        LaunchKind::Url => launch_url(&target),
        LaunchKind::Folder => launch_folder(&target),
        LaunchKind::File => launch_file(&target),
        LaunchKind::App => launch_app(&target),
    }
}

fn launch_url(target: &LaunchTarget) -> Result<(), String> {
    let url = target.url.as_deref().unwrap_or("");
    launch_shell(url, None)
}

fn launch_folder(target: &LaunchTarget) -> Result<(), String> {
    let path = target.path.as_deref().unwrap_or("");
    launch_shell(path, None)
}

fn launch_file(target: &LaunchTarget) -> Result<(), String> {
    let path = target.path.as_deref().unwrap_or("");
    launch_shell(path, None)
}

fn launch_app(target: &LaunchTarget) -> Result<(), String> {
    // UWP/Store app via AUMID
    if let Some(aumid) = &target.aumid {
        return launch_aumid(aumid);
    }
    // Win32 .exe
    let path = target.path.as_deref().unwrap_or("");
    if path.is_empty() {
        return Err("no path or AUMID specified".into());
    }
    let args = target.args.join(" ");
    let cmdline = if args.is_empty() {
        path.to_string()
    } else {
        format!("{} {}", path, args)
    };
    launch_shell(&cmdline, Some(path))
}

fn launch_aumid(aumid: &str) -> Result<(), String> {
    // ShellExecute with the AUMID as the "file" launches UWP apps.
    launch_shell(aumid, None)
}

fn launch_shell(file: &str, _working_dir: Option<&str>) -> Result<(), String> {
    unsafe {
        let mut sei: SHELLEXECUTEINFOW = std::mem::zeroed();
        sei.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        sei.fMask = windows::Win32::UI::Shell::SEE_MASK_FLAG_NO_UI
            | windows::Win32::UI::Shell::SEE_MASK_NO_CONSOLE;
        sei.hwnd = HWND::default();
        sei.lpVerb = PCWSTR::null(); // default verb ("open")
        sei.lpFile = PCWSTR::from_raw(
            file.encode_utf16().chain(Some(0)).collect::<Vec<u16>>().as_ptr(),
        );
        sei.lpParameters = PCWSTR::null();
        sei.lpDirectory = PCWSTR::null();
        sei.nShow = windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL.0 as i32;

        let ok = ShellExecuteExW(&mut sei).is_ok();
        if ok {
            Ok(())
        } else {
            Err(format!("ShellExecuteEx failed for: {}", file))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_target_serializes() {
        let t = LaunchTarget {
            kind: LaunchKind::App,
            path: Some("C:\\Windows\\notepad.exe".into()),
            aumid: None,
            url: None,
            args: vec![],
            elevated: false,
        };
        let json = serde_json::to_string(&t).unwrap();
        let back: LaunchTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(back.path, t.path);
    }
}
