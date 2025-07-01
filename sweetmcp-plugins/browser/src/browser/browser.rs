use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::{
    browser::{Browser as ChromiumBrowser, BrowserConfig as ChromiumConfig},
    cdp::browser_protocol::target::CreateTargetParams,
    handler::viewport::Viewport,
};
use futures::StreamExt;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, error, info};

use crate::browser::{
    BrowserConfig, BrowserContext, BrowserContextConfig, BrowserError, BrowserResult,
};
use crate::browser::context::BrowserContext;

/// Browser implementation that manages a chromiumoxide browser instance
#[derive(Clone)]
pub struct Browser {
    browser: Arc<ChromiumBrowser>,
    config: BrowserConfig,
}

impl Browser {
    /// Create a new browser instance
    pub async fn new(config: BrowserConfig) -> BrowserResult<Self> {
        let browser = if let Some(chrome_path) = &config.chrome_instance_path {
            Self::setup_browser_with_instance(chrome_path, &config.extra_chromium_args).await?
        } else {
            Self::setup_browser(&config).await?
        };
        
        Ok(Self { 
            browser: Arc::new(browser),
            config,
        })
    }
    
    /// Create a new browser context
    pub async fn new_context(
        &self,
        config: BrowserContextConfig,
    ) -> BrowserResult<BrowserContext> {
        let context = BrowserContext::new(self.clone(), config).await?;
        Ok(context)
    }
    
    /// Close the browser instance and all associated resources
    pub async fn close(&self) -> BrowserResult<()> {
        if let Err(e) = self.browser.close().await {
            error!("Error closing browser: {}", e);
            return Err(BrowserError::CloseFailed(e.to_string()));
        }
        Ok(())
    }
    
    /// Get a reference to the underlying chromiumoxide browser
    pub fn chromium_browser(&self) -> &ChromiumBrowser {
        &self.browser
    }
    
    /// Set up a browser instance using Chrome path
    async fn setup_browser_with_instance(
        chrome_path: &str,
        extra_args: &[String],
    ) -> BrowserResult<ChromiumBrowser> {
        // Check if a browser is already running on debug port
        if let Ok(resp) = reqwest::get("http://localhost:9222/json/version").await {
            if resp.status().is_success() {
                info!("Reusing existing Chrome instance");
                
                // Connect to existing Chrome instance
                let cdp_url = "http://localhost:9222";
                let (browser, mut handler) = ChromiumBrowser::connect(cdp_url)
                    .await
                    .map_err(|e| BrowserError::ConnectionFailed(e.to_string()))?;
                
                // Spawn a task to handle browser events
                tokio::spawn(async move {
                    while let Some(h) = handler.next().await {
                        if let Err(e) = h {
                            error!("Browser handler error: {}", e);
                        }
                    }
                });
                
                return Ok(browser);
            }
        }
        
        debug!("Starting new Chrome instance at {}", chrome_path);
        
        // Start Chrome in debug mode
        let mut args = vec!["--remote-debugging-port=9222".to_string()];
        args.extend_from_slice(extra_args);
        
        let _process = Command::new(chrome_path)
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;
        
        // Wait for Chrome to start (retry connection)
        for attempt in 1..=10 {
            if let Ok(resp) = reqwest::get("http://localhost:9222/json/version").await {
                if resp.status().is_success() {
                    break;
                }
            }
            
            if attempt == 10 {
                return Err(BrowserError::ConnectionFailed(
                    "Failed to connect to Chrome after 10 attempts".into(),
                ));
            }
            
            sleep(Duration::from_millis(500)).await;
        }
        
        // Connect to the started Chrome instance
        let cdp_url = "http://localhost:9222";
        let (browser, mut handler) = ChromiumBrowser::connect(cdp_url)
            .await
            .map_err(|e| BrowserError::ConnectionFailed(e.to_string()))?;
        
        // Spawn a task to handle browser events
        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if let Err(e) = h {
                    error!("Browser handler error: {}", e);
                }
            }
        });
        
        Ok(browser)
    }
    
    /// Set up a standard browser instance
    async fn setup_browser(config: &BrowserConfig) -> BrowserResult<ChromiumBrowser> {
        let viewport = Viewport {
            width: 1280,
            height: 720,
            ..Default::default()
        };
        
        let mut browser_config = ChromiumConfig::builder()
            .viewport(viewport)
            .window_size(1280, 720);
        
        // Apply configuration options
        if config.headless {
            browser_config = browser_config.headless();
        }
        
        if config.disable_security {
            browser_config = browser_config
                .arg("--disable-web-security")
                .arg("--disable-features=IsolateOrigins,site-per-process");
        }
        
        // Add any extra arguments
        for arg in &config.extra_chromium_args {
            browser_config = browser_config.arg(arg);
        }
        
        // Launch the browser
        let (browser, mut handler) = ChromiumBrowser::launch(browser_config.build())
            .await
            .map_err(|e| BrowserError::LaunchFailed(e.to_string()))?;
        
        // Spawn a task to handle browser events
        tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if let Err(e) = h {
                    error!("Browser handler error: {}", e);
                }
            }
        });
        
        Ok(browser)
    }
}
