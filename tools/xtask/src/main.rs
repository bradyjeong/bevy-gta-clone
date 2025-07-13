//! Development automation tool for the Amp game engine

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Command;

mod util;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the full CI pipeline locally
    Ci,
    /// Format all code
    Fmt,
    /// Run linting
    Lint,
    /// Run tests
    Test,
    /// Generate documentation
    Doc,
    /// Validate documentation
    DocValidate,
    /// Check all crates
    Check,
    /// Run coverage analysis
    Coverage,
    /// Bump version
    BumpVersion {
        /// Version type to bump
        #[arg(value_enum)]
        version_type: VersionType,
    },
    /// Validate configuration files
    ValidateConfigs,
    /// Run benchmarks
    Bench {
        /// Benchmark name pattern (optional)
        #[arg(short, long)]
        name: Option<String>,
        /// Enable GPU culling benchmarks
        #[arg(long)]
        gpu_culling: bool,
        /// Output format (default: terminal)
        #[arg(long, default_value = "terminal")]
        output: String,
    },
    /// Clean up compiled shader artifacts (SPIR-V, cache dirs)
    ShaderCache,
}

#[derive(clap::ValueEnum, Clone)]
enum VersionType {
    Patch,
    Minor,
    Major,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ci => run_ci(),
        Commands::Fmt => run_fmt(),
        Commands::Lint => run_lint(),
        Commands::Test => run_test(),
        Commands::Doc => run_doc(),
        Commands::DocValidate => run_doc_validate(),
        Commands::Check => run_check(),
        Commands::Coverage => run_coverage(),
        Commands::BumpVersion { version_type } => bump_version(version_type),
        Commands::ValidateConfigs => validate_configs(),
        Commands::Bench {
            name,
            gpu_culling,
            output,
        } => run_bench(name, gpu_culling, output),
        Commands::ShaderCache => clean_shader_cache(),
    }
}

fn run_ci() -> Result<()> {
    println!("Running full CI pipeline...");

    run_fmt()?;
    run_lint()?;
    run_test()?;
    run_coverage()?;
    run_doc()?;
    run_doc_validate()?;
    validate_configs()?;

    println!("✅ CI pipeline completed successfully!");
    Ok(())
}

fn run_fmt() -> Result<()> {
    println!("Formatting code...");

    let output = util::run_cargo_output(&["fmt", "--all", "--", "--check"])?;

    if !output.status.success() {
        println!("Running cargo fmt to fix formatting issues...");
        util::run_cargo(&["fmt", "--all"])?;
    }

    util::success("Code formatted");
    Ok(())
}

fn run_lint() -> Result<()> {
    println!("Running clippy...");
    util::run_cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
    ])?;

    // Run cargo udeps to check for unused dependencies (non-blocking)
    println!("Checking for unused dependencies...");
    let udeps_result =
        util::run_cargo_output(&["udeps", "--workspace", "--all-targets", "--all-features"]);

    match udeps_result {
        Ok(output) => {
            if !output.status.success() {
                println!("⚠️  Unused dependencies found (non-blocking):");
                println!("{}", String::from_utf8_lossy(&output.stdout));
                println!("{}", String::from_utf8_lossy(&output.stderr));
            } else {
                println!("✅ No unused dependencies found");
            }
        }
        Err(e) => {
            println!("⚠️  cargo udeps not available or failed (non-blocking): {e}");
            println!("   Install with: cargo install cargo-udeps");
        }
    }

    util::success("Linting passed");
    Ok(())
}

fn run_test() -> Result<()> {
    println!("Running tests...");

    let status = Command::new("cargo")
        .args(["test", "--workspace", "--all-features"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Tests failed");
    }

    println!("✅ Tests passed");
    Ok(())
}

fn run_doc() -> Result<()> {
    println!("Generating documentation...");

    let status = Command::new("cargo")
        .args(["doc", "--workspace", "--no-deps", "--all-features"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Documentation generation failed");
    }

    println!("✅ Documentation generated");
    Ok(())
}

fn run_doc_validate() -> Result<()> {
    println!("Validating documentation...");

    // Check for missing documentation
    let status = Command::new("cargo")
        .args(["doc", "--workspace", "--no-deps", "--all-features"])
        .env("RUSTDOCFLAGS", "-D missing_docs")
        .status()?;

    if !status.success() {
        anyhow::bail!("Documentation validation failed - missing docs");
    }

    // Check markdown files exist and are not empty
    let markdown_files = [
        "README.md",
        "CONTRIBUTING.md",
        "AGENT.md",
        "docs/README.md",
        "docs/architecture/README.md",
        "docs/guides/development.md",
        "docs/adr/README.md",
    ];

    for file in &markdown_files {
        let path = std::path::Path::new(file);
        if !path.exists() {
            anyhow::bail!("Required markdown file missing: {}", file);
        }

        let content = std::fs::read_to_string(path)?;
        if content.trim().is_empty() {
            anyhow::bail!("Markdown file is empty: {}", file);
        }
    }

    println!("✅ Documentation validation passed");
    Ok(())
}

fn run_check() -> Result<()> {
    println!("Checking all crates...");

    let status = Command::new("cargo")
        .args(["check", "--workspace", "--all-targets", "--all-features"])
        .status()?;

    if !status.success() {
        anyhow::bail!("Check failed");
    }

    println!("✅ All crates check passed");
    Ok(())
}

fn run_coverage() -> Result<()> {
    println!("Running coverage analysis...");
    util::ensure_cargo_tool("llvm-cov")?;

    util::run_cargo(&[
        "llvm-cov",
        "--workspace",
        "--all-features",
        "--lcov",
        "--output-path",
        "lcov.info",
        "--fail-under-lines",
        "70",
    ])?;

    util::success("Coverage analysis passed (≥70%)");
    Ok(())
}

fn bump_version(version_type: VersionType) -> Result<()> {
    let version_arg = match version_type {
        VersionType::Patch => "patch",
        VersionType::Minor => "minor",
        VersionType::Major => "major",
    };

    println!("Bumping {version_arg} version...");

    // This is a stub - in a real implementation you'd use cargo-edit or similar
    println!("Version bump for {version_arg} - implementation needed");

    Ok(())
}

fn validate_configs() -> Result<()> {
    println!("Validating configuration files...");

    // Find all config files in the workspace
    let config_dirs = [
        "assets/configs",
        "crates/config_core/assets/examples/configs",
        "examples/configs",
    ];

    let mut found_configs = false;
    let mut validation_errors = Vec::new();

    for config_dir in &config_dirs {
        let path = std::path::Path::new(config_dir);
        if path.exists() {
            println!("  Checking config directory: {config_dir}");

            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let file_path = entry.path();
                    if file_path.extension().is_some_and(|ext| ext == "ron") {
                        found_configs = true;
                        println!("    Validating: {}", file_path.display());

                        // Try to parse the RON file
                        match std::fs::read_to_string(&file_path) {
                            Ok(content) => {
                                if let Err(e) = ron::from_str::<ron::Value>(&content) {
                                    validation_errors.push(format!(
                                        "  ❌ {}: RON parsing error: {}",
                                        file_path.display(),
                                        e
                                    ));
                                } else {
                                    println!("    ✅ {}: Valid RON syntax", file_path.display());
                                }
                            }
                            Err(e) => {
                                validation_errors.push(format!(
                                    "  ❌ {}: Failed to read file: {}",
                                    file_path.display(),
                                    e
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Validate example configs with specific config types
    validate_typed_configs(&mut validation_errors)?;

    if !validation_errors.is_empty() {
        println!("❌ Configuration validation failed:");
        for error in &validation_errors {
            println!("{error}");
        }
        anyhow::bail!(
            "Configuration validation failed with {} errors",
            validation_errors.len()
        );
    }

    if found_configs {
        println!("✅ All configuration files are valid");
    } else {
        println!("⚠️  No configuration files found to validate");
    }

    Ok(())
}

fn validate_typed_configs(validation_errors: &mut Vec<String>) -> Result<()> {
    // Test loading configs using the config_core system
    let status = Command::new("cargo")
        .args([
            "test",
            "-p",
            "config_core",
            "--",
            "test_config",
            "--nocapture",
        ])
        .output()?;

    if !status.status.success() {
        let error_output = String::from_utf8_lossy(&status.stderr);
        validation_errors.push(format!(
            "  ❌ config_core validation tests failed:\n{error_output}"
        ));
    }

    Ok(())
}

fn run_bench(name: Option<String>, gpu_culling: bool, output: String) -> Result<()> {
    println!("Running benchmarks...");

    let mut args = vec!["bench"];

    if gpu_culling {
        args.extend(["--features", "gpu_culling"]);
        util::info("GPU culling benchmarks enabled");
    }

    if let Some(ref pattern) = name {
        args.push(pattern);
        util::info(&format!("Filtering benchmarks: {pattern}"));
    }

    if output != "terminal" {
        unsafe {
            std::env::set_var("CARGO_PROFILE_BENCH_DEBUG", "true");
        }
        util::info(&format!("Output format: {output}"));
    }

    util::run_cargo(&args)?;

    if gpu_culling {
        util::info("GPU vs CPU culling benchmarks completed - should be ≥2x faster than CPU");
    }

    util::info("Results saved to: target/criterion/");
    util::success("Benchmark execution completed successfully!");
    Ok(())
}

fn clean_shader_cache() -> Result<()> {
    println!("Cleaning shader cache and SPIR-V artifacts...");

    let (shader_dirs, spv_files) = util::find_gpu_artifacts()?;

    util::info(&format!("Found {} shader directories", shader_dirs.len()));
    util::info(&format!("Found {} SPIR-V files", spv_files.len()));

    let dirs_removed = util::remove_dirs(&shader_dirs)?;
    let files_removed = util::remove_files(&spv_files)?;

    // Also clean common target/shader_cache if it exists
    let shader_cache_path = std::path::PathBuf::from("target/shader_cache");
    if shader_cache_path.exists() {
        std::fs::remove_dir_all(&shader_cache_path)?;
        util::info("Removed target/shader_cache directory");
    }

    util::success(&format!(
        "Shader cache cleanup: {dirs_removed} directories, {files_removed} files removed"
    ));
    Ok(())
}
