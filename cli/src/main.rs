//! YAML Linter CLI

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use is_terminal::IsTerminal;
use std::io;
use std::path::PathBuf;
use walkdir::WalkDir;
use yaml_lint_core::{Config, LintLevel, Linter};

/// Color mode for output
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum ColorMode {
    /// Always use colors
    Always,
    /// Never use colors
    Never,
    /// Auto-detect based on terminal (default)
    #[default]
    Auto,
}

#[derive(Parser)]
#[command(name = "yaml-lint")]
#[command(version, about = "A fast YAML linter written in Rust", long_about = None)]
struct Cli {
    /// Files or directories to lint
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Output format (standard, colored, parsable)
    #[arg(short = 'f', long, default_value = "standard")]
    format: String,

    /// When to use colors (always, never, auto)
    #[arg(long, value_enum, default_value = "auto")]
    color: ColorMode,

    /// Return non-zero exit code on warnings
    #[arg(long)]
    strict: bool,

    /// Preset to use (default, relaxed)
    #[arg(short = 'd', long)]
    preset: Option<String>,

    /// List files that would be linted
    #[arg(long)]
    list_files: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Configure color output
    configure_colors(&cli.color);

    // Load configuration
    let config = load_config(&cli)?;

    // Create linter
    let linter = Linter::new(config);

    // Collect YAML files
    let yaml_files = collect_yaml_files(&cli.paths)?;

    if cli.list_files {
        for file in &yaml_files {
            println!("{}", file.display());
        }
        return Ok(());
    }

    // Parse output format, defaulting to colored if TTY and not explicitly set
    let format: yaml_lint_core::output::OutputFormat =
        if cli.format == "standard" && should_use_colors(&cli.color) {
            yaml_lint_core::output::OutputFormat::Colored
        } else {
            cli.format
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid format: {}", e))?
        };

    let formatter = format.formatter();

    // Lint all files
    let mut has_errors = false;
    let mut has_warnings = false;
    let mut total_problems = 0;

    for file in &yaml_files {
        match linter.lint_file(file) {
            Ok(problems) => {
                if !problems.is_empty() {
                    let output = formatter.format_problems(&problems, &file.display().to_string());
                    print!("{}", output);

                    for problem in &problems {
                        match problem.level {
                            LintLevel::Error => has_errors = true,
                            LintLevel::Warning => has_warnings = true,
                        }
                    }

                    total_problems += problems.len();
                }
            }
            Err(e) => {
                eprintln!("Error linting {}: {}", file.display(), e);
                has_errors = true;
            }
        }
    }

    // Print summary if there were problems
    if total_problems > 0 {
        eprintln!();
        eprintln!(
            "Found {} problem(s) in {} file(s)",
            total_problems,
            yaml_files.len()
        );
    }

    // Exit with appropriate code
    if has_errors {
        std::process::exit(1);
    } else if has_warnings && cli.strict {
        std::process::exit(2);
    }

    Ok(())
}

/// Load configuration from CLI options
fn load_config(cli: &Cli) -> Result<Config> {
    if let Some(config_path) = &cli.config {
        // Load from specified path
        Config::load_from_file(config_path)
            .with_context(|| format!("Failed to load config from {}", config_path.display()))
    } else if let Some(preset) = &cli.preset {
        // Use specified preset
        match preset.as_str() {
            "default" => Ok(Config::with_default_preset()),
            "relaxed" => Ok(Config::with_relaxed_preset()),
            _ => Err(anyhow::anyhow!("Unknown preset: {}", preset)),
        }
    } else {
        // Try to find config file in current directory and parents
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;
        if let Some(config_path) = Config::find_config_file(&current_dir) {
            Config::load_from_file(&config_path)
                .with_context(|| format!("Failed to load config from {}", config_path.display()))
        } else {
            // Use default preset
            Ok(Config::with_default_preset())
        }
    }
}

/// Collect all YAML files from the given paths
fn collect_yaml_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut yaml_files = Vec::new();

    for path in paths {
        if path.is_file() {
            if is_yaml_file(path) {
                yaml_files.push(path.clone());
            }
        } else if path.is_dir() {
            // Walk directory and collect YAML files
            for entry in WalkDir::new(path)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();
                if entry_path.is_file() && is_yaml_file(entry_path) {
                    yaml_files.push(entry_path.to_path_buf());
                }
            }
        } else {
            eprintln!(
                "Warning: {} is neither a file nor a directory",
                path.display()
            );
        }
    }

    if yaml_files.is_empty() {
        return Err(anyhow::anyhow!("No YAML files found"));
    }

    Ok(yaml_files)
}

/// Check if a file is a YAML file based on extension
fn is_yaml_file(path: &std::path::Path) -> bool {
    if let Some(ext) = path.extension() {
        matches!(ext.to_str(), Some("yaml") | Some("yml"))
    } else {
        // Also check for common config file names without extensions
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            matches!(name, ".yamllint" | ".yamllint.yml" | ".yamllint.yaml")
        } else {
            false
        }
    }
}

/// Configure color output based on mode and environment
fn configure_colors(mode: &ColorMode) {
    // Respect NO_COLOR environment variable (https://no-color.org/)
    let no_color = std::env::var("NO_COLOR").is_ok();

    let enable_colors = match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => !no_color && io::stdout().is_terminal(),
    };

    if enable_colors {
        colored::control::set_override(true);
    } else {
        colored::control::set_override(false);
    }
}

/// Check if colors should be used based on mode and environment
fn should_use_colors(mode: &ColorMode) -> bool {
    let no_color = std::env::var("NO_COLOR").is_ok();

    match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => !no_color && io::stdout().is_terminal(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_yaml_file() {
        assert!(is_yaml_file(std::path::Path::new("test.yaml")));
        assert!(is_yaml_file(std::path::Path::new("test.yml")));
        assert!(is_yaml_file(std::path::Path::new(".yamllint")));
        assert!(!is_yaml_file(std::path::Path::new("test.txt")));
        assert!(!is_yaml_file(std::path::Path::new("test")));
    }
}
