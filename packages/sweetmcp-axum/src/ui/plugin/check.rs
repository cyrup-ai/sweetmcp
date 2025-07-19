use std::{
    fs,
    io::{Write, stdout},
    path::Path,
    process::Command,
};

use clap::Args;
use ratatui::style::Stylize;

#[derive(Args, Debug)]
pub struct CheckArgs {
    /// Check all plugins in the plugins directory
    #[arg(short, long)]
    pub all: bool,

    /// Specific plugin name to check
    #[arg(short, long)]
    pub name: Option<String>,
}

pub fn check_plugins(args: &CheckArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Checking plugins...".bold().cyan());

    let project_root = std::env::current_dir()?;
    let plugins_dir = project_root.join("plugins");

    if !plugins_dir.exists() {
        return Err(format!("Plugins directory not found at {}", plugins_dir.display()).into());
    }

    // If a specific plugin is specified, only check that one
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

        check_plugin(&plugin_path)?; // Removed build argument
    } else if args.all {
        // Check all plugins
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
            let result = check_plugin(&path);

            if let Err(e) = result {
                failed_plugins.push((plugin_name, e.to_string()));
            }
        }

        // Report summary
        println!();
        println!("{}", "Check Summary".bold().underlined());

        if failed_plugins.is_empty() {
            println!("{}", "✅ All plugins passed checks".green());
        } else {
            println!(
                "{} {} {}",
                "❌".red(),
                failed_plugins.len(),
                "plugins failed checks:".red()
            );
            for (name, error) in failed_plugins {
                println!("  - {}: {}", name.bold(), error);
            }
            return Err("Some plugins failed checks".into());
        }
    } else {
        // No plugin specified and --all not set
        println!("No plugin specified. Use --name <plugin_name> or --all to check plugins.");
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

fn check_plugin(plugin_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let plugin_name = plugin_path
        .file_name()
        .ok_or_else(|| format!("Invalid plugin path: {}", plugin_path.display()))?
        .to_string_lossy();
    println!();
    println!(
        "{} {}",
        "Checking plugin:".bold(),
        plugin_name.to_string().green().bold()
    );

    // Check Cargo.toml exists
    let cargo_path = plugin_path.join("Cargo.toml");
    if !cargo_path.exists() {
        return Err(format!("Cargo.toml not found in {}", plugin_path.display()).into());
    }

    // Check src directory exists
    let src_path = plugin_path.join("src");
    if !src_path.exists() {
        return Err(format!("src directory not found in {}", plugin_path.display()).into());
    }

    // Check lib.rs exists
    let lib_path = src_path.join("lib.rs");
    if !lib_path.exists() {
        return Err(format!("lib.rs not found in {}", src_path.display()).into());
    }

    // Check plugin.rs exists (or pdk.rs)
    let plugin_rs_path = src_path.join("plugin.rs");
    let pdk_rs_path = src_path.join("pdk.rs");

    if !plugin_rs_path.exists() && !pdk_rs_path.exists() {
        return Err(format!(
            "Neither plugin.rs nor pdk.rs found in {}",
            src_path.display()
        )
        .into());
    }

    // Run cargo check
    print!("Running cargo check... ");
    stdout().flush()?;

    let check_output = Command::new("cargo")
        .args(["check", "--quiet"])
        .current_dir(plugin_path)
        .output()?;

    if check_output.status.success() {
        println!("{}", "✅".green());
    } else {
        println!("{}", "❌".red());
        let error = String::from_utf8_lossy(&check_output.stderr);
        return Err(format!("cargo check failed: {}", error).into());
    }

    println!(
        "{} {}",
        "Plugin".green(),
        plugin_name.to_string().green().bold()
    );
    println!("{}", "✅ All checks passed".green());

    Ok(())
}
