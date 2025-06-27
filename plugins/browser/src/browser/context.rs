use std::path::PathBuf;
use std::sync::Arc;

use chromiumoxide::{
    cdp::browser_protocol::target::CreateTargetParams,
    page::{Page, ScreenshotParams},
};
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::browser::{
    BrowserContext, BrowserContextConfig, BrowserError, BrowserPage, BrowserResult,
};
use crate::browser::browser::Browser;

/// Browser context implementation
pub struct BrowserContext {
    browser: Browser,
    config: BrowserContextConfig,
    pages: Arc<Mutex<Vec<BrowserPage>>>,
    current_page_index: Arc<Mutex<usize>>,
}

impl BrowserContext {
    /// Create a new browser context
    pub async fn new(
        browser: Browser,
        config: BrowserContextConfig,
    ) -> BrowserResult<crate::browser::BrowserContext> {
        let context = Self {
            browser,
            config,
            pages: Arc::new(Mutex::new(Vec::new())),
            current_page_index: Arc::new(Mutex::new(0)),
        };
        
        // Initialize first page
        let page = context.new_page().await?;
        let mut pages = context.pages.lock().await;
        pages.push(page);
        
        // Setup recording if configured
        if let Some(path) = &config.save_recording_path {
            debug!("Recording enabled with path: {}", path);
            let recording_dir = PathBuf::from(path);
            
            // Create directory if it doesn't exist
            if !recording_dir.exists() {
                std::fs::create_dir_all(&recording_dir)
                    .map_err(|e| BrowserError::IoError(e.to_string()))?;
            }
        }
        
        // Setup tracing if configured
        if let Some(path) = &config.trace_path {
            debug!("Tracing enabled with path: {}", path);
            let trace_dir = PathBuf::from(path);
            
            // Create directory if it doesn't exist
            if !trace_dir.exists() {
                std::fs::create_dir_all(&trace_dir)
                    .map_err(|e| BrowserError::IoError(e.to_string()))?;
            }
        }
        
        Ok(crate::browser::BrowserContext::Native(Arc::new(context)))
    }
    
    /// Create a new page in the context
    pub async fn new_page(&self) -> BrowserResult<BrowserPage> {
        let params = CreateTargetParams::builder()
            .url("about:blank")
            .width(self.config.browser_window_size.width)
            .height(self.config.browser_window_size.height)
            .build();
        
        let target = self.browser.chromium_browser()
            .create_target(params)
            .await
            .map_err(|e| BrowserError::PageCreationFailed(e.to_string()))?;
        
        let page = self.browser.chromium_browser()
            .connect_target(target)
            .await
            .map_err(|e| BrowserError::ConnectionFailed(e.to_string()))?;
        
        // Set default viewport if not disabled
        if !self.config.no_viewport {
            page.set_viewport(chromiumoxide::handler::viewport::Viewport {
                width: self.config.browser_window_size.width as u32,
                height: self.config.browser_window_size.height as u32,
                ..Default::default()
            })
            .await
            .map_err(|e| BrowserError::ConfigurationFailed(e.to_string()))?;
        }
        
        Ok(BrowserPage::new(page))
    }
    
    /// Get the current page
    pub async fn get_current_page(&self) -> BrowserResult<BrowserPage> {
        let pages = self.pages.lock().await;
        let current_index = *self.current_page_index.lock().await;
        
        if pages.is_empty() {
            return Err(BrowserError::NoPage);
        }
        
        if current_index >= pages.len() {
            return Err(BrowserError::InvalidPageIndex(current_index));
        }
        
        Ok(pages[current_index].clone())
    }
    
    /// Switch to a different page by index
    pub async fn switch_to_page(&self, index: usize) -> BrowserResult<()> {
        let pages = self.pages.lock().await;
        
        if index >= pages.len() {
            return Err(BrowserError::InvalidPageIndex(index));
        }
        
        let mut current_index = self.current_page_index.lock().await;
        *current_index = index;
        
        Ok(())
    }
    
    /// Close the context and all associated pages
    pub async fn close(&self) -> BrowserResult<()> {
        let mut pages = self.pages.lock().await;
        
        // Close all pages
        for page in pages.iter() {
            if let Err(e) = page.close().await {
                error!("Error closing page: {}", e);
            }
        }
        
        pages.clear();
        *self.current_page_index.lock().await = 0;
        
        Ok(())
    }
    
    /// Take a screenshot of the current page
    pub async fn screenshot(&self) -> BrowserResult<Vec<u8>> {
        let page = self.get_current_page().await?;
        page.screenshot(None).await
    }
    
    /// Navigate to a URL on the current page
    pub async fn navigate(&self, url: &str) -> BrowserResult<()> {
        let page = self.get_current_page().await?;
        page.navigate(url).await
    }
}
