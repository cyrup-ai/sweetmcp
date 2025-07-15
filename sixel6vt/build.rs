use std::env;

fn main() {
    // Platform-specific build configurations
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    
    match target_os.as_str() {
        "macos" => configure_macos(&target_arch),
        "windows" => configure_windows(&target_arch),
        "linux" => configure_linux(&target_arch),
        _ => println!("cargo:warning=Unsupported target OS: {}", target_os),
    }
    
    // Set common build flags
    println!("cargo:rustc-env=TARGET_OS={}", target_os);
    println!("cargo:rustc-env=TARGET_ARCH={}", target_arch);
}

fn configure_macos(arch: &str) {
    println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.15");
    
    // Architecture-specific settings
    match arch {
        "x86_64" => {
            println!("cargo:rustc-link-arg=-mmacosx-version-min=10.15");
            println!("cargo:rustc-link-arg=-arch");
            println!("cargo:rustc-link-arg=x86_64");
        }
        "aarch64" => {
            println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=11.0");
            println!("cargo:rustc-link-arg=-mmacosx-version-min=11.0");
            println!("cargo:rustc-link-arg=-arch");
            println!("cargo:rustc-link-arg=arm64");
        }
        _ => println!("cargo:warning=Unsupported macOS architecture: {}", arch),
    }
    
    // macOS specific frameworks and libraries
    println!("cargo:rustc-link-lib=framework=Security");
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    println!("cargo:rustc-link-lib=framework=SystemConfiguration");
    
    // Code signing preparation
    if env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "release" {
        println!("cargo:rustc-env=ENABLE_CODE_SIGNING=1");
    }
}

fn configure_windows(arch: &str) {
    // Windows-specific settings
    println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
    
    match arch {
        "x86_64" => {
            println!("cargo:rustc-link-arg=/MACHINE:X64");
        }
        "aarch64" => {
            println!("cargo:rustc-link-arg=/MACHINE:ARM64");
        }
        _ => println!("cargo:warning=Unsupported Windows architecture: {}", arch),
    }
    
    // Windows libraries
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=advapi32");
    
    // Authenticode signing preparation
    if env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "release" {
        println!("cargo:rustc-env=ENABLE_AUTHENTICODE_SIGNING=1");
    }
}

fn configure_linux(arch: &str) {
    // Linux-specific settings
    match arch {
        "x86_64" => {
            // x86_64 specific optimizations
            println!("cargo:rustc-env=TARGET_CPU=x86-64");
        }
        "aarch64" => {
            // ARM64 specific optimizations
            println!("cargo:rustc-env=TARGET_CPU=generic");
        }
        _ => println!("cargo:warning=Unsupported Linux architecture: {}", arch),
    }
    
    // Linux libraries
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=dl");
    println!("cargo:rustc-link-lib=m");
    
    // Package signing preparation
    if env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "release" {
        println!("cargo:rustc-env=ENABLE_PACKAGE_SIGNING=1");
    }
}