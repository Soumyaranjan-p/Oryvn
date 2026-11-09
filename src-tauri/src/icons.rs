//! Icon extraction + on-disk PNG cache, keyed by path + mtime.
//! Frontend receives asset URLs served by Tauri.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use tauri::{AppHandle, Manager};

use crate::brand::APP_NAME;

/// In-flight icon cache: path -> asset URL. Keyed by path+mtime in the file name.
fn icon_cache() -> &'static std::sync::Mutex<HashMap<String, String>> {
    static CACHE: OnceLock<std::sync::Mutex<HashMap<String, String>>> = OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

fn cache_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data = std::env::var("APPDATA").map_err(|_| "APPDATA not set")?;
    let dir = PathBuf::from(app_data).join(APP_NAME).join("icon_cache");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Extract an icon for `path` (exe/folder/file), cache as PNG, return an asset URL.
/// Falls back to a generic placeholder if extraction fails.
#[tauri::command]
pub fn icon_extract(app: AppHandle, path: String) -> Result<String, String> {
    // Check in-memory cache first
    {
        let cache = icon_cache().lock().unwrap();
        if let Some(url) = cache.get(&path) {
            return Ok(url.clone());
        }
    }

    let url = do_extract(&app, &path).unwrap_or_else(|_| {
        tracing::warn!("Icon extraction failed for {}, using placeholder", path);
        "asset://oryvn-icon-placeholder".into()
    });

    icon_cache().lock().unwrap().insert(path, url.clone());
    Ok(url)
}

fn do_extract(_app: &AppHandle, path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let pb = PathBuf::from(path);
    if !pb.exists() {
        return Err("path does not exist".into());
    }

    // Use SHGetFileInfo to get the shell icon index, then extract via IShellItemImageFactory
    // for high-res. For the spike we produce a placeholder PNG keyed by path+mtime.
    // Real implementation: SHGetFileInfo(..., SHGFI_ICON) -> HICON -> bitmap -> PNG bytes.
    let mtime = pb.metadata()?.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();
    let cache_name = format!("{}_{}.png", urlsafe_path(&pb), mtime);
    let cache_path = cache_dir()?.join(&cache_name);

    if !cache_path.exists() {
        // Write a small placeholder PNG so we have a valid cache file.
        // Real impl: render the extracted HICON into these bytes.
        let placeholder = placeholder_png();
        std::fs::write(&cache_path, placeholder)?;
    }

    // Serve via Tauri's asset protocol: register a custom protocol or use file://.
    // For now, return a file:// URL (Tauri allows file assets via the allowlist).
    let url = format!("file://{}", cache_path.display());
    Ok(url)
}

fn urlsafe_path(pb: &PathBuf) -> String {
    pb.to_string_lossy()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

fn placeholder_png() -> Vec<u8> {
    // 1x1 transparent PNG
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0x82, 0x69, 0x82, 0x97,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlsafe_path() {
        let s = urlsafe_path(&PathBuf::from("C:\\foo\\bar.exe"));
        assert!(!s.contains(':'));
        assert!(!s.contains('\\'));
    }
}
