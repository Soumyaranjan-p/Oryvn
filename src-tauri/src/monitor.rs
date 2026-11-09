//! Monitor enumeration, DPI scale factors, and stable monitor identity.
//! All native geometry is in physical pixels; convert via each monitor's scale factor.

use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateDCW, DeleteDC, EnumDisplayMonitors, GetDeviceCaps, GetMonitorInfoW, HMONITOR, LOGPIXELSX,
    MONITORINFO, MONITORINFOEXW,
};
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

    // Per-monitor DPI scale factor via a DC created for this specific monitor.
    let scale = unsafe {
        let wide: Vec<u16> = device_name.encode_utf16().collect();
        let hdc = CreateDCW(
            windows::core::PCWSTR::from_raw(wide.as_ptr()),
            None,
            None,
            None,
        );
        let scale = if !hdc.is_invalid() {
            let dpi_x = GetDeviceCaps(Some(hdc), LOGPIXELSX);
            let _ = DeleteDC(hdc);
            dpi_x as f64 / 96.0
        } else {
            1.0
        };
        scale
    };

    monitors.push(Monitor { identity, scale });
    windows::core::BOOL(1) // continue enumeration
}

type HDC = windows::Win32::Graphics::Gdi::HDC;

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
        };
        assert_eq!(m.to_physical(100.0), 150.0);
        assert!((m.to_logical(150.0) - 100.0).abs() < f64::EPSILON);
    }
}
