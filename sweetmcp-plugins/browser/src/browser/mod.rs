mod browser;
mod context;

pub use browser::Browser;
pub use context::BrowserContext;

use std::fmt;
use std::path::Path;
use std::sync::Arc;
use chromiumoxide::page::Page;
use thiserror::Error;

/// Browser window size configuration
#[derive(Debug, Clone, Copy)]
pub struct BrowserWindowSize {
    pub width: i32,
    pub height: i32,
}

impl Default for BrowserWindowSize {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
        }
    }
}

/// Browser context configuration
#[derive(Debug, Clone)]
pub struct BrowserContextConfig {
    pub browser_window_size: BrowserWindowSize,
    pub no_viewport: bool,
    pub save_recording_path: Option<String>,
    pub trace_path: Option<String>,
}

impl Default for BrowserContextConfig {
    fn default() -> Self {
        Self {
            browser_window_size: BrowserWindowSize::default(),
            no_viewport: false,
            save_recording_path: None,
            trace_path: None,
        }
    }
}

/// Error type for browser operations
#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Failed to launch browser: {0}")]
    LaunchError(String),
    
    #[error("Browser connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Failed to create page: {0}")]
    PageCreationFailed(String),
    
    #[error("Navigation failed: {0}")]
    NavigationFailed(String),
    
    #[error("Configuration failed: {0}")]
    ConfigurationFailed(String),
    
    #[error("Evaluation failed: {0}")]
    EvaluationFailed(String),
    
    #[error("Screenshot failed: {0}")]
    ScreenshotFailed(String),
    
    #[error("No page available")]
    NoPage,
    
    #[error("Invalid page index: {0}")]
    InvalidPageIndex(usize),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

/// Result type for browser operations
pub type BrowserResult<T> = Result<T, BrowserError>;

/// Browser context interface
#[derive(Clone)]
pub enum BrowserContext {
    Native(Arc<context::BrowserContext>),
    // Add other browser context types if needed
}

impl BrowserContext {
    /// Create a new page in the context
    pub async fn new_page(&self) -> BrowserResult<BrowserPage> {
        match self {
            BrowserContext::Native(ctx) => ctx.new_page().await,
        }
    }
    
    /// Get the current page
    pub async fn get_current_page(&self) -> BrowserResult<BrowserPage> {
        match self {
            BrowserContext::Native(ctx) => ctx.get_current_page().await,
        }
    }
    
    /// Close the context and all associated pages
    pub async fn close(&self) -> BrowserResult<()> {
        match self {
            BrowserContext::Native(ctx) => ctx.close().await,
        }
    }
    
    /// Take a screenshot of the current page
    pub async fn screenshot(&self) -> BrowserResult<Vec<u8>> {
        match self {
            BrowserContext::Native(ctx) => ctx.screenshot().await,
        }
    }
    
    /// Navigate to a URL on the current page
    pub async fn navigate(&self, url: &str) -> BrowserResult<()> {
        match self {
            BrowserContext::Native(ctx) => ctx.navigate(url).await,
        }
    }
}

/// Browser page wrapper
#[derive(Clone)]
pub struct BrowserPage {
    page: Page,
}

impl BrowserPage {
    /// Create a new browser page wrapper
    pub fn new(page: Page) -> Self {
        Self { page }
    }
    
    /// Navigate to a URL
    pub async fn navigate(&self, url: &str) -> BrowserResult<()> {
        self.page.goto(url).await
            .map_err(|e| BrowserError::NavigationFailed(e.to_string()))?;
        Ok(())
    }
    
    /// Get the current URL
    pub async fn url(&self) -> BrowserResult<String> {
        self.page.url()
            .map(|u| u.to_string())
            .ok_or_else(|| BrowserError::UnexpectedError("Failed to get URL".into()))
    }
    
    /// Get the current page title
    pub async fn title(&self) -> BrowserResult<String> {
        self.page.title().await
            .map_err(|e| BrowserError::UnexpectedError(format!("Failed to get title: {}", e)))
    }
    
    /// Get the page content
    pub async fn content(&self) -> BrowserResult<String> {
        self.page.content().await
            .map_err(|e| BrowserError::EvaluationFailed(e.to_string()))
    }
    
    /// Click on an element
    pub async fn click(&self, selector: &str) -> BrowserResult<()> {
        self.page.find_element(selector).await
            .map_err(|e| BrowserError::EvaluationFailed(format!("Element not found: {}", e)))?
            .click().await
            .map_err(|e| BrowserError::EvaluationFailed(format!("Click failed: {}", e)))?;
        Ok(())
    }
    
    /// Type text into an element
    pub async fn type_text(&self, selector: &str, text: &str) -> BrowserResult<()> {
        self.page.find_element(selector).await
            .map_err(|e| BrowserError::EvaluationFailed(format!("Element not found: {}", e)))?
            .type_str(text).await
            .map_err(|e| BrowserError::EvaluationFailed(format!("Type failed: {}", e)))?;
        Ok(())
    }
    
    /// Type directly with the keyboard
    pub async fn keyboard_type(&self, text: &str) -> BrowserResult<()> {
        for c in text.chars() {
            self.page.press_key(&c.to_string()).await
                .map_err(|e| BrowserError::EvaluationFailed(format!("Keyboard type failed: {}", e)))?;
        }
        Ok(())
    }
    
    /// Execute JavaScript on the page
    pub async fn evaluate(&self, script: &str) -> BrowserResult<String> {
        self.page.evaluate(script).await
            .map_err(|e| BrowserError::EvaluationFailed(e.to_string()))?
            .as_string()
            .ok_or_else(|| BrowserError::EvaluationFailed("Failed to get result as string".into()))
    }
    
    /// Get text content from an element
    pub async fn text_content(&self, selector: &str) -> BrowserResult<String> {
        self.page.find_element(selector).await
            .map_err(|e| BrowserError::EvaluationFailed(format!("Element not found: {}", e)))?
            .text().await
            .map_err(|e| BrowserError::EvaluationFailed(format!("Failed to get text: {}", e)))
    }
    
    /// Take a screenshot
    pub async fn screenshot(&self, path: Option<&str>) -> BrowserResult<Vec<u8>> {
        let data = self.page.screenshot(chromiumoxide::page::ScreenshotParams::builder()
            .full_page(true)
            .build())
            .await
            .map_err(|e| BrowserError::ScreenshotFailed(e.to_string()))?;
            
        // Save to file if path is provided
        if let Some(p) = path {
            let path = Path::new(p);
            std::fs::write(path, &data)
                .map_err(|e| BrowserError::IoError(e.to_string()))?;
        }
        
        Ok(data)
    }
    
    /// Close the page
    pub async fn close(&self) -> BrowserResult<()> {
        self.page.close(None).await
            .map_err(|e| BrowserError::UnexpectedError(format!("Failed to close page: {}", e)))?;
        Ok(())
    }
}
