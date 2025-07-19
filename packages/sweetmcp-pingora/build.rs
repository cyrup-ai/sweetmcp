use std::process::Command;

fn main() {
    // Get git commit hash
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "dev".to_string());

    // Check if working directory is dirty
    let is_dirty = Command::new("git")
        .args(["diff", "--quiet"])
        .status()
        .map(|status| !status.success())
        .unwrap_or(false)
        || Command::new("git")
            .args(["diff", "--cached", "--quiet"])
            .status()
            .map(|status| !status.success())
            .unwrap_or(false);

    // Get target architecture
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    let arch = if target.contains("x86_64") {
        "x86_64"
    } else if target.contains("aarch64") {
        "aarch64"
    } else if target.contains("arm") {
        "arm"
    } else {
        "unknown"
    };

    // Build the BUILD_ID
    let build_id = if is_dirty {
        format!("{git_hash}-dirty-{arch}")
    } else {
        format!("{git_hash}-{arch}")
    };

    println!("cargo:rustc-env=BUILD_ID={build_id}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}
