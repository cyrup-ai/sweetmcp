#[cfg(target_os = "macos")]
mod macos_helper {
    use std::env;
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    // We'll use tauri-bundler after build for the main daemon signing

    pub fn build_and_sign_helper() -> Result<(), Box<dyn std::error::Error>> {
        let out_dir = PathBuf::from(env::var("OUT_DIR")?);
        let helper_dir = out_dir.join("SweetMCPHelper.app");
        
        // Create app bundle structure
        let contents_dir = helper_dir.join("Contents");
        let macos_dir = contents_dir.join("MacOS");
        fs::create_dir_all(&macos_dir)?;
        
        // Create helper executable
        let helper_path = macos_dir.join("SweetMCPHelper");
        create_helper_executable(&helper_path)?;
        
        // Create Info.plist
        let info_plist_path = contents_dir.join("Info.plist");
        create_info_plist(&info_plist_path)?;
        
        // Sign the helper app
        sign_helper_app(&helper_dir)?;
        
        // Create ZIP for embedding
        create_helper_zip(&helper_dir, &out_dir)?;
        
        Ok(())
    }
    
    fn create_helper_executable(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Create a minimal helper executable using cc
        let helper_code = r#"
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/wait.h>

int main(int argc, char *argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <command> [args...]\n", argv[0]);
        return 1;
    }
    
    // Execute the command with elevated privileges
    pid_t pid = fork();
    if (pid == 0) {
        // Child process
        execvp(argv[1], &argv[1]);
        perror("execvp failed");
        exit(1);
    } else if (pid > 0) {
        // Parent process
        int status;
        waitpid(pid, &status, 0);
        return WEXITSTATUS(status);
    } else {
        perror("fork failed");
        return 1;
    }
}
"#;
        
        let temp_dir = tempfile::tempdir()?;
        let src_path = temp_dir.path().join("helper.c");
        fs::write(&src_path, helper_code)?;
        
        // Compile the helper
        cc::Build::new()
            .file(&src_path)
            .opt_level(3)
            .compile("helper");
        
        // Link the helper executable
        let status = Command::new("cc")
            .args(&[
                "-o", path.to_str().unwrap(),
                &format!("{}/libhelper.a", env::var("OUT_DIR")?),
                "-framework", "ServiceManagement",
                "-framework", "Security",
            ])
            .status()?;
        
        if !status.success() {
            return Err("Failed to link helper executable".into());
        }
        
        Ok(())
    }
    
    fn create_info_plist(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let team_id = get_team_id()?;
        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.cyrupd.sweetmcp.helper</string>
    <key>CFBundleName</key>
    <string>SweetMCPHelper</string>
    <key>CFBundleExecutable</key>
    <string>SweetMCPHelper</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>SMAuthorizedClients</key>
    <array>
        <string>identifier "com.cyrupd.sweetmcp" and certificate leaf[subject.OU] = "{}"</string>
    </array>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>"#,
            team_id
        );
        
        fs::write(path, plist_content)?;
        Ok(())
    }
    
    fn get_team_id() -> Result<String, Box<dyn std::error::Error>> {
        // Try to get team ID from certificate
        if let Ok(cert_path) = env::var("APPLE_CERTIFICATE_PATH") {
            // For local development using the certificate file
            let output = Command::new("security")
                .args(&["find-certificate", "-c", "Developer ID Application", "-p"])
                .output()?;
            
            if output.status.success() {
                let cert_data = String::from_utf8_lossy(&output.stdout);
                // Parse team ID from certificate data
                // This is a simplified version - in production you'd parse the certificate properly
                if let Some(start) = cert_data.find("OU=") {
                    if let Some(end) = cert_data[start+3..].find(char::is_whitespace) {
                        return Ok(cert_data[start+3..start+3+end].to_string());
                    }
                }
            }
        }
        
        // Fallback to environment variable
        env::var("APPLE_TEAM_ID").or_else(|_| Ok("PLACEHOLDER".to_string()))
    }
    
    fn sign_helper_app(helper_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let cert_path = "/Users/davidmaple/.ssh/development.cer";
        
        // Check if certificate exists
        if !Path::new(cert_path).exists() {
            // In CI or if cert doesn't exist, try environment variables
            if let Ok(identity) = env::var("APPLE_SIGNING_IDENTITY") {
                sign_with_identity(helper_dir, &identity)?;
            } else {
                // Ad-hoc signing for development
                sign_with_identity(helper_dir, "-")?;
            }
        } else {
            // Import certificate and sign
            import_and_sign(helper_dir, cert_path)?;
        }
        
        Ok(())
    }
    
    fn import_and_sign(helper_dir: &Path, cert_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Create temporary keychain for signing
        let keychain_name = format!("sweetmcp-build-{}.keychain", std::process::id());
        let keychain_password = "temp-keychain-password";
        
        // Create keychain
        Command::new("security")
            .args(&["create-keychain", "-p", keychain_password, &keychain_name])
            .status()?;
        
        // Import certificate
        let status = Command::new("security")
            .args(&[
                "import", cert_path,
                "-k", &keychain_name,
                "-T", "/usr/bin/codesign",
                "-T", "/usr/bin/security",
            ])
            .status()?;
        
        if !status.success() {
            // Clean up keychain
            let _ = Command::new("security")
                .args(&["delete-keychain", &keychain_name])
                .status();
            return Err("Failed to import certificate".into());
        }
        
        // Get signing identity from the certificate
        let output = Command::new("security")
            .args(&["find-identity", "-v", "-p", "codesigning", &keychain_name])
            .output()?;
        
        let identity = if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Parse the identity from the output
            output_str.lines()
                .find(|line| line.contains("Developer ID Application"))
                .and_then(|line| line.split('"').nth(1))
                .unwrap_or("-")
                .to_string()
        } else {
            "-".to_string()
        };
        
        // Sign with the identity
        sign_with_identity(helper_dir, &identity)?;
        
        // Clean up keychain
        let _ = Command::new("security")
            .args(&["delete-keychain", &keychain_name])
            .status();
        
        Ok(())
    }
    
    fn sign_with_identity(helper_dir: &Path, identity: &str) -> Result<(), Box<dyn std::error::Error>> {
        let status = Command::new("codesign")
            .args(&[
                "--force",
                "--sign", identity,
                "--options", "runtime",
                "--timestamp",
                "--deep",
                "--verbose",
                helper_dir.to_str().unwrap(),
            ])
            .status()?;
        
        if !status.success() {
            return Err(format!("Failed to sign helper app with identity: {}", identity).into());
        }
        
        // Verify signature
        let verify_status = Command::new("codesign")
            .args(&[
                "--verify",
                "--deep",
                "--strict",
                "--verbose=2",
                helper_dir.to_str().unwrap(),
            ])
            .status()?;
        
        if !verify_status.success() {
            return Err("Helper app signature verification failed".into());
        }
        
        Ok(())
    }
    
    fn create_helper_zip(helper_dir: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        use std::io::BufWriter;
        use zip::write::FileOptions;
        use zip::CompressionMethod;
        
        let zip_path = out_dir.join("SweetMCPHelper.app.zip");
        let file = fs::File::create(&zip_path)?;
        let mut zip = zip::ZipWriter::new(BufWriter::new(file));
        
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o755);
        
        // Add all files from the helper app
        add_directory_to_zip(&mut zip, helper_dir, helper_dir.parent().unwrap(), &options)?;
        
        zip.finish()?;
        
        println!("cargo:rustc-env=HELPER_ZIP_PATH={}", zip_path.display());
        Ok(())
    }
    
    fn add_directory_to_zip<W: Write + std::io::Seek>(
        zip: &mut zip::ZipWriter<W>,
        dir: &Path,
        base: &Path,
        options: &FileOptions,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(base)?;
            
            if path.is_dir() {
                // Add directory
                zip.add_directory(relative_path.to_str().unwrap(), *options)?;
                add_directory_to_zip(zip, &path, base, options)?;
            } else {
                // Add file
                let mut file = fs::File::open(&path)?;
                zip.start_file(relative_path.to_str().unwrap(), *options)?;
                std::io::copy(&mut file, zip)?;
            }
        }
        Ok(())
    }
}

fn main() {
    // Check for systemd on Linux
    #[cfg(target_os = "linux")]
    {
        if pkg_config::probe_library("libsystemd").is_ok() {
            println!("cargo:rustc-cfg=feature=\"systemd_available\"");
        }
    }
    
    // Build and sign macOS helper app
    #[cfg(target_os = "macos")]
    {
        if let Err(e) = macos_helper::build_and_sign_helper() {
            eprintln!("Warning: Failed to build macOS helper app: {}", e);
            eprintln!("The daemon will still work but may require manual privilege escalation");
            
            // Create a placeholder ZIP file so the build doesn't fail
            let out_dir = std::env::var("OUT_DIR").unwrap();
            let zip_path = format!("{}/SweetMCPHelper.app.zip", out_dir);
            
            // Create empty ZIP
            use std::io::Write;
            let file = std::fs::File::create(&zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            zip.finish().unwrap();
        }
    }
}