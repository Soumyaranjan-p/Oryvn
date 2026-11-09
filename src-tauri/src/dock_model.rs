//! Pure data model for docks and items. Decoupled from windows — no
//! WebviewWindow or HWND references here. The DockWindowHost owns windows.

use serde::{Deserialize, Serialize};

/// A dock: a small always-on-top bar of items pinned to a screen edge or floating.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DockConfig {
    pub id: String,
    pub name: String,
    /// Stable monitor identity string (e.g. "\\\\.\\DISPLAY1" or device name).
    /// Empty string means "primary monitor" / auto.
    pub monitor_id: String,
    pub edge: Edge,
    /// Saved floating position (physical pixels), used when edge == Floating.
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub icon_size: i32,
    pub gap: i32,
    pub scale: f64,
    pub theme: Theme,
    pub background: Background,
    pub auto_hide: bool,
    pub collapse_to_button: bool,
    pub hide_when_fullscreen: bool,
    pub orientation: Orientation,
}

impl Default for DockConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "New Dock".into(),
            monitor_id: String::new(),
            edge: Edge::Bottom,
            x: 100,
            y: 100,
            width: 400,
            height: 60,
            icon_size: 32,
            gap: 8,
            scale: 1.0,
            theme: Theme::System,
            background: Background::Solid,
            auto_hide: false,
            collapse_to_button: false,
            hide_when_fullscreen: false,
            orientation: Orientation::Horizontal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
    Floating,
}

impl Default for Edge {
    fn default() -> Self {
        Edge::Bottom
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::System
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Background {
    Solid,
    Transparent,
    Mica,
    Acrylic,
}

impl Default for Background {
    fn default() -> Self {
        Background::Solid
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Default for Orientation {
    fn default() -> Self {
        Orientation::Horizontal
    }
}

/// An item on a dock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockItem {
    pub id: String,
    pub kind: ItemType,
    pub label: String,
    pub path: Option<String>,      // .exe / folder / file
    pub aumid: Option<String>,     // UWP/Store app
    pub url: Option<String>,
    pub args: Vec<String>,         // launch args
    pub icon_url: Option<String>,  // asset URL from Rust icon cache
    pub custom_icon_url: Option<String>, // user override
    pub order: i32,
}

impl DockItem {
    pub fn sort_key(&self) -> i32 {
        self.order
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    App,
    Folder,
    File,
    Url,
    Separator,
}

/// Top-level application config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    pub docks: Vec<DockConfig>,
    /// Per-dock items, keyed by dock id.
    pub items: std::collections::HashMap<String, Vec<DockItem>>,
    pub launch_at_login: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            docks: Vec::new(),
            items: std::collections::HashMap::new(),
            launch_at_login: false,
        }
    }
}

impl AppConfig {
    pub fn dock_mut(&mut self, id: &str) -> Option<&mut DockConfig> {
        self.docks.iter_mut().find(|d| d.id == id)
    }

    pub fn dock(&self, id: &str) -> Option<&DockConfig> {
        self.docks.iter().find(|d| d.id == id)
    }

    pub fn items_for(&self, dock_id: &str) -> Vec<DockItem> {
        self.items
            .get(dock_id)
            .cloned()
            .unwrap_or_default()
    }
}
