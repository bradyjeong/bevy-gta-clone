//! Common utilities for xtask commands
//!
//! This module provides shared functionality to keep individual commands ≤30 LOC each.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Execute a cargo command with given arguments
pub fn run_cargo(args: &[&str]) -> Result<()> {
    let status = Command::new("cargo")
        .args(args)
        .status()
        .context("Failed to execute cargo command")?;

    if !status.success() {
        anyhow::bail!("Cargo command failed: cargo {}", args.join(" "));
    }

    Ok(())
}

/// Execute a cargo command and return output
pub fn run_cargo_output(args: &[&str]) -> Result<std::process::Output> {
    Command::new("cargo")
        .args(args)
        .output()
        .context("Failed to execute cargo command")
}

/// Find files with specific extensions in a directory tree
pub fn find_files_with_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !dir.exists() {
        return Ok(files);
    }

    fn visit_dir(dir: &Path, extension: &str, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                visit_dir(&path, extension, files)?;
            } else if let Some(ext) = path.extension() {
                if ext == extension {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    visit_dir(dir, extension, &mut files)?;
    Ok(files)
}

/// Remove files matching a pattern
pub fn remove_files(files: &[PathBuf]) -> Result<usize> {
    let mut removed_count = 0;

    for file in files {
        if file.exists() {
            std::fs::remove_file(file)
                .with_context(|| format!("Failed to remove file: {}", file.display()))?;
            removed_count += 1;
            println!("  🗑️  Removed: {}", file.display());
        }
    }

    Ok(removed_count)
}

/// Remove directories recursively
pub fn remove_dirs(dirs: &[PathBuf]) -> Result<usize> {
    let mut removed_count = 0;

    for dir in dirs {
        if dir.exists() && dir.is_dir() {
            std::fs::remove_dir_all(dir)
                .with_context(|| format!("Failed to remove directory: {}", dir.display()))?;
            removed_count += 1;
            println!("  🗑️  Removed directory: {}", dir.display());
        }
    }

    Ok(removed_count)
}

/// Check if a command exists in PATH
pub fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Install a cargo tool if not already installed
pub fn ensure_cargo_tool(tool: &str) -> Result<()> {
    let tool_name = format!("cargo-{tool}");

    if !command_exists(&tool_name) {
        println!("Installing cargo-{tool}...");
        run_cargo(&["install", &format!("cargo-{tool}")])?;
        println!("✅ Installed cargo-{tool}");
    }

    Ok(())
}

/// Format success message
pub fn success(message: &str) {
    println!("✅ {message}");
}

/// Format info message
pub fn info(message: &str) {
    println!("  ℹ️  {message}");
}

/// Find common GPU artifact directories and files
pub fn find_gpu_artifacts() -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut shader_dirs = Vec::new();
    let mut spv_files = Vec::new();

    // Common shader cache locations
    let search_paths = [
        "target/debug/build",
        "target/release/build",
        "target/shader_cache",
        "assets/shaders",
        "crates/amp_render/assets/shaders",
        "examples/assets/shaders",
    ];

    for search_path in &search_paths {
        let path = Path::new(search_path);
        if path.exists() {
            // Find .spv files (compiled SPIR-V)
            let spirv_files = find_files_with_extension(path, "spv")?;
            spv_files.extend(spirv_files);

            // Find shader cache directories
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir()
                        && (entry_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .contains("shader")
                            || entry_path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .contains("spirv"))
                    {
                        shader_dirs.push(entry_path);
                    }
                }
            }
        }
    }

    Ok((shader_dirs, spv_files))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_find_files_with_extension() {
        let temp_dir = std::env::temp_dir().join("xtask_test");
        fs::create_dir_all(&temp_dir).unwrap();

        // Create test files
        let test_file = temp_dir.join("test.spv");
        fs::write(&test_file, "test").unwrap();

        let result = find_files_with_extension(&temp_dir, "spv").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], test_file);

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_find_gpu_artifacts() {
        let result = find_gpu_artifacts();
        assert!(result.is_ok());

        let (_dirs, _files) = result.unwrap();
        // Should not error even if no artifacts exist - test passes if no panic
    }

    #[test]
    fn test_command_exists() {
        // Test with a command that should exist
        assert!(command_exists("ls") || command_exists("dir"));

        // Test with a command that should not exist
        assert!(!command_exists("nonexistent_command_xyz123"));
    }
}
