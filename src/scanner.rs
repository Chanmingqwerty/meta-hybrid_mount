use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::defs;

/// Scans for active modules based on the logic from original main.rs
pub fn scan_active_modules() -> Result<HashMap<String, PathBuf>> {
    let mut active_modules: HashMap<String, PathBuf> = HashMap::new();

    // 1.1 Scan Standard Modules (Metadata Dir)
    // Checks for disable/remove/skip_mount files
    let std_module_ids = scan_enabled_module_ids(Path::new(defs::MODULE_METADATA_DIR))?;
    for id in std_module_ids {
        // Construct path based on MODULE_CONTENT_DIR as per original logic
        let path = Path::new(defs::MODULE_CONTENT_DIR).join(&id);
        active_modules.insert(id, path);
    }

    // 1.2 Scan Mnt Directory
    // Preserving the exact path logic from original main.rs:
    // Path::new(defs::MODULE_CONTENT_DIR).join("meta-hybrid/mnt")
    let mnt_base_dir = Path::new(defs::MODULE_CONTENT_DIR).join("meta-hybrid/mnt");
    
    if mnt_base_dir.exists() {
        tracing::debug!("Scanning mnt directory: {}", mnt_base_dir.display());
        if let Ok(entries) = fs::read_dir(&mnt_base_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let id = entry.file_name().to_string_lossy().to_string();
                    // Only insert if not already present (or_insert logic)
                    active_modules.entry(id).or_insert(entry.path());
                }
            }
        }
    }

    Ok(active_modules)
}

/// Helper to scan enabled module IDs from metadata directory
fn scan_enabled_module_ids(metadata_dir: &Path) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    if !metadata_dir.exists() {
        return Ok(ids);
    }

    for entry in fs::read_dir(metadata_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let id = entry.file_name().to_string_lossy().to_string();
            // Skip self
            if id == "meta-hybrid" {
                continue;
            }
            // Check markers
            if path.join(defs::DISABLE_FILE_NAME).exists()
                || path.join(defs::REMOVE_FILE_NAME).exists()
                || path.join(defs::SKIP_MOUNT_FILE_NAME).exists()
            {
                continue;
            }
            ids.push(id);
        }
    }
    Ok(ids)
}
