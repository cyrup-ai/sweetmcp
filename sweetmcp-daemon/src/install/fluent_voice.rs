use std::process::Command;
use std::fs;
use log::info;
use anyhow::{Context, Result};

/// Install fluent voice from git repository
pub async fn install_fluent_voice(fluent_voice_dir: &std::path::Path) -> Result<()> {
    clone_from_git(fluent_voice_dir).await
}

/// Clone from git repository with retries
async fn clone_from_git(fluent_voice_dir: &std::path::Path) -> Result<()> {
    const MAX_RETRIES: u32 = 3;
    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES {
        if attempt > 1 {
            info!("Retrying git clone (attempt {}/{})", attempt, MAX_RETRIES);
            // Brief delay before retry
            #[cfg(feature = "runtime")]
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            #[cfg(not(feature = "runtime"))]
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        let output = Command::new("git")
            .args(&[
                "clone",
                "https://github.com/cyrup-ai/fluent-voice.git",
                fluent_voice_dir.to_str().ok_or_else(|| {
                    anyhow::anyhow!("fluent-voice directory path contains invalid UTF-8")
                })?,
            ])
            .output()
            .context("Failed to execute git clone")?;

        if output.status.success() {
            info!("Successfully cloned fluent-voice repository");
            return Ok(());
        }

        let error_msg = String::from_utf8_lossy(&output.stderr);
        last_error = Some(error_msg.to_string());

        // Clean up failed attempt
        if fluent_voice_dir.exists() {
            let _ = fs::remove_dir_all(fluent_voice_dir);
        }
    }

    Err(anyhow::anyhow!(
        "Failed to clone fluent-voice after {} attempts. Last error: {}",
        MAX_RETRIES,
        last_error.unwrap_or_else(|| "Unknown error".to_string())
    ))
}
