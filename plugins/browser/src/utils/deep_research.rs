use std::sync::Arc;
use std::time::Duration;

use kalosm_common::traits::Llm;
use kalosm_common::message::ChatMessage;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::browser::{BrowserContext, BrowserError, BrowserResult};
use crate::utils::errors::UtilsError;

/// Research result containing extracted information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub url: String,
    pub title: String,
    pub content: String,
    pub summary: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Research options for izing research behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchOptions {
    pub max_pages: usize,
    pub max_depth: usize,
    pub search_engine: String,
    pub include_links: bool,
    pub extract_tables: bool,
    pub extract_images: bool,
    pub timeout_seconds: u64,
}

impl Default for ResearchOptions {
    fn default() -> Self {
        Self {
            max_pages: 5,
            max_depth: 2,
            search_engine: "google".to_string(),
            include_links: true,
            extract_tables: true,
            extract_images: false,
            timeout_seconds: 60,
        }
    }
}

/// Deep research service for web research
pub struct DeepResearch {
    browser_context: Arc<BrowserContext>,
    llm: Arc<dyn Llm>,
    visited_urls: Arc<Mutex<Vec<String>>>,
}

impl DeepResearch {
    /// Create a new deep research service
    pub fn new(browser_context: Arc<BrowserContext>, llm: impl Llm + 'static) -> Self {
        Self {
            browser_context,
            llm: Arc::new(llm),
            visited_urls: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Perform web research on a query
    pub async fn research(
        &self,
        query: &str,
        options: Option<ResearchOptions>,
    ) -> Result<Vec<ResearchResult>, UtilsError> {
        let options = options.unwrap_or_default();
        
        // Initialize results
        let mut results = Vec::new();
        
        // Reset visited URLs
        let mut visited = self.visited_urls.lock().await;
        visited.clear();
        drop(visited);
        
        // Search for query
        let search_results = self.search_query(query, &options).await?;
        
        // Process each search result
        for url in search_results.iter().take(options.max_pages) {
            match self.process_url(url, &options).await {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    warn!("Error processing URL {}: {}", url, e);
                }
            }
            
            // Add to visited URLs
            let mut visited = self.visited_urls.lock().await;
            visited.push(url.clone());
            drop(visited);
        }
        
        Ok(results)
    }
    
    /// Search for a query and return URLs
    async fn search_query(
        &self,
        query: &str,
        options: &ResearchOptions,
    ) -> Result<Vec<String>, UtilsError> {
        let search_engine = &options.search_engine;
        let encoded_query = urlencoding::encode(query);
        
        let search_url = match search_engine.to_lowercase().as_str() {
            "google" => format!("https://www.google.com/search?q={}", encoded_query),
            "bing" => format!("https://www.bing.com/search?q={}", encoded_query),
            "duckduckgo" => format!("https://duckduckgo.com/?q={}", encoded_query),
            _ => format!("https://www.google.com/search?q={}", encoded_query),
        };
        
        // Navigate to search engine
        let page = self.browser_context.get_current_page().await
            .map_err(|e| UtilsError::BrowserError(e.to_string()))?;
            
        page.navigate(&search_url).await
            .map_err(|e| UtilsError::BrowserError(e.to_string()))?;
            
        // Wait for results to load
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Extract search result URLs with JavaScript
        let extract_script = match search_engine.to_lowercase().as_str() {
            "google" => r#"
                Array.from(document.querySelectorAll('a[href]'))
                    .map(a => a.href)
                    .filter(url => url.startsWith('http') && 
                                  !url.includes('google.com') && 
                                  !url.includes('accounts.') &&
                                  !url.includes('webcache.') &&
                                  !url.includes('/search?'))
            "#,
            "bing" => r#"
                Array.from(document.querySelectorAll('a[href]'))
                    .map(a => a.href)
                    .filter(url => url.startsWith('http') && 
                                  !url.includes('bing.com') && 
                                  !url.includes('microsoft.com') &&
                                  !url.includes('msn.com'))
            "#,
            "duckduckgo" => r#"
                Array.from(document.querySelectorAll('a.result__url'))
                    .map(a => a.href)
                    .filter(url => url.startsWith('http') && 
                                  !url.includes('duckduckgo.com'))
            "#,
            _ => r#"
                Array.from(document.querySelectorAll('a[href]'))
                    .map(a => a.href)
                    .filter(url => url.startsWith('http'))
            "#,
        };
        
        let result = page.evaluate(extract_script).await
            .map_err(|e| UtilsError::BrowserError(e.to_string()))?;
            
        // Parse JSON result to get URLs
        let urls: Vec<String> = serde_json::from_str(&result)
            .map_err(|e| UtilsError::JsonParseError(e.to_string()))?;
            
        // Deduplicate URLs
        let mut unique_urls = Vec::new();
        for url in urls {
            if !unique_urls.contains(&url) {
                unique_urls.push(url);
            }
        }
        
        Ok(unique_urls)
    }
    
    /// Process a URL and extract content
    async fn process_url(
        &self,
        url: &str,
        options: &ResearchOptions,
    ) -> Result<ResearchResult, UtilsError> {
        // Check if already visited
        let visited = self.visited_urls.lock().await;
        if visited.contains(&url.to_string()) {
            return Err(UtilsError::UnexpectedError("URL already visited".into()));
        }
        drop(visited);
        
        // Navigate to URL
        let page = self.browser_context.get_current_page().await
            .map_err(|e| UtilsError::BrowserError(e.to_string()))?;
            
        page.navigate(url).await
            .map_err(|e| UtilsError::BrowserError(e.to_string()))?;
            
        // Wait for page to load
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Get page title
        let title = page.title().await
            .unwrap_or_else(|_| "Untitled".into());
            
        // Extract content
        let content = self.extract_content(&page, options).await?;
        
        // Generate summary
        let summary = self.summarize_content(&title, &content).await?;
        
        let result = ResearchResult {
            url: url.to_string(),
            title,
            content,
            summary,
            timestamp: chrono::Utc::now(),
        };
        
        Ok(result)
    }
    
    /// Extract content from a page
    async fn extract_content(
        &self,
        page: &BrowserResult<crate::browser::BrowserPage>,
        options: &ResearchOptions,
    ) -> Result<String, UtilsError> {
        let page = match page {
            Ok(p) => p,
            Err(e) => return Err(UtilsError::BrowserError(e.to_string())),
        };
        
        // Extract content based on options
        let extract_script = format!(
            r#"
            function extractContent() {{
                // Remove script tags, style tags, and non-visible elements
                const clone = document.cloneNode(true);
                const scripts = clone.getElementsByTagName('script');
                const styles = clone.getElementsByTagName('style');
                
                while (scripts.length > 0) {{
                    scripts[0].parentNode.removeChild(scripts[0]);
                }}
                
                while (styles.length > 0) {{
                    styles[0].parentNode.removeChild(styles[0]);
                }}
                
                // Try to find main content
                const mainContent = 
                    document.querySelector('main') || 
                    document.querySelector('article') || 
                    document.querySelector('.content') || 
                    document.querySelector('#content') || 
                    document.body;
                
                let text = mainContent.innerText;
                let html = mainContent.innerHTML;
                
                // Extract tables if requested
                const tables = {extract_tables} ? Array.from(document.querySelectorAll('table')).map(table => {{
                    return Array.from(table.rows).map(row => {{
                        return Array.from(row.cells).map(cell => cell.innerText).join(' | ');
                    }}).join('\n');
                }}).join('\n\n') : '';
                
                // Extract image alt text if requested
                const images = {extract_images} ? Array.from(document.querySelectorAll('img')).map(img => {{
                    return img.alt || img.title || '';
                }}).filter(alt => alt.length > 0).join('\n') : '';
                
                // Extract links if requested
                const links = {include_links} ? Array.from(document.querySelectorAll('a[href]')).map(a => {{
                    return `${a.innerText} - ${a.href}`;
                }}).join('\n') : '';
                
                return {{ text, html, tables, images, links }};
            }}
            extractContent();
            "#,
            extract_tables = options.extract_tables.to_string(),
            extract_images = options.extract_images.to_string(),
            include_links = options.include_links.to_string(),
        );
        
        let result = page.evaluate(&extract_script).await
            .map_err(|e| UtilsError::BrowserError(e.to_string()))?;
            
        let content_obj: serde_json::Value = serde_json::from_str(&result)
            .map_err(|e| UtilsError::JsonParseError(e.to_string()))?;
            
        // Combine all extracted content
        let mut content = String::new();
        
        if let Some(text) = content_obj["text"].as_str() {
            content.push_str(text);
        }
        
        if options.extract_tables {
            if let Some(tables) = content_obj["tables"].as_str() {
                if !tables.is_empty() {
                    content.push_str("\n\nTABLES:\n");
                    content.push_str(tables);
                }
            }
        }
        
        if options.extract_images {
            if let Some(images) = content_obj["images"].as_str() {
                if !images.is_empty() {
                    content.push_str("\n\nIMAGE DESCRIPTIONS:\n");
                    content.push_str(images);
                }
            }
        }
        
        if options.include_links {
            if let Some(links) = content_obj["links"].as_str() {
                if !links.is_empty() {
                    content.push_str("\n\nLINKS:\n");
                    content.push_str(links);
                }
            }
        }
        
        Ok(content)
    }
    
    /// Summarize content using LLM
    async fn summarize_content(
        &self,
        title: &str,
        content: &str,
    ) -> Result<String, UtilsError> {
        // Truncate content if too long
        let max_content_length = 8000;
        let truncated_content = if content.len() > max_content_length {
            format!("{}... [content truncated]", &content[0..max_content_length])
        } else {
            content.to_string()
        };
        
        // Create summarization prompt
        let mut messages = Vec::new();
        messages.push(ChatMessage::system(
            "You are an AI research assistant that summarizes web content accurately and concisely. \
            Extract key information, findings, data points, and conclusions from the content. \
            Your summary should be comprehensive but focused on the most important aspects. \
            Organize information logically and provide accurate section headers where appropriate."
        ));
        
        messages.push(ChatMessage::user(
            &format!(
                "Please summarize the following webpage content. Title: '{}'\n\nContent:\n{}",
                title, truncated_content
            )
        ));
        
        // Generate summary
        let summary = self.llm.chat(messages).await
            .map_err(|e| UtilsError::LlmError(e.to_string()))?;
            
        Ok(summary)
    }
}
