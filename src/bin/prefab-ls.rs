//! CLI tool for inspecting prefab IDs and detecting collisions
//!
//! This tool lists all registered prefab IDs and helps detect hash collisions
//! across the Factory system.

use std::collections::HashMap;
use std::env;
use std::path::Path;

use config_core::{ConfigLoader, FactorySettings, GameConfig};
use gameplay_factory::{clear_all_prefab_ids, get_all_prefab_ids, Factory, PrefabId};

/// Information about a prefab ID
#[derive(Debug, Clone)]
struct PrefabInfo {
    id: PrefabId,
    path: Option<String>,
    source: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let mut scan_paths = Vec::new();
    let mut show_collisions = false;
    let mut verbose = false;

    // Parse command line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--path" | "-p" => {
                if i + 1 < args.len() {
                    scan_paths.push(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --path requires a value");
                    return Ok(());
                }
            }
            "--collisions" | "-c" => {
                show_collisions = true;
                i += 1;
            }
            "--verbose" | "-v" => {
                verbose = true;
                i += 1;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_help();
                return Ok(());
            }
        }
    }

    // Clear global registry to start fresh
    clear_all_prefab_ids();

    let mut prefab_infos = Vec::new();

    // If no paths provided, try to load from default config
    if scan_paths.is_empty() {
        match load_from_config() {
            Ok(infos) => prefab_infos.extend(infos),
            Err(e) => {
                eprintln!("Warning: Could not load from config: {e}");
                eprintln!("Use --path to specify directories to scan");
            }
        }
    } else {
        // Scan provided paths
        for path in &scan_paths {
            match scan_directory(path) {
                Ok(infos) => prefab_infos.extend(infos),
                Err(e) => eprintln!("Error scanning {path}: {e}"),
            }
        }
    }

    // Detect collisions
    let mut hash_map: HashMap<u64, Vec<PrefabInfo>> = HashMap::new();
    for info in prefab_infos {
        let hash = info.id.raw();
        hash_map.entry(hash).or_default().push(info);
    }

    // Report results
    let mut collision_count = 0;
    let mut total_prefabs = 0;

    if show_collisions {
        println!("=== PREFAB ID COLLISION REPORT ===");
        for (hash, infos) in &hash_map {
            if infos.len() > 1 {
                collision_count += 1;
                println!("COLLISION: Hash {} has {} prefabs:", hash, infos.len());
                for info in infos {
                    println!(
                        "  - {} (source: {})",
                        info.path.as_deref().unwrap_or("unknown"),
                        info.source
                    );
                }
                println!();
            }
        }

        if collision_count == 0 {
            println!("✅ No collisions detected!");
        } else {
            println!("❌ {collision_count} collision(s) detected!");
        }
    } else {
        println!("=== PREFAB ID REGISTRY ===");
        let hash_map_len = hash_map.len();
        let mut sorted_infos: Vec<_> = hash_map.into_iter().collect();
        sorted_infos.sort_by_key(|(hash, _)| *hash);

        for (hash, infos) in sorted_infos {
            total_prefabs += infos.len();
            for info in infos {
                if verbose {
                    println!(
                        "PrefabId({:016x}) - {} (source: {})",
                        hash,
                        info.path.as_deref().unwrap_or("unknown"),
                        info.source
                    );
                } else {
                    println!(
                        "PrefabId({:016x}) - {}",
                        hash,
                        info.path.as_deref().unwrap_or("unknown")
                    );
                }
            }
        }

        println!("\n=== SUMMARY ===");
        println!("Total prefabs: {total_prefabs}");
        println!("Unique hashes: {hash_map_len}");
        if total_prefabs != hash_map_len {
            println!(
                "⚠️  {} hash collision(s) detected!",
                total_prefabs - hash_map_len
            );
        }
    }

    Ok(())
}

fn print_help() {
    println!(
        r#"prefab-ls - Prefab ID Inspector

USAGE:
    prefab-ls [OPTIONS]

OPTIONS:
    -p, --path <DIR>        Scan directory for prefab files
    -c, --collisions        Show only collisions
    -v, --verbose           Verbose output
    -h, --help              Show this help message

EXAMPLES:
    prefab-ls                           # Load from default config
    prefab-ls --path assets/prefabs     # Scan specific directory
    prefab-ls --collisions              # Show only collisions
    prefab-ls --path assets --verbose   # Verbose output
"#
    );
}

fn load_from_config() -> Result<Vec<PrefabInfo>, Box<dyn std::error::Error>> {
    let loader = ConfigLoader::new();
    let config: GameConfig = loader.load_with_merge()?;
    let settings = &config.factory;

    let mut factory = Factory::new();
    let loaded_count = factory.load_directory(settings)?;

    let mut infos = Vec::new();
    for id in get_all_prefab_ids() {
        infos.push(PrefabInfo {
            id,
            path: None, // We don't track paths in the global registry yet
            source: "config".to_string(),
        });
    }

    println!("Loaded {loaded_count} prefabs from config");
    Ok(infos)
}

fn scan_directory(path: &str) -> Result<Vec<PrefabInfo>, Box<dyn std::error::Error>> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(format!("Directory {} does not exist", path.display()).into());
    }

    // Create a temporary factory to scan the directory
    let mut factory = Factory::new();

    // Create temporary settings for this path
    let settings = FactorySettings {
        prefab_path: format!("{}/**/*.ron", path.display()),
        hot_reload: false,
    };

    let loaded_count = factory.load_directory(&settings)?;

    let mut infos = Vec::new();
    for id in get_all_prefab_ids() {
        infos.push(PrefabInfo {
            id,
            path: Some(path.display().to_string()),
            source: "directory_scan".to_string(),
        });
    }

    println!("Loaded {} prefabs from {}", loaded_count, path.display());
    Ok(infos)
}
