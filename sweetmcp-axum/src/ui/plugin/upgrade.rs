use std::{
    fs,
    io::{Write, stdout},
    path::Path,
    process::Command,
};

use clap::Args;
use ratatui::style::Stylize;

#[derive(Args, Debug)]
pub struct UpgradeArgs {
    /// Upgrade all plugins in the plugins directory
    #[arg(short, long)]
    pub all: bool,

    /// Specific plugin name to upgrade
    #[arg(short, long)]
    pub name: Option<String>,

    /// Only print what would be upgraded without making changes
    #[arg(short, long)]
    pub dry_run: bool,
}

pub fn upgrade_plugins(args: &UpgradeArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Upgrading plugin dependencies...".bold().cyan());

    let project_root = std::env::current_dir()?;
    let plugins_dir = project_root.join("plugins");

    if !plugins_dir.exists() {
        return Err(format!("Plugins directory not found at {}", plugins_dir.display()).into());
    }

    // Get latest versions of common dependencies
    let latest_versions = get_latest_versions()?;

    // If a specific plugin is specified, only upgrade that one
    if let Some(plugin_name) = &args.name {
        let plugin_path = plugins_dir.join(plugin_name);
        if !plugin_path.exists() {
            return Err(format!(
                "Plugin '{}' not found at {}",
                plugin_name,
                plugin_path.display()
            )
            .into());
        }

        upgrade_plugin_dependencies(&plugin_path, &latest_versions, args.dry_run)?;
    } else if args.all {
        // Upgrade all plugins
        let mut upgraded_plugins = Vec::new();
        let mut failed_plugins = Vec::new();

        for entry in fs::read_dir(&plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let plugin_name = match path.file_name() {
                // Already fixed
                Some(name) => name.to_string_lossy().to_string(),
                None => {
                    log::warn!("Skipping directory with invalid name: {}", path.display());
                    continue;
                }
            };

            match upgrade_plugin_dependencies(&path, &latest_versions, args.dry_run) {
                Ok(upgraded) => {
                    if upgraded {
                        upgraded_plugins.push(plugin_name);
                    }
                }
                Err(e) => {
                    failed_plugins.push((plugin_name, e.to_string()));
                }
            }
        }

        // Report summary
        println!();
        println!("{}", "Upgrade Summary".bold().underlined());

        if !upgraded_plugins.is_empty() {
            println!(
                "{} {} {}",
                "✅".green(),
                upgraded_plugins.len(),
                "plugins upgraded:".green()
            );
            for name in upgraded_plugins {
                println!("  - {}", name);
            }
        } else if args.dry_run {
            println!("{}", "No plugins need upgrading".yellow());
        }

        if !failed_plugins.is_empty() {
            println!(
                "{} {} {}",
                "❌".red(),
                failed_plugins.len(),
                "plugins failed to upgrade:".red()
            );
            for (name, error) in failed_plugins {
                println!("  - {}: {}", name.bold(), error);
            }
            return Err("Some plugins failed to upgrade".into());
        }
    } else {
        // No plugin specified and --all not set
        println!("No plugin specified. Use --name <plugin_name> or --all to upgrade plugins.");
        println!("Available plugins:");

        for entry in fs::read_dir(&plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Already fixed
                if let Some(plugin_name) = path.file_name() {
                    println!("  - {}", plugin_name.to_string_lossy());
                }
            }
        }
    }

    Ok(())
}
// Removed extra closing brace here

fn get_latest_versions() -> Result<Vec<(&'static str, String)>, Box<dyn std::error::Error>> {
    let dependencies = [
        "extism-pdk",
        "serde",
        "serde_json",
        "base64",
        "base64-serde",
    ];

    let mut latest_versions = Vec::new();

    for &dep in &dependencies {
        print!("Checking latest version of {}... ", dep);
        stdout().flush()?;

        let output = Command::new("cargo")
            .args(["search", dep, "--limit", "1"])
            .output()?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let version = parse_crate_version(&output_str, dep);

            if let Some(version) = version {
                println!("{}", version.clone().green());
                latest_versions.push((dep, version));
            } else {
                println!("{}", "not found".yellow());
            }
        } else {
            println!("{}", "failed".red());
        }
    }

    Ok(latest_versions)
}

fn parse_crate_version(output: &str, crate_name: &str) -> Option<String> {
    let line = output.lines().next()?;
    let prefix = format!("{} = \"", crate_name);

    if line.starts_with(&prefix) {
        let version_start = prefix.len();
        let version_end = line[version_start..].find('"')?;

        Some(line[version_start..(version_start + version_end)].to_string())
    } else {
        None
    }
}

fn upgrade_plugin_dependencies(
    plugin_path: &Path,
    latest_versions: &[(&str, String)],
    dry_run: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    // Already fixed
    let plugin_name = plugin_path
        .file_name()
        .ok_or_else(|| format!("Invalid plugin path: {}", plugin_path.display()))?
        .to_string_lossy();
    println!();
    println!(
        "{} {}",
        "Upgrading dependencies for:".bold(),
        plugin_name.to_string().green().bold()
    );

    // Check Cargo.toml exists
    let cargo_path = plugin_path.join("Cargo.toml");
    if !cargo_path.exists() {
        return Err(format!("Cargo.toml not found in {}", plugin_path.display()).into());
    }

    // Read Cargo.toml
    let cargo_content = fs::read_to_string(&cargo_path)?;
    let mut lines: Vec<String> = cargo_content.lines().map(String::from).collect();
    let mut upgraded = false;

    // Update dependency versions
    for i in 0..lines.len() {
        for &(dep, ref ver) in latest_versions {
            if lines[i].contains(&format!("{} = ", dep)) {
                if dep == "serde" && lines[i].contains("features") {
                    let current_ver = lines[i].clone();
                    if current_ver.contains(&format!("version = \"{}", ver)) {
                        // Already at latest version
                        continue;
                    }

                    let new_line = lines[i].replace(
                        &format!("{} = {{ version = \"", dep),
                        &format!("{} = {{ version = \"{}\"", dep, ver),
                    );

                    println!("  {} → {}", current_ver.red(), new_line.clone().green());

                    if !dry_run {
                        lines[i] = new_line;
                    }
                    upgraded = true;
                } else {
                    let current_ver = lines[i].clone();
                    if current_ver.contains(&format!("{} = \"{}\"", dep, ver)) {
                        // Already at latest version
                        continue;
                    }

                    let new_line = lines[i]
                        .replace(&format!("{} = \"", dep), &format!("{} = \"{}\"", dep, ver));

                    println!("  {} → {}", current_ver.red(), new_line.clone().green());

                    if !dry_run {
                        lines[i] = new_line;
                    }
                    upgraded = true;
                }
            }
        }
    }

    if !upgraded {
        println!(
            "  {} All dependencies are already at the latest version",
            "✓".green()
        );
        return Ok(false);
    }

    // Write updated content back if not dry run
    if !dry_run {
        fs::write(&cargo_path, lines.join("\n"))?;
        println!(
            "  {} Dependencies upgraded in {}",
            "✅".green(),
            cargo_path.display()
        );
    } else {
        println!("  {} Dry run - no changes made", "ℹ️".blue());
    }

    Ok(upgraded)
}
