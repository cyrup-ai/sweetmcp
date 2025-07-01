use anyhow::Result;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::ScreenshotParams;
use futures::StreamExt;
use std::sync::Arc;
use image::RgbImage;
use tokio::sync::mpsc::Sender;

/// Streams screenshots as RgbImage to the provided channel, given a URL.
pub async fn stream_screenshots(url: &str, tx: Sender<RgbImage>) -> Result<()> {
    // Create a temporary directory for Chrome
    let temp_dir = Arc::new(tempfile::Builder::new()
        .prefix("rio-ext-browser")
        .tempdir()?);
    let config = BrowserConfig::builder()
        .user_data_dir(temp_dir.path())
        .args(vec!["--no-sandbox", "--disable-dev-shm-usage"])
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build browser config: {}", e))?;
    let (mut browser, mut handler) = Browser::launch(config).await?;
    // Spawn handler for browser events
    tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            let _ = h;
        }
    });
    // Create a new page and navigate
    let page = browser.new_page(url).await?;
    // Take a screenshot (could be looped for periodic shots)
    let screenshot_data = page.screenshot(
        ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .full_page(true)
            .build(),
    ).await?;
    // Decode PNG to RgbImage
    let img = image::load_from_memory(&screenshot_data)?.to_rgb8();
    // Send to channel
    let _ = tx.send(img).await;
    
    // Properly close the browser to avoid background kill warning
    browser.close().await?;
    
    Ok(())
}

// (Legacy WebBrowser struct and impl removed for modular interface)
