//! Build script for sweetmcp-daemon
//!
//! This build script handles cross-platform build tasks including macOS helper
//! app creation, code signing, and platform-specific optimizations.

// Include the build module
#[path = "src/build/mod.rs"]
mod build;

fn main() {
    build::main();
}