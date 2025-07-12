//! Development automation tool for the Amp game engine

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process::Command;

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

    let output = Command::new("cargo")
        .args(["fmt", "--all", "--", "--check"])
        .output()?;

    if !output.status.success() {
        println!("Running cargo fmt to fix formatting issues...");
        Command::new("cargo").args(["fmt", "--all"]).status()?;
    }

    println!("✅ Code formatted");
    Ok(())
}

fn run_lint() -> Result<()> {
    println!("Running clippy...");

    let status = Command::new("cargo")
        .args([
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("Clippy found issues");
    }

    println!("✅ Linting passed");
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

    // Install cargo-llvm-cov if not available
    let install_status = Command::new("cargo")
        .args(["install", "cargo-llvm-cov"])
        .status()?;

    if !install_status.success() {
        println!("cargo-llvm-cov already installed or installation failed");
    }

    // Run coverage with 70% threshold
    let status = Command::new("cargo")
        .args([
            "llvm-cov",
            "--workspace",
            "--all-features",
            "--lcov",
            "--output-path",
            "lcov.info",
            "--fail-under-lines",
            "70",
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("Coverage below 70% threshold");
    }

    println!("✅ Coverage analysis passed (≥70%)");
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
