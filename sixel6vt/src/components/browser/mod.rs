use crate::{browser, renderer};
use anyhow::Result;
use image;
use rio_backend::event::{EventProxy, RioEvent, RioEventType};
use rio_window::window::WindowId;
use tracing::info;
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;

pub struct BrowserPane {
    inner: Arc<RwLock<BrowserPaneInner>>,
}

struct BrowserPaneInner {
    current_url: String,
    title: String,
}

impl BrowserPane {
    pub async fn new(url: &str) -> Result<Self> {
        // For initial setup, just store the URL - actual navigation happens later
        Ok(Self {
            inner: Arc::new(RwLock::new(BrowserPaneInner {
                current_url: url.to_string(),
                title: "Web Page".to_string(),
            })),
        })
    }
    
    pub async fn current_url(&self) -> String {
        self.inner.read().await.current_url.clone()
    }
    
    pub async fn title(&self) -> String {
        self.inner.read().await.title.clone()
    }

    /// Captures current browser view, converts to Sixel, and sends to terminal PTY.
    /// The sixel will be displayed on the left half of the terminal.
    pub async fn update_and_send_sixel(&self, event_proxy: EventProxy, terminal_window_id: WindowId) -> Result<()> {
        let current_url = self.current_url().await;
        info!("Capturing screenshot from URL: {}", current_url);
        
        // Create a channel to receive the screenshot
        let (tx, mut rx) = mpsc::channel(1);
        
        // Stream screenshots from the browser
        let url = current_url.clone();
        tokio::spawn(async move {
            if let Err(e) = browser::stream_screenshots(&url, tx).await {
                tracing::error!("Failed to stream screenshots: {}", e);
            }
        });
        
        // Wait for the screenshot
        if let Some(mut image) = rx.recv().await {
            info!("Received screenshot data...");
            
            // Calculate the appropriate size for the left half of the terminal
            // We aim to keep the aspect ratio while fitting the image into the left half
            let (width, height) = (image.width(), image.height());
            let aspect_ratio = width as f32 / height as f32;
            
            // Resize to fit the left half of the screen - make width about 50% of terminal width
            // Since the image will be displayed in the terminal which is already on the right half,
            // we need to make sure it doesn't take up too much space
            let target_width = (width as f32 * 0.75) as u32; // Scale down to 75% of original width
            let target_height = (target_width as f32 / aspect_ratio) as u32;
            
            // Resize the image to the target size (optional - the sixel renderer can scale too)
            if width > target_width {
                image = image::imageops::resize(&image, target_width, target_height, 
                    image::imageops::FilterType::Lanczos3);
                info!("Resized image to {}x{} for display in left half", target_width, target_height);
            }

            // Convert the image to Sixel
            info!("Encoding image to Sixel...");
            let sixel = renderer::encode_sixel(&image);
            info!("Sixel data generated ({} bytes). Sending to terminal...", sixel.len());

            // Position cursor at beginning of line before sending the Sixel
            // This helps ensure the Sixel stays in the left half of the terminal
            let position_cursor = "\x1B[1;1H"; // Move cursor to top-left
            
            // Send the cursor position + Sixel data to the terminal PTY input
            let payload = RioEvent::PtyWrite(format!("{}{}", position_cursor, sixel));
            event_proxy.send_event(RioEventType::Rio(payload), terminal_window_id);
        } else {
            tracing::error!("Failed to receive screenshot from browser");
        }

        Ok(())
    }

    pub async fn navigate(&self, url: &str) -> Result<(String, String)> {
        // Update the URL and return it
        let mut inner = self.inner.write().await;
        inner.current_url = url.to_string();
        // For now, we don't fetch the title from the browser
        // This could be enhanced later
        inner.title = "Web Page".to_string();
        
        // Return title and URL so caller can properly update state
        Ok((inner.title.clone(), inner.current_url.clone()))
    }
    
    /// Update the current URL value and store in browser state - used in navigation
    pub async fn set_url(&self, url: String) {
        let mut inner = self.inner.write().await;
        inner.current_url = url;
    }
    
    /// Update the title value from page metadata - used for window titling
    pub async fn set_title(&self, title: String) {
        let mut inner = self.inner.write().await;
        inner.title = title;
    }
}