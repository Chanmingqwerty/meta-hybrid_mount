use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use anyhow::Result;

use crate::config::Config;
use crate::magic_mount;
use crate::overlay_mount;
use crate::utils;

// Hardcoded built-in partitions from original main.rs
const BUILTIN_PARTITIONS: &[&str] = &[
    "system", "vendor", "product", "system_ext", "odm", "oem",
];

pub fn run(active_modules: HashMap<String, PathBuf>, config: &Config) -> Result<()> {
    // 1. Load Module Modes
    let module_modes = crate::config::load_module_modes();

    // 2. Group by Partition
    let mut partition_map: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut magic_force_map: HashMap<String, bool> = HashMap::new();

    let mut all_partitions = BUILTIN_PARTITIONS.to_vec();
    let extra_parts: Vec<&str> = config.partitions.iter().map(|s| s.as_str()).collect();
    all_partitions.extend(extra_parts);
    let mut sorted_modules: Vec<_> = active_modules.into_iter().collect();
    sorted_modules.sort_by(|a, b| a.0.cmp(&b.0));

    for (module_id, content_path) in &sorted_modules {
        if !content_path.exists() {
            tracing::debug!("Module {} content missing at {}", module_id, content_path.display());
            continue;
        }

        let mode = module_modes.get(module_id).map(|s| s.as_str()).unwrap_or("auto");
        let is_magic = mode == "magic";

        for &part in &all_partitions {
            let part_dir = content_path.join(part);
            if part_dir.is_dir() {
                partition_map
                    .entry(part.to_string())
                    .or_default()
                    .push(content_path.clone());

                if is_magic {
                    magic_force_map.insert(part.to_string(), true);
                    tracing::info!("Partition /{} forced to Magic Mount by module '{}'", part, module_id);
                }
            }
        }
    }

    // 3. Execute Mounts
    let tempdir = if let Some(t) = &config.tempdir {
        t.clone()
    } else {
        utils::select_temp_dir()?
    };
    
    let mut magic_modules: HashSet<PathBuf> = HashSet::new();

    // Pass 1: OverlayFS
    for (part, modules) in &partition_map {
        let use_magic = *magic_force_map.get(part).unwrap_or(&false);
        if !use_magic {
            let target_path = format!("/{}", part);
            
            // [FIX] Reverse module order for OverlayFS lowerdir.
            // modules list is sorted [A, B, C].
            // OverlayFS expects: lowerdir=C:B:A (C overrides B, B overrides A).
            let overlay_paths: Vec<String> = modules
                .iter()
                .rev() 
                .map(|m| m.join(part).display().to_string())
                .collect();

            tracing::info!("Mounting {} [OVERLAY] ({} layers)", target_path, overlay_paths.len());
            
            if let Err(e) = overlay_mount::mount_overlay(&target_path, &overlay_paths, None, None) {
                tracing::error!(
                    "OverlayFS mount failed for {}: {:#}, falling back to Magic Mount",
                    target_path, e
                );
                magic_force_map.insert(part.to_string(), true);
            }
        }
    }

    // Pass 2: Magic Mount
    let mut magic_partitions = Vec::new();
    for (part, _) in &partition_map {
        if *magic_force_map.get(part).unwrap_or(&false) {
            magic_partitions.push(part.clone());
            if let Some(mods) = partition_map.get(part) {
                for m in mods {
                    magic_modules.insert(m.clone());
                }
            }
        }
    }

    if !magic_modules.is_empty() {
        tracing::info!("Starting Magic Mount Engine for partitions: {:?}", magic_partitions);
        utils::ensure_temp_dir(&tempdir)?;

        // Filter module list based on sorted_modules to ensure correct order is passed to Magic Mount
        let module_list: Vec<PathBuf> = sorted_modules
            .iter()
            .filter(|(_, path)| magic_modules.contains(path))
            .map(|(_, path)| path.clone())
            .collect();

        if let Err(e) = magic_mount::mount_partitions(
            &tempdir,
            &module_list,
            &config.mountsource,
            &config.partitions,
        ) {
            tracing::error!("Magic Mount failed: {:#}", e);
        }

        utils::cleanup_temp_dir(&tempdir);
    }

    tracing::info!("Hybrid Mount Completed");
    Ok(())
}
