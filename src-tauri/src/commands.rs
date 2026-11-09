//! Typed Tauri command contracts for Phase 1 (core multi-dock).
//! Mirrors `src/tauri.ts` on the frontend.

use std::collections::HashSet;

use serde::Deserialize;
use tauri::{AppHandle, Manager, State};

use crate::config::{load_config, save_config};
use crate::dock_host::{
    dock_host_close, dock_host_list_open, dock_host_open, DockWindowHost,
};
use crate::dock_model::{AppConfig, DockConfig, DockItem};
use crate::icons::icon_extract;
use crate::launcher::{item_launch, LaunchTarget};
use crate::windows_tracker::{RunningApp, windows_tracker_list};

// ---------------------------------------------------------------------------
// Dock CRUD
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn dock_list() -> Result<AppConfig, String> {
    Ok(load_config())
}

#[tauri::command]
pub fn dock_create(
    app: AppHandle,
    dock: DockConfig,
    host: State<'_, DockWindowHost>,
) -> Result<DockConfig, String> {
    let mut cfg = load_config();
    cfg.docks.push(dock.clone());
    save_config(&cfg).map_err(|e| e.to_string())?;
    dock_host_open(app, dock.clone(), host)?;
    Ok(dock)
}

#[tauri::command]
pub fn dock_update(id: String, changes: DockConfig) -> Result<DockConfig, String> {
    let mut cfg = load_config();
    let found = cfg.docks.iter_mut().find(|d| d.id == id).is_some();
    if !found {
        return Err(format!("dock {} not found", id));
    }
    if let Some(dock) = cfg.docks.iter_mut().find(|d| d.id == id) {
        *dock = changes.clone();
    }
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(changes)
}

#[tauri::command]
pub fn dock_delete(id: String, host: State<'_, DockWindowHost>) -> Result<(), String> {
    dock_host_close(id.clone(), host)?;
    let mut cfg = load_config();
    cfg.docks.retain(|d| d.id != id);
    cfg.items.remove(&id);
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn dock_duplicate(
    id: String,
    app: AppHandle,
    host: State<'_, DockWindowHost>,
) -> Result<DockConfig, String> {
    let mut cfg = load_config();
    let Some(src) = cfg.docks.iter().find(|d| d.id == id).cloned() else {
        return Err(format!("dock {} not found", id));
    };
    let mut dup = src;
    dup.id = format!("{}-copy", dup.id);
    dup.name = format!("{} copy", dup.name);
    cfg.docks.push(dup.clone());
    if let Some(items) = cfg.items.get(&id) {
        cfg.items.insert(dup.id.clone(), items.clone());
    }
    save_config(&cfg).map_err(|e| e.to_string())?;
    dock_host_open(app, dup.clone(), host)?;
    Ok(dup)
}

// ---------------------------------------------------------------------------
// Items
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AddItemRequest {
    pub dock_id: String,
    pub item: DockItem,
}

#[tauri::command]
pub fn dock_items(dock_id: String) -> Result<Vec<DockItem>, String> {
    let cfg = load_config();
    Ok(cfg.items_for(&dock_id))
}

#[tauri::command]
pub fn item_add(req: AddItemRequest) -> Result<DockItem, String> {
    let mut cfg = load_config();
    let item = {
        let items = cfg.items.entry(req.dock_id.clone()).or_default();
        let mut item = req.item;
        item.order = items.len() as i32;
        items.push(item.clone());
        item
    };
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(item)
}

#[tauri::command]
pub fn item_remove(dock_id: String, item_id: String) -> Result<(), String> {
    let mut cfg = load_config();
    if let Some(items) = cfg.items.get_mut(&dock_id) {
        items.retain(|i| i.id != item_id);
        for (i, item) in items.iter_mut().enumerate() {
            item.order = i as i32;
        }
    }
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ReorderRequest {
    pub dock_id: String,
    pub item_ids: Vec<String>,
}

#[tauri::command]
pub fn item_reorder(req: ReorderRequest) -> Result<(), String> {
    let mut cfg = load_config();
    if let Some(items) = cfg.items.get_mut(&req.dock_id) {
        let id_set: HashSet<String> = req.item_ids.iter().cloned().collect();
        items.retain(|i| id_set.contains(&i.id));
        let new_items: Vec<DockItem> = req
            .item_ids
            .iter()
            .filter_map(|id| items.iter().find(|i| i.id == *id).cloned())
            .collect();
        let mut new_items = new_items;
        for (i, item) in new_items.iter_mut().enumerate() {
            item.order = i as i32;
        }
        *items = new_items;
    }
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn item_extract_icon(
    app: AppHandle,
    _dock_id: String,
    path: String,
) -> Result<String, String> {
    icon_extract(app, path)
}

// ---------------------------------------------------------------------------
// Launch
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn app_launch(app: AppHandle, target: LaunchTarget) -> Result<(), String> {
    item_launch(app, target)
}

// ---------------------------------------------------------------------------
// Running apps
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn apps_running() -> Result<Vec<RunningApp>, String> {
    windows_tracker_list()
}

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn config_get() -> Result<AppConfig, String> {
    Ok(load_config())
}

#[tauri::command]
pub fn config_save(cfg: AppConfig) -> Result<(), String> {
    save_config(&cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn config_reset() -> Result<AppConfig, String> {
    let mut cfg = AppConfig::default();
    save_config(&cfg).map_err(|e| e.to_string())?;
    Ok(cfg)
}
