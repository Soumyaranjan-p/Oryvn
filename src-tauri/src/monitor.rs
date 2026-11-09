//! Monitor enumeration, DPI scale factors, and stable monitor identity.
//! All native geometry is in physical pixels; convert via each monitor's scale factor.
//!
//! Scale factor is obtained via `GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, ...)`,
//! which returns the effective DPI (accounts for per-monitor display scaling set by
//! the user). Scale = dpi / 96.0. This is the recommended API for per-monitor DPI
//! awareness on Windows 10 1607+ (our target: 22H2+), and is more correct than the
//! legacy `CreateDCW` + `GetDeviceCaps(LOGPIXELSX)` approach on mixed-DPI systems.

use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::core::Result;

/// Stable identity for a monitor: device name + physical bounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorIdentity {
    pub device_name: String,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub is_primary: bool,
}

impl MonitorIdentity {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

/// A monitor with its current scale factor (for DPI conversion).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub identity: MonitorIdentity,
    /// DPI scale factor (e.g. 1.0, 1.25, 1.5, 2.0).
    pub scale: f64,
    /// Effective DPI as reported by GetDpiForMonitor (e.g. 96, 120, 144, 192).
    pub dpi: u32,
}

impl Monitor {
    /// Convert logical pixels to physical pixels for this monitor.
    pub fn to_physical(&self, logical: f64) -> f64 {
        logical * self.scale
    }
    /// Convert physical pixels to logical pixels for this monitor.
    pub fn to_logical(&self, physical: f64) -> f64 {
        physical / self.scale
    }
}

/// Enumerate all display monitors with their per-monitor scale factors.
pub fn enumerate_monitors() -> Result<Vec<Monitor>> {
    let mut monitors: Vec<Monitor> = Vec::new();
    let _ret = unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut monitors as *mut Vec<Monitor> as isize),
        )
    };
    // BOOL return ignored for the spike; monitors may be empty on failure.
    Ok(monitors)
}

unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> windows::core::BOOL {
    let monitors = &mut *(lparam.0 as *mut Vec<Monitor>);

    let mut info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    let _ = unsafe {
        GetMonitorInfoW(
            hmonitor,
            &mut info as *mut MONITORINFOEXW as *mut MONITORINFO,
        )
    };

    let device_name = String::from_utf16_lossy(&info.szDevice)
        .trim_end_matches('\0')
        .to_string();

    let is_primary = (info.monitorInfo.dwFlags & 0x0000_0001) == 1; // MONITORINFOF_PRIMARY

    let rc = info.monitorInfo.rcMonitor;
    let identity = MonitorIdentity {
        device_name: device_name.clone(),
        left: rc.left,
        top: rc.top,
        right: rc.right,
        bottom: rc.bottom,
        is_primary,
    };

    // Per-monitor effective DPI via GetDpiForMonitor (MDT_EFFECTIVE_DPI).
    // Scale = dpi / 96.0. Falls back to 1.0 if the call fails.
    let (dpi, scale) = unsafe {
        let mut dpi_x: u32 = 0;
        let mut dpi_y: u32 = 0;
        match GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) {
            Ok(()) => {
                let dpi = dpi_x.max(96);
                (dpi, dpi as f64 / 96.0)
            }
            Err(_) => (96, 1.0),
        }
    };

    monitors.push(Monitor { identity, scale, dpi });
    windows::core::BOOL(1) // continue enumeration
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_roundtrip() {
        let m = Monitor {
            identity: MonitorIdentity {
                device_name: "test".into(),
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
                is_primary: true,
            },
            scale: 1.5,
            dpi: 144,
        };
        assert_eq!(m.to_physical(100.0), 150.0);
        assert!((m.to_logical(150.0) - 100.0).abs() < f64::EPSILON);
        assert_eq!(m.dpi, 144);
    }
}
