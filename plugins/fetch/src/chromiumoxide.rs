" use async_trait::async_trait;
use base64::Engine;
use chromiumoxide::{Browser, BrowserConfig, Page};
use futures::StreamExt;
use std::error::Error as StdError;
use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub enum ChromiumFetchError {
    Browser(String),
    Navigation(String),
    Screenshot(String),
    Content(String),
    Timeout(String),
}

impl fmt::Display for ChromiumFetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChromiumFetchError::Browser(e) => write!(f, "Browser error: {}", e),
            ChromiumFetchError::Navigation(e) => write!(f, "Navigation error: {}", e),
            ChromiumFetchError::Screenshot(e) => write!(f, "Screenshot error: {}", e),
            ChromiumFetchError::Content(e) => write!(f, "Content error: {}", e),
            ChromiumFetchError::Timeout(e) => write!(f, "Timeout error: {}", e),
        }
    }
}

impl StdError for ChromiumFetchError {}

pub struct FetchResult {
    pub content: String,
    pub screenshot_base64: String,
    pub content_type: String,
}

#[async_trait]
pub trait ContentFetcher {
    async fn fetch_content(&self, url: &str) -> Result<FetchResult, Box<dyn StdError + Send + Sync>>;
}

pub struct ChromiumFetcher;

impl ChromiumFetcher {
    // Create a new browser instance
    async fn create_browser() -> Result<(Browser, futures::channel::mpsc::Receiver<()>), ChromiumFetchError> {
        let config = BrowserConfig::builder()
            .viewport(Some((1280, 800)))
            .build()
            .map_err(|e| ChromiumFetchError::Browser(format!("Failed to build browser config: {}", e)))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| ChromiumFetchError::Browser(format!("Failed to launch browser: {}", e)))?;

        let disconnect_receiver = handler.take_disconnect_receiver().unwrap();
        
        // Spawn the handler
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    eprintln!("Browser event error: {}", e);
                }
            }
        });

        Ok((browser, disconnect_receiver))
    }

    // Take a screenshot of the page
    async fn take_screenshot(page: &Page) -> Result<String, ChromiumFetchError> {
        let screenshot_data = page
            .screenshot(None)
            .await
            .map_err(|e| ChromiumFetchError::Screenshot(format!("Failed to take screenshot: {}", e)))?;

        // Convert to base64
        let screenshot_base64 = base64::engine::general_purpose::STANDARD.encode(&screenshot_data);
        Ok(screenshot_base64)
    }

    // Get page content with scripts and styles removed
    async fn get_cleaned_content(page: &Page) -> Result<String, ChromiumFetchError> {
        // Execute JavaScript to get HTML content with script and style tags removed
        let js = r#"
        (function() {
            // Clone the document body to avoid modifying the actual page
            const clone = document.documentElement.cloneNode(true);
            
            // Remove script and style tags
            const scripts = clone.querySelectorAll('script');
            scripts.forEach(script => script.remove());
            
            const styles = clone.querySelectorAll('style');
            styles.forEach(style => style.remove());
            
            // Also remove style attributes from all elements
            const elements = clone.querySelectorAll('*');
            elements.forEach(el => el.removeAttribute('style'));
            
            return clone.outerHTML;
        })()
        "#;

        let content = page
            .evaluate(js)
            .await
            .map_err(|e| ChromiumFetchError::Content(format!("Failed to get page content: {}", e)))?
            .into_value::<String>()
            .map_err(|e| ChromiumFetchError::Content(format!("Failed to parse page content: {}", e)))?;

        Ok(content)
    }
}

#[async_trait]
impl ContentFetcher for ChromiumFetcher {
    async fn fetch_content(&self, url: &str) -> Result<FetchResult, Box<dyn StdError + Send + Sync>> {
        // Launch browser
        let (browser, _) = Self::create_browser().await?;

        // Create a new page
        let page = browser.new_page("")
            .await
            .map_err(|e| ChromiumFetchError::Browser(format!("Failed to create page: {}", e)))?;

        // Navigate to the URL with a timeout
        let navigation_result = tokio::time::timeout(
            Duration::from_secs(30),
            page.goto(url),
        ).await;

        // Check for timeout or navigation error
        match navigation_result {
            Ok(result) => {
                result.map_err(|e| ChromiumFetchError::Navigation(format!("Failed to navigate to URL: {}", e)))?;
            },
            Err(_) => {
                return Err(Box::new(ChromiumFetchError::Timeout(format!("Navigation to {} timed out", url))));
            }
        }

        // Wait for page to be fully loaded
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Take screenshot
        let screenshot_base64 = Self::take_screenshot(&page).await?;

        // Get content
        let content = Self::get_cleaned_content(&page).await?;

        // Get content type
        let content_type = page.content_type()
            .await
            .map_err(|e| ChromiumFetchError::Content(format!("Failed to get content type: {}", e)))?
            .unwrap_or_else(|| "text/html".to_string());

        // Close browser
        browser.close().await
            .map_err(|e| ChromiumFetchError::Browser(format!("Failed to close browser: {}", e)))?;

        Ok(FetchResult {
            content,
            screenshot_base64,
            content_type,
        })
    }
}
