// riovt/src/setup.rs
use std::process::Command;
use std::path::Path;
use std::fs; // For creating lib.rs later
use std::io::Write; // For writing to lib.rs later

fn run_command(dir: &str, cmd: &str, args: &[&str]) -> Result<(), String> {
    println!("Running command: {} {:?} in dir {}", cmd, args, dir);
    let output = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| format!("Failed to execute command '{}': {}", cmd, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Command '{} {:?}' failed in dir {}:\nStatus: {}\nStdout:\n{}\nStderr:\n{}",
            cmd, args, dir, output.status, stdout, stderr
        ));
    }
    println!("Command successful: {} {:?} for dir {}", cmd, args, dir);
    Ok(())
}

// Function to create/overwrite lib.rs (will be expanded later)
fn inject_library_file(rioterm_crate_path: &str) -> Result<(), String> {
    let lib_rs_path = Path::new(rioterm_crate_path).join("src").join("lib.rs");
    println!("Attempting to inject/create lib.rs at: {}", lib_rs_path.display());

    // Placeholder content for now
    let lib_rs_content = r#"
// Dynamically injected lib.rs for rioterm
pub fn hello_from_rioterm_lib() {
    println!("Hello from dynamically injected rioterm lib!");
}

// TODO: Expose dependencies from main.rs and configuration options
// pub use crate:: ... ;
"#;

    let parent_dir = lib_rs_path.parent().ok_or_else(|| "Failed to get parent directory for lib.rs".to_string())?;
    fs::create_dir_all(parent_dir)
        .map_err(|e| format!("Failed to create directory {}: {}", parent_dir.display(), e))?;

    let mut file = fs::File::create(&lib_rs_path)
        .map_err(|e| format!("Failed to create {}: {}", lib_rs_path.display(), e))?;
    
    file.write_all(lib_rs_content.as_bytes())
        .map_err(|e| format!("Failed to write to {}: {}", lib_rs_path.display(), e))?;

    println!("Successfully created/updated {}", lib_rs_path.display());
    Ok(())
}

// Function to ensure Cargo.toml has a lib target (will be expanded later)
fn ensure_cargo_lib_target(rioterm_crate_path: &str) -> Result<(), String> {
    let cargo_toml_path = Path::new(rioterm_crate_path).join("Cargo.toml");
    println!("Checking/Updating Cargo.toml for lib target at: {}", cargo_toml_path.display());
    // This is complex. Per CONVENTIONS.md, we should use `cargo` commands.
    // `cargo add --path . --crate-type lib` might not work as expected or might not exist.
    // Manually adding a [lib] section or ensuring `src/lib.rs` is picked up is typical.
    // For now, this is a placeholder. We'll investigate the best way according to conventions.
    // A simple `src/lib.rs` is often enough for Cargo to detect it if no explicit [[bin]] only section exists.
    // If Cargo.toml has:
    // [[bin]]
    // name = "rioterm"
    // path = "src/main.rs"
    //
    // We might need to add:
    // [lib]
    // name = "rioterm_lib" // Or just "rioterm" if the bin name is different or we want to share
    // path = "src/lib.rs"

    // For now, we'll just print a message. Actual modification needs careful handling.
    println!("TODO: Ensure {} has a [lib] target if necessary, using cargo commands if possible.", cargo_toml_path.display());
    // Example: Check if `[lib]` entry exists or if `crate-type` needs adjustment.
    // This might involve reading the TOML, and if modification is needed,
    // potentially calling `cargo` if a suitable command exists, or
    // guiding the user if manual edit (or a more specific cargo command) is required.
    Ok(())
}


fn main() -> Result<(), String> {
    // Path to the rio source directory within the project structure
    let rio_source_dir = "riovt/vendor/rio";
    // Path to the rioterm crate within the rio source directory
    let rioterm_crate_path = Path::new(rio_source_dir).join("frontends").join("rioterm");
    let rioterm_crate_path_str = rioterm_crate_path.to_str().ok_or("Invalid path for rioterm crate")?;

    // --- Git Operations ---
    println!("Starting Git operations in {}", rio_source_dir);
    if !Path::new(rio_source_dir).exists() {
        return Err(format!("RIO_SOURCE_DIR '{}' does not exist. Please check the path.", rio_source_dir));
    }
    if !Path::new(rio_source_dir).join(".git").exists() {
         return Err(format!("RIO_SOURCE_DIR '{}' does not appear to be a git repository.", rio_source_dir));
    }

    run_command(rio_source_dir, "git", &["reset", "--hard"])?;
    run_command(rio_source_dir, "git", &["clean", "-fd"])?;
    run_command(rio_source_dir, "git", &["fetch", "origin"])?;
    run_command(rio_source_dir, "git", &["merge", "origin/main"])?;
    println!("Git operations completed successfully.");

    // --- Dynamic Library Injection ---
    println!("\nStarting dynamic library injection for rioterm crate at {}", rioterm_crate_path_str);
    inject_library_file(rioterm_crate_path_str)?;
    ensure_cargo_lib_target(rioterm_crate_path_str)?;
    println!("Dynamic library injection steps initiated.");

    println!("\nSetup script finished.");
    Ok(())
}
