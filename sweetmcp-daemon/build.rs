#[cfg(target_os = "macos")]
mod macos_helper {
    use std::env;
    use std::fs;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use zip::write::FileOptions;
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
#include <signal.h>
#include <errno.h>
#ifdef __APPLE__
#include <libproc.h>
#endif

#define SCRIPT_MAX_SIZE 1048576  // 1MB max script size
#define TIMEOUT_SECONDS 300      // 5 minute timeout

// Signal handler for timeout
void timeout_handler(int sig) {
    fprintf(stderr, "Helper: Script execution timed out after %d seconds\n", TIMEOUT_SECONDS);
    exit(124); // Standard timeout exit code
}

int main(int argc, char *argv[]) {
    // Verify parent process is sweetmcp-daemon
    pid_t parent_pid = getppid();
    char parent_path[1024];
    snprintf(parent_path, sizeof(parent_path), "/proc/%d/exe", parent_pid);
    
    // On macOS, use different approach to get parent process path
    #ifdef __APPLE__
    char pathbuf[PROC_PIDPATHINFO_MAXSIZE];
    if (proc_pidpath(parent_pid, pathbuf, sizeof(pathbuf)) <= 0) {
        fprintf(stderr, "Helper: Failed to get parent process path\n");
        return 1;
    }
    #else
    char pathbuf[1024];
    ssize_t len = readlink(parent_path, pathbuf, sizeof(pathbuf) - 1);
    if (len == -1) {
        fprintf(stderr, "Helper: Failed to get parent process path\n");
        return 1;
    }
    pathbuf[len] = '\0';
    #endif
    
    // Check if parent is sweetmcp-daemon
    if (!strstr(pathbuf, "sweetmcp-daemon")) {
        fprintf(stderr, "Helper: Unauthorized parent process: %s\n", pathbuf);
        return 1;
    }
    
    // Read shell script from stdin
    char *script_buffer = malloc(SCRIPT_MAX_SIZE);
    if (!script_buffer) {
        fprintf(stderr, "Helper: Failed to allocate memory for script\n");
        return 1;
    }
    
    size_t total_read = 0;
    ssize_t bytes_read;
    
    // Read script from stdin
    while ((bytes_read = read(STDIN_FILENO, script_buffer + total_read, 
                              SCRIPT_MAX_SIZE - total_read - 1)) > 0) {
        total_read += bytes_read;
        if (total_read >= SCRIPT_MAX_SIZE - 1) {
            fprintf(stderr, "Helper: Script too large (max %d bytes)\n", SCRIPT_MAX_SIZE);
            free(script_buffer);
            return 1;
        }
    }
    
    if (bytes_read < 0) {
        perror("Helper: Failed to read script from stdin");
        free(script_buffer);
        return 1;
    }
    
    script_buffer[total_read] = '\0';
    
    // Set up timeout
    signal(SIGALRM, timeout_handler);
    alarm(TIMEOUT_SECONDS);
    
    // Execute the script using system()
    // This allows full shell script execution with pipes, redirects, etc.
    int result = system(script_buffer);
    
    // Cancel timeout
    alarm(0);
    
    free(script_buffer);
    
    if (result == -1) {
        perror("Helper: Failed to execute script");
        return 127;
    }
    
    // Return the exit status from system()
    if (WIFEXITED(result)) {
        return WEXITSTATUS(result);
    } else if (WIFSIGNALED(result)) {
        fprintf(stderr, "Helper: Script terminated by signal %d\n", WTERMSIG(result));
        return 128 + WTERMSIG(result);
    }
    
    return 1; // Shouldn't reach here
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
            .args([
                "-o",
                path.to_str().unwrap(),
                &format!("{}/libhelper.a", env::var("OUT_DIR")?),
                "-framework",
                "ServiceManagement",
                "-framework",
                "Security",
            ])
            .status()?;

        if !status.success() {
            return Err("Failed to link helper executable".into());
        }

        Ok(())
    }

    fn create_info_plist(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let plist_content = r#"<?xml version="1.0" encoding="UTF-8"?>
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
        <string>identifier "com.cyrupd.sweetmcp"</string>
    </array>
    <key>LSUIElement</key>
    <false/>
    <key>CFBundleDisplayName</key>
    <string>SweetMCP Helper</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2024 Cyrupd. This helper performs privileged installation tasks for SweetMCP.</string>
</dict>
</plist>"#;

        fs::write(path, plist_content)?;
        Ok(())
    }

    // get_team_id function removed - now using Tauri's signing infrastructure

    fn sign_helper_app(helper_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        use tauri_macos_sign::Keychain;
        
        // Create ad-hoc keychain for distributed deployment
        // Ad-hoc signing (identity = "-") requires no certificate and works for distribution
        let keychain = Keychain::with_signing_identity("-");
        
        // Sign the helper app with hardened runtime enabled
        // This is the complete Tauri signing approach - no manual codesign calls
        keychain.sign(
            helper_dir,
            None, // No custom entitlements needed for helper
            true, // Enable hardened runtime for security
        )?;
        
        // Verify the signing worked by checking for _CodeSignature
        let code_signature = helper_dir.join("Contents/_CodeSignature/CodeResources");
        if !code_signature.exists() {
            return Err("Failed to create code signature - _CodeSignature missing".into());
        }
        
        println!("cargo:warning=Helper app signed successfully using Tauri signing infrastructure");
        Ok(())
    }

    // Legacy manual signing functions removed - now using Tauri's signing infrastructure

    fn create_helper_zip(
        helper_dir: &Path,
        out_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        options: &FileOptions<'static, ()>,
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
            eprintln!("Warning: Failed to build macOS helper app: {e}");
            eprintln!("Installation will use osascript for privilege escalation");

            // Create a placeholder ZIP file so the build doesn't fail
            let out_dir = std::env::var("OUT_DIR").unwrap();
            let zip_path = format!("{out_dir}/SweetMCPHelper.app.zip");

            // Create empty ZIP
            let file = std::fs::File::create(&zip_path).unwrap();
            let zip = zip::ZipWriter::new(file);
            zip.finish().unwrap();
        }
    }
}
