#!/usr/bin/env cargo +nightly -Zscript
//! # Multi-Platform Build & Installer Script
//! 
//! This script builds the PTY application for multiple platforms and creates
//! appropriate installers for each target platform.
//! 
//! ## Usage
//! ```bash
//! # Build for current platform
//! cargo run --bin build-installers
//! 
//! # Build for all platforms (requires cross-compilation setup)
//! cargo run --bin build-installers -- --all-platforms
//! 
//! # Build for specific platform
//! cargo run --bin build-installers -- --platform macos-aarch64
//! ```

use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

#[derive(Debug, Clone)]
struct Target {
    name: &'static str,
    rust_target: &'static str,
    display_name: &'static str,
    installer_type: InstallerType,
    requires_cross: bool,
}

#[derive(Debug, Clone)]
enum InstallerType {
    AppBundle,  // macOS .app bundle
    Msi,        // Windows MSI installer
    Deb,        // Debian package
    Rpm,        // Red Hat package
    AppImage,   // Linux AppImage
    Dmg,        // macOS disk image
    Nsis,       // Windows NSIS installer
}

const TARGETS: &[Target] = &[
    Target {
        name: "macos-aarch64",
        rust_target: "aarch64-apple-darwin",
        display_name: "macOS Apple Silicon",
        installer_type: InstallerType::AppBundle,
        requires_cross: cfg!(not(all(target_os = "macos", target_arch = "aarch64"))),
    },
    Target {
        name: "macos-x86_64",
        rust_target: "x86_64-apple-darwin",
        display_name: "macOS Intel",
        installer_type: InstallerType::AppBundle,
        requires_cross: cfg!(not(all(target_os = "macos", target_arch = "x86_64"))),
    },
    Target {
        name: "windows-x86_64",
        rust_target: "x86_64-pc-windows-msvc",
        display_name: "Windows 64-bit",
        installer_type: InstallerType::Msi,
        requires_cross: cfg!(not(all(target_os = "windows", target_arch = "x86_64"))),
    },
    Target {
        name: "windows-aarch64",
        rust_target: "aarch64-pc-windows-msvc",
        display_name: "Windows ARM64",
        installer_type: InstallerType::Msi,
        requires_cross: cfg!(not(all(target_os = "windows", target_arch = "aarch64"))),
    },
    Target {
        name: "linux-x86_64",
        rust_target: "x86_64-unknown-linux-gnu",
        display_name: "Linux 64-bit",
        installer_type: InstallerType::AppImage,
        requires_cross: cfg!(not(all(target_os = "linux", target_arch = "x86_64"))),
    },
    Target {
        name: "linux-aarch64",
        rust_target: "aarch64-unknown-linux-gnu",
        display_name: "Linux ARM64",
        installer_type: InstallerType::AppImage,
        requires_cross: cfg!(not(all(target_os = "linux", target_arch = "aarch64"))),
    },
];

struct BuildContext {
    project_root: PathBuf,
    build_dir: PathBuf,
    dist_dir: PathBuf,
    version: String,
    sign_builds: bool,
    verbose: bool,
}

impl BuildContext {
    fn new() -> anyhow::Result<Self> {
        let project_root = env::current_dir()?;
        let build_dir = project_root.join("target").join("installers");
        let dist_dir = project_root.join("dist");
        
        // Extract version from Cargo.toml
        let cargo_toml = fs::read_to_string(project_root.join("Cargo.toml"))?;
        let version = extract_version(&cargo_toml)?;
        
        // Check if signing is enabled
        let sign_builds = env::var("ENABLE_SIGNING").unwrap_or_default() == "1" ||
                         Path::new("signing/signing-config.toml").exists();
        
        Ok(Self {
            project_root,
            build_dir,
            dist_dir,
            version,
            sign_builds,
            verbose: env::var("VERBOSE").unwrap_or_default() == "1",
        })
    }
    
    fn ensure_directories(&self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.build_dir)?;
        fs::create_dir_all(&self.dist_dir)?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut all_platforms = false;
    let mut specific_platform = None;
    
    // Parse command line arguments
    for (i, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--all-platforms" => all_platforms = true,
            "--platform" => {
                if let Some(platform) = args.get(i + 1) {
                    specific_platform = Some(platform.clone());
                }
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {}
        }
    }
    
    let ctx = BuildContext::new()?;
    ctx.ensure_directories()?;
    
    println!("🚀 Starting build process for PTY Terminal v{}", ctx.version);
    println!("📦 Project root: {}", ctx.project_root.display());
    println!("🔧 Build directory: {}", ctx.build_dir.display());
    println!("📋 Distribution directory: {}", ctx.dist_dir.display());
    
    if ctx.sign_builds {
        println!("🔐 Code signing enabled");
    }
    
    // Determine which targets to build
    let targets_to_build: Vec<&Target> = if all_platforms {
        TARGETS.iter().collect()
    } else if let Some(platform) = specific_platform {
        TARGETS.iter()
            .filter(|t| t.name == platform)
            .collect()
    } else {
        // Build for current platform only
        get_current_platform_targets()
    };
    
    if targets_to_build.is_empty() {
        eprintln!("❌ No valid targets found to build");
        return Err(anyhow::anyhow!("No targets specified"));
    }
    
    println!("🎯 Building for {} target(s):", targets_to_build.len());
    for target in &targets_to_build {
        println!("  • {} ({})", target.display_name, target.rust_target);
    }
    
    // Install required toolchains
    install_toolchains(&targets_to_build, &ctx)?;
    
    // Build each target
    let mut build_results = HashMap::new();
    for target in targets_to_build {
        println!("\n🔨 Building {}", target.display_name);
        match build_target(target, &ctx) {
            Ok(artifact_path) => {
                println!("✅ Successfully built {}", target.display_name);
                build_results.insert(target.name, Ok(artifact_path));
            }
            Err(e) => {
                eprintln!("❌ Failed to build {}: {}", target.display_name, e);
                build_results.insert(target.name, Err(e));
            }
        }
    }
    
    // Create installers
    println!("\n📦 Creating installers...");
    for target in &targets_to_build {
        if let Some(Ok(artifact_path)) = build_results.get(target.name) {
            match create_installer(target, artifact_path, &ctx) {
                Ok(installer_path) => {
                    println!("✅ Created installer: {}", installer_path.display());
                }
                Err(e) => {
                    eprintln!("❌ Failed to create installer for {}: {}", target.display_name, e);
                }
            }
        }
    }
    
    // Generate build manifest
    generate_build_manifest(&build_results, &ctx)?;
    
    println!("\n🎉 Build process completed!");
    println!("📋 Results summary:");
    
    let successful = build_results.values().filter(|r| r.is_ok()).count();
    let failed = build_results.len() - successful;
    
    println!("  • ✅ Successful builds: {}", successful);
    if failed > 0 {
        println!("  • ❌ Failed builds: {}", failed);
    }
    
    println!("📂 Distribution files available in: {}", ctx.dist_dir.display());
    
    Ok(())
}

fn get_current_platform_targets() -> Vec<&'static Target> {
    let current_target = format!("{}-{}", env::consts::ARCH, env::consts::OS);
    
    TARGETS.iter()
        .filter(|t| {
            match env::consts::OS {
                "macos" => t.name.starts_with("macos"),
                "windows" => t.name.starts_with("windows"),
                "linux" => t.name.starts_with("linux"),
                _ => false,
            } && match env::consts::ARCH {
                "x86_64" => t.name.contains("x86_64"),
                "aarch64" => t.name.contains("aarch64"),
                _ => false,
            }
        })
        .collect()
}

fn install_toolchains(targets: &[&Target], ctx: &BuildContext) -> anyhow::Result<()> {
    println!("🔧 Installing required toolchains...");
    
    for target in targets {
        if target.requires_cross {
            println!("  Installing target: {}", target.rust_target);
            let status = Command::new("rustup")
                .args(&["target", "add", target.rust_target])
                .status()?;
            
            if !status.success() {
                return Err(anyhow::anyhow!("Failed to install target: {}", target.rust_target));
            }
        }
    }
    
    // Install cross-compilation tools if needed
    let cross_targets: Vec<&&Target> = targets.iter()
        .filter(|t| t.requires_cross)
        .collect();
    
    if !cross_targets.is_empty() {
        // Check if cross is installed
        if Command::new("cross").arg("--version").status().is_err() {
            println!("  Installing cross-compilation tool...");
            let status = Command::new("cargo")
                .args(&["install", "cross", "--git", "https://github.com/cross-rs/cross"])
                .status()?;
            
            if !status.success() {
                eprintln!("⚠️  Failed to install cross - some builds may fail");
            }
        }
    }
    
    Ok(())
}

fn build_target(target: &Target, ctx: &BuildContext) -> anyhow::Result<PathBuf> {
    let binary_name = "rio-ext-test";
    let output_dir = ctx.build_dir.join(target.name);
    fs::create_dir_all(&output_dir)?;
    
    // Choose build command based on cross-compilation requirements
    let mut cmd = if target.requires_cross && Command::new("cross").arg("--version").status().is_ok() {
        let mut c = Command::new("cross");
        c.args(&["build", "--release", "--target", target.rust_target]);
        c
    } else {
        let mut c = Command::new("cargo");
        c.args(&["build", "--release", "--target", target.rust_target]);
        c
    };
    
    // Set environment variables for build
    cmd.env("CARGO_TARGET_DIR", &ctx.build_dir);
    cmd.current_dir(&ctx.project_root);
    
    if ctx.verbose {
        cmd.arg("--verbose");
    }
    
    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Build failed for target: {}", target.rust_target));
    }
    
    // Determine binary extension
    let binary_extension = if target.rust_target.contains("windows") { ".exe" } else { "" };
    let binary_filename = format!("{}{}", binary_name, binary_extension);
    
    // Locate the built binary
    let source_path = ctx.build_dir
        .join(target.rust_target)
        .join("release")
        .join(&binary_filename);
    
    if !source_path.exists() {
        return Err(anyhow::anyhow!("Built binary not found at: {}", source_path.display()));
    }
    
    // Copy binary to output directory
    let dest_path = output_dir.join(&binary_filename);
    fs::copy(&source_path, &dest_path)?;
    
    // Sign the binary if signing is enabled
    if ctx.sign_builds {
        sign_binary(&dest_path, target, ctx)?;
    }
    
    Ok(dest_path)
}

fn create_installer(target: &Target, binary_path: &Path, ctx: &BuildContext) -> anyhow::Result<PathBuf> {
    match target.installer_type {
        InstallerType::AppBundle => create_macos_app_bundle(target, binary_path, ctx),
        InstallerType::Msi => create_windows_msi(target, binary_path, ctx),
        InstallerType::Deb => create_debian_package(target, binary_path, ctx),
        InstallerType::AppImage => create_linux_appimage(target, binary_path, ctx),
        InstallerType::Dmg => create_macos_dmg(target, binary_path, ctx),
        InstallerType::Nsis => create_windows_nsis(target, binary_path, ctx),
        InstallerType::Rpm => create_rpm_package(target, binary_path, ctx),
    }
}

fn create_macos_app_bundle(target: &Target, binary_path: &Path, ctx: &BuildContext) -> anyhow::Result<PathBuf> {
    let app_name = "PTY Terminal";
    let bundle_name = format!("{}.app", app_name);
    let bundle_path = ctx.dist_dir.join(&bundle_name);
    
    // Remove existing bundle
    if bundle_path.exists() {
        fs::remove_dir_all(&bundle_path)?;
    }
    
    // Create bundle structure
    let contents_dir = bundle_path.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let resources_dir = contents_dir.join("Resources");
    
    fs::create_dir_all(&macos_dir)?;
    fs::create_dir_all(&resources_dir)?;
    
    // Copy binary
    let bundle_binary = macos_dir.join("rio-ext-test");
    fs::copy(binary_path, &bundle_binary)?;
    
    // Make binary executable
    Command::new("chmod")
        .args(&["+x", &bundle_binary.to_string_lossy()])
        .status()?;
    
    // Create Info.plist
    let info_plist = create_info_plist(app_name, &ctx.version, target)?;
    fs::write(contents_dir.join("Info.plist"), info_plist)?;
    
    // Copy icons
    copy_app_icons(&resources_dir)?;
    
    // Sign the app bundle if signing is enabled
    if ctx.sign_builds {
        sign_app_bundle(&bundle_path, ctx)?;
    }
    
    // Create DMG
    let dmg_path = create_dmg(&bundle_path, ctx)?;
    
    Ok(dmg_path)
}

fn create_windows_msi(target: &Target, binary_path: &Path, ctx: &BuildContext) -> anyhow::Result<PathBuf> {
    let installer_name = format!("pty-terminal-{}-{}.msi", ctx.version, target.name);
    let installer_path = ctx.dist_dir.join(&installer_name);
    
    // Create WiX configuration
    let wix_config = create_wix_config(target, binary_path, ctx)?;
    let wix_file = ctx.build_dir.join(format!("{}.wxs", target.name));
    fs::write(&wix_file, wix_config)?;
    
    // Check if WiX toolset is available
    if Command::new("candle").arg("--version").status().is_err() {
        return Err(anyhow::anyhow!("WiX toolset not found. Please install WiX v3.11 or later."));
    }
    
    // Compile with candle
    let obj_file = ctx.build_dir.join(format!("{}.wixobj", target.name));
    let status = Command::new("candle")
        .args(&[
            "-out", &obj_file.to_string_lossy(),
            &wix_file.to_string_lossy()
        ])
        .status()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("WiX compilation failed"));
    }
    
    // Link with light
    let status = Command::new("light")
        .args(&[
            "-out", &installer_path.to_string_lossy(),
            &obj_file.to_string_lossy()
        ])
        .status()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("WiX linking failed"));
    }
    
    // Sign the MSI if signing is enabled
    if ctx.sign_builds {
        sign_windows_installer(&installer_path, ctx)?;
    }
    
    Ok(installer_path)
}

fn create_linux_appimage(target: &Target, binary_path: &Path, ctx: &BuildContext) -> anyhow::Result<PathBuf> {
    let appimage_name = format!("PTY_Terminal-{}-{}.AppImage", ctx.version, target.name);
    let appimage_path = ctx.dist_dir.join(&appimage_name);
    
    // Create AppDir structure
    let appdir = ctx.build_dir.join(format!("{}.AppDir", target.name));
    fs::create_dir_all(&appdir)?;
    
    // Copy binary
    let app_binary = appdir.join("AppRun");
    fs::copy(binary_path, &app_binary)?;
    Command::new("chmod").args(&["+x", &app_binary.to_string_lossy()]).status()?;
    
    // Create .desktop file
    let desktop_content = create_desktop_file(ctx)?;
    fs::write(appdir.join("pty-terminal.desktop"), desktop_content)?;
    
    // Copy icon
    copy_linux_icon(&appdir)?;
    
    // Download and use appimagetool
    let appimage_tool = download_appimagetool(ctx)?;
    
    let status = Command::new(&appimage_tool)
        .args(&[&appdir.to_string_lossy(), &appimage_path.to_string_lossy()])
        .status()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("AppImage creation failed"));
    }
    
    Ok(appimage_path)
}

fn create_debian_package(target: &Target, binary_path: &Path, ctx: &BuildContext) -> anyhow::Result<PathBuf> {
    let package_name = format!("pty-terminal_{}-1_{}.deb", 
                              ctx.version,
                              if target.name.contains("aarch64") { "arm64" } else { "amd64" });
    let package_path = ctx.dist_dir.join(&package_name);
    
    // Create package structure
    let pkg_dir = ctx.build_dir.join(format!("deb-{}", target.name));
    let debian_dir = pkg_dir.join("DEBIAN");
    let usr_bin = pkg_dir.join("usr").join("bin");
    let usr_share = pkg_dir.join("usr").join("share");
    
    fs::create_dir_all(&debian_dir)?;
    fs::create_dir_all(&usr_bin)?;
    fs::create_dir_all(&usr_share)?;
    
    // Copy binary
    fs::copy(binary_path, usr_bin.join("pty-terminal"))?;
    
    // Create control file
    let control_content = create_debian_control(target, ctx)?;
    fs::write(debian_dir.join("control"), control_content)?;
    
    // Create desktop entry and icons
    let applications_dir = usr_share.join("applications");
    let icons_dir = usr_share.join("icons").join("hicolor").join("512x512").join("apps");
    fs::create_dir_all(&applications_dir)?;
    fs::create_dir_all(&icons_dir)?;
    
    let desktop_content = create_desktop_file(ctx)?;
    fs::write(applications_dir.join("pty-terminal.desktop"), desktop_content)?;
    
    // Copy icon
    copy_linux_icon(&icons_dir)?;
    
    // Build package
    let status = Command::new("dpkg-deb")
        .args(&["--build", &pkg_dir.to_string_lossy(), &package_path.to_string_lossy()])
        .status()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("Debian package creation failed"));
    }
    
    Ok(package_path)
}

fn create_rpm_package(target: &Target, binary_path: &Path, ctx: &BuildContext) -> anyhow::Result<PathBuf> {
    // Similar to Debian package but using rpmbuild
    // Implementation would be similar to create_debian_package but with RPM spec files
    // For brevity, returning a placeholder error
    Err(anyhow::anyhow!("RPM package creation not yet implemented"))
}

// Helper functions for signing
fn sign_binary(binary_path: &Path, target: &Target, ctx: &BuildContext) -> anyhow::Result<()> {
    if target.rust_target.contains("darwin") {
        sign_macos_binary(binary_path, ctx)
    } else if target.rust_target.contains("windows") {
        sign_windows_binary(binary_path, ctx)
    } else {
        // Linux binaries typically don't need signing
        Ok(())
    }
}

fn sign_macos_binary(binary_path: &Path, ctx: &BuildContext) -> anyhow::Result<()> {
    let signing_script = ctx.project_root.join("signing").join("macos-sign.sh");
    if signing_script.exists() {
        let status = Command::new("sh")
            .args(&[&signing_script.to_string_lossy(), &binary_path.to_string_lossy()])
            .status()?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("macOS binary signing failed"));
        }
    }
    Ok(())
}

fn sign_windows_binary(binary_path: &Path, ctx: &BuildContext) -> anyhow::Result<()> {
    let signing_script = ctx.project_root.join("signing").join("windows-sign.ps1");
    if signing_script.exists() {
        let status = Command::new("powershell")
            .args(&["-File", &signing_script.to_string_lossy(), &binary_path.to_string_lossy()])
            .status()?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Windows binary signing failed"));
        }
    }
    Ok(())
}

fn sign_app_bundle(bundle_path: &Path, ctx: &BuildContext) -> anyhow::Result<()> {
    let signing_script = ctx.project_root.join("signing").join("macos-sign.sh");
    if signing_script.exists() {
        let status = Command::new("sh")
            .args(&[&signing_script.to_string_lossy(), &bundle_path.to_string_lossy()])
            .status()?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("macOS app bundle signing failed"));
        }
    }
    Ok(())
}

fn sign_windows_installer(installer_path: &Path, ctx: &BuildContext) -> anyhow::Result<()> {
    let signing_script = ctx.project_root.join("signing").join("windows-sign.ps1");
    if signing_script.exists() {
        let status = Command::new("powershell")
            .args(&["-File", &signing_script.to_string_lossy(), &installer_path.to_string_lossy()])
            .status()?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Windows installer signing failed"));
        }
    }
    Ok(())
}

// Helper functions for creating configuration files
fn create_info_plist(app_name: &str, version: &str, target: &Target) -> anyhow::Result<String> {
    Ok(format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>rio-ext-test</string>
    <key>CFBundleIdentifier</key>
    <string>com.sweetmcp.pty-terminal</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundleDisplayName</key>
    <string>{}</string>
    <key>CFBundleVersion</key>
    <string>{}</string>
    <key>CFBundleShortVersionString</key>
    <string>{}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>LSMinimumSystemVersion</key>
    <string>{}</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>"#, 
           app_name, 
           app_name, 
           version, 
           version,
           if target.name.contains("aarch64") { "11.0" } else { "10.15" }))
}

fn create_wix_config(target: &Target, binary_path: &Path, ctx: &BuildContext) -> anyhow::Result<String> {
    let arch = if target.name.contains("aarch64") { "arm64" } else { "x64" };
    
    Ok(format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <Product Id="*" Name="PTY Terminal" Language="1033" Version="{}" 
             Manufacturer="SweetMCP" UpgradeCode="{{12345678-1234-1234-1234-123456789012}}">
        
        <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" Platform="{}" />
        
        <MediaTemplate EmbedCab="yes" />
        
        <Feature Id="ProductFeature" Title="PTY Terminal" Level="1">
            <ComponentGroupRef Id="ProductComponents" />
        </Feature>
        
        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="ProgramFiles64Folder">
                <Directory Id="INSTALLFOLDER" Name="PTY Terminal" />
            </Directory>
            <Directory Id="ProgramMenuFolder">
                <Directory Id="ApplicationProgramsFolder" Name="PTY Terminal" />
            </Directory>
        </Directory>
        
        <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
            <Component Id="MainExecutable" Guid="*">
                <File Id="PTYTerminalExe" Source="{}" KeyPath="yes" Checksum="yes" />
            </Component>
        </ComponentGroup>
        
        <Icon Id="icon.ico" SourceFile="assets/icon/icon.ico" />
        <Property Id="ARPPRODUCTICON" Value="icon.ico" />
        
    </Product>
</Wix>"#, ctx.version, arch, binary_path.display()))
}

fn create_desktop_file(ctx: &BuildContext) -> anyhow::Result<String> {
    Ok(format!(r#"[Desktop Entry]
Name=PTY Terminal
Comment=Advanced terminal emulator with sixel support
Exec=pty-terminal
Icon=pty-terminal
Terminal=false
Type=Application
Categories=System;TerminalEmulator;
Version={}
"#, ctx.version))
}

fn create_debian_control(target: &Target, ctx: &BuildContext) -> anyhow::Result<String> {
    let arch = if target.name.contains("aarch64") { "arm64" } else { "amd64" };
    
    Ok(format!(r#"Package: pty-terminal
Version: {}-1
Section: utils
Priority: optional
Architecture: {}
Depends: libc6
Maintainer: SweetMCP <info@sweetmcp.com>
Description: Advanced terminal emulator with sixel support
 PTY Terminal is a modern terminal emulator that supports sixel graphics,
 providing enhanced visual capabilities for terminal applications.
"#, ctx.version, arch))
}

// Utility functions
fn extract_version(cargo_toml: &str) -> anyhow::Result<String> {
    for line in cargo_toml.lines() {
        if line.trim().starts_with("version") {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let version = parts[1].trim().trim_matches('"');
                return Ok(version.to_string());
            }
        }
    }
    Err(anyhow::anyhow!("Version not found in Cargo.toml"))
}

fn copy_app_icons(resources_dir: &Path) -> anyhow::Result<()> {
    // Copy icon files if they exist
    let icon_sources = [
        "assets/icon/icon.icns",
        "assets/icon/icon-512x512.png",
    ];
    
    for icon_source in &icon_sources {
        let source = Path::new(icon_source);
        if source.exists() {
            let filename = source.file_name().unwrap();
            fs::copy(source, resources_dir.join(filename))?;
        }
    }
    
    Ok(())
}

fn copy_linux_icon(target_dir: &Path) -> anyhow::Result<()> {
    let icon_source = Path::new("assets/icon/icon-512x512.png");
    if icon_source.exists() {
        fs::copy(icon_source, target_dir.join("pty-terminal.png"))?;
    }
    Ok(())
}

fn create_dmg(app_bundle: &Path, ctx: &BuildContext) -> anyhow::Result<PathBuf> {
    let dmg_name = format!("PTY_Terminal-{}.dmg", ctx.version);
    let dmg_path = ctx.dist_dir.join(&dmg_name);
    
    // Remove existing DMG
    if dmg_path.exists() {
        fs::remove_file(&dmg_path)?;
    }
    
    let status = Command::new("hdiutil")
        .args(&[
            "create",
            "-volname", "PTY Terminal",
            "-srcfolder", &app_bundle.to_string_lossy(),
            "-ov",
            "-format", "UDZO",
            &dmg_path.to_string_lossy()
        ])
        .status()?;
    
    if !status.success() {
        return Err(anyhow::anyhow!("DMG creation failed"));
    }
    
    Ok(dmg_path)
}

fn download_appimagetool(ctx: &BuildContext) -> anyhow::Result<PathBuf> {
    let tool_path = ctx.build_dir.join("appimagetool");
    
    if !tool_path.exists() {
        let arch = if cfg!(target_arch = "aarch64") { "aarch64" } else { "x86_64" };
        let url = format!("https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-{}.AppImage", arch);
        
        let status = Command::new("wget")
            .args(&["-O", &tool_path.to_string_lossy(), &url])
            .status()?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Failed to download appimagetool"));
        }
        
        Command::new("chmod")
            .args(&["+x", &tool_path.to_string_lossy()])
            .status()?;
    }
    
    Ok(tool_path)
}

fn generate_build_manifest(build_results: &HashMap<&str, Result<PathBuf, anyhow::Error>>, ctx: &BuildContext) -> anyhow::Result<()> {
    let manifest_path = ctx.dist_dir.join("build-manifest.json");
    
    let mut manifest = serde_json::Map::new();
    manifest.insert("version".to_string(), serde_json::Value::String(ctx.version.clone()));
    manifest.insert("timestamp".to_string(), serde_json::Value::String(
        chrono::Utc::now().to_rfc3339()
    ));
    
    let mut builds = serde_json::Map::new();
    for (target_name, result) in build_results {
        let build_info = match result {
            Ok(path) => {
                let mut info = serde_json::Map::new();
                info.insert("status".to_string(), serde_json::Value::String("success".to_string()));
                info.insert("path".to_string(), serde_json::Value::String(path.to_string_lossy().to_string()));
                if let Ok(metadata) = fs::metadata(path) {
                    info.insert("size".to_string(), serde_json::Value::Number(
                        serde_json::Number::from(metadata.len())
                    ));
                }
                serde_json::Value::Object(info)
            }
            Err(e) => {
                let mut info = serde_json::Map::new();
                info.insert("status".to_string(), serde_json::Value::String("failed".to_string()));
                info.insert("error".to_string(), serde_json::Value::String(e.to_string()));
                serde_json::Value::Object(info)
            }
        };
        builds.insert(target_name.to_string(), build_info);
    }
    
    manifest.insert("builds".to_string(), serde_json::Value::Object(builds));
    
    let manifest_content = serde_json::to_string_pretty(&manifest)?;
    fs::write(manifest_path, manifest_content)?;
    
    Ok(())
}

fn print_help() {
    println!("PTY Terminal Build Script");
    println!("Usage: cargo run --bin build-installers [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  --all-platforms    Build for all supported platforms");
    println!("  --platform <name>  Build for specific platform");
    println!("  --help, -h         Show this help message");
    println!();
    println!("Supported platforms:");
    for target in TARGETS {
        println!("  {} - {}", target.name, target.display_name);
    }
    println!();
    println!("Environment variables:");
    println!("  ENABLE_SIGNING=1   Enable code signing");
    println!("  VERBOSE=1          Enable verbose output");
}

// Dependencies that need to be added to Cargo.toml
mod dependencies {
    /*
    [dependencies]
    anyhow = "1.0"
    serde_json = "1.0"
    chrono = { version = "0.4", features = ["serde"] }
    */
}