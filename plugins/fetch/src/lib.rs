mod pdk;
mod hyper;
mod chromiumoxide;
mod bevy;
mod firecrawl;

use std::collections::BTreeMap;
use std::str::FromStr;

use async_trait::async_trait;
use chromiumoxide::ContentFetcher;
use extism_pdk::*;
use htmd::HtmlToMarkdown;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sixel::sixel_from_image;
use syntect::{highlighting::{ThemeSet, Theme}, html::{highlighted_html_for_string, IncludeBackground}, parsing::{SyntaxSet, SyntaxReference}};
use base64::Engine;

#[derive(Debug, Deserialize)]
enum ScreenshotFormat {
    Base64,
    Sixel,
}

impl Default for ScreenshotFormat {
    fn default() -> Self {
        ScreenshotFormat::Base64
    }
}

impl FromStr for ScreenshotFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "base64" => Ok(ScreenshotFormat::Base64),
            "sixel" => Ok(ScreenshotFormat::Sixel),
            _ => Err(format!("Invalid screenshot format: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize)]
enum ContentFormat {
    Markdown,
    Json,
    Txt,
}

impl Default for ContentFormat {
    fn default() -> Self {
        ContentFormat::Markdown
    }
}

impl FromStr for ContentFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" => Ok(ContentFormat::Markdown),
            "json" => Ok(ContentFormat::Json),
            "txt" => Ok(ContentFormat::Txt),
            _ => Err(format!("Invalid content format: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize)]
struct FetchOptions {
    url: String,
    #[serde(default)]
    screenshot_format: ScreenshotFormat,
    #[serde(default)]
    content_format: ContentFormat,
    #[serde(default)]
    syntax_highlighting: bool,
    #[serde(default)]
    theme: Option<String>,
}

#[derive(Debug, Serialize)]
struct FetchResponse {
    screenshot: String,
    content: String,
    content_type: String,
}

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "fetch" => fetch(input),
        _ => Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(format!("Unknown tool: {}", input.params.name)),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        }),
    }
}

fn fetch(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    
    // Parse and validate arguments
    let options = parse_options(args)?;
    
    // Run the async fetching process
    let fetch_result = block_on_fetch(options.url.as_str())?;
    
    // Process results based on user preferences
    let response = process_fetch_result(fetch_result, options)?;
    
    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(response.content),
            mime_type: Some(response.content_type),
            r#type: ContentType::Text,
            data: Some(response.screenshot),
        }],
    })
}

// Parse and validate the input options
fn parse_options(args: serde_json::Map<String, Value>) -> Result<FetchOptions, Error> {
    if let Some(Value::String(url)) = args.get("url") {
        let screenshot_format = args.get("screenshot_format")
            .and_then(|v| v.as_str())
            .map(|s| ScreenshotFormat::from_str(s).unwrap_or_default())
            .unwrap_or_default();
            
        let content_format = args.get("content_format")
            .and_then(|v| v.as_str())
            .map(|s| ContentFormat::from_str(s).unwrap_or_default())
            .unwrap_or_default();
            
        let syntax_highlighting = args.get("syntax_highlighting")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        let theme = args.get("theme")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
            
        Ok(FetchOptions {
            url: url.clone(),
            screenshot_format,
            content_format,
            syntax_highlighting,
            theme,
        })
    } else {
        Err(Error::Other("Please provide a url".into()))
    }
}

// Helper function to run async code from the sync world
fn block_on_fetch(url: &str) -> Result<chromiumoxide::FetchResult, Error> {
    // Set up a minimal runtime for async execution
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::Other(format!("Failed to create runtime: {}", e)))?;
        
    rt.block_on(async {
        // Multi-stage fetching with fallbacks:
        
        // 1. First attempt: Use hyper with bevy rendering
        let bevy_result = bevy::BevyRenderer.fetch_content(url).await;
        
        if let Ok(result) = bevy_result {
            return Ok(result);
        }
        
        // 2. Fallback: Use chromiumoxide (headless browser)
        let chromium_result = chromiumoxide::ChromiumFetcher.fetch_content(url).await;
        
        if let Ok(result) = chromium_result {
            return Ok(result);
        }
        
        // 3. Final contingency: Use firecrawl
        let firecrawl_result = firecrawl::FirecrawlFetcher.fetch_content(url).await;
        
        match firecrawl_result {
            Ok(result) => Ok(result),
            Err(e) => Err(Error::Other(format!("All fetch attempts failed. Last error: {}", e))),
        }
    })
}

// Process the fetch result to get the desired format
fn process_fetch_result(
    result: chromiumoxide::FetchResult,
    options: FetchOptions,
) -> Result<FetchResponse, Error> {
    // Process the screenshot based on the requested format
    let screenshot = match options.screenshot_format {
        ScreenshotFormat::Base64 => result.screenshot_base64,
        ScreenshotFormat::Sixel => {
            // Convert base64 to image, then to sixel
            let image_data = base64::engine::general_purpose::STANDARD.decode(&result.screenshot_base64)
                .map_err(|e| Error::Other(format!("Failed to decode screenshot: {}", e)))?;
                
            let image = image::load_from_memory(&image_data)
                .map_err(|e| Error::Other(format!("Failed to load image: {}", e)))?;
                
            sixel_from_image(&image, Default::default())
                .map_err(|e| Error::Other(format!("Failed to convert to sixel: {}", e)))?
        }
    };
    
    // Process the content based on the requested format
    let (content, content_type) = match options.content_format {
        ContentFormat::Markdown => {
            let converter = HtmlToMarkdown::builder()
                .skip_tags(vec!["script", "style"])
                .build();
                
            let markdown = converter.convert(&result.content)
                .map_err(|e| Error::Other(format!("Failed to convert HTML to markdown: {}", e)))?;
                
            (markdown, "text/markdown".to_string())
        },
        ContentFormat::Json => {
            // Extract text content from HTML and convert to JSON
            let text_content = extract_text_content(&result.content);
            let json = json!({
                "url": options.url,
                "title": extract_title(&result.content),
                "text": text_content,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            
            (json.to_string(), "application/json".to_string())
        },
        ContentFormat::Txt => {
            let text_content = extract_text_content(&result.content);
            (text_content, "text/plain".to_string())
        }
    };
    
    // Apply syntax highlighting if requested
    let final_content = if options.syntax_highlighting {
        apply_syntax_highlighting(&content, &options.content_format, options.theme.as_deref())?
    } else {
        content
    };
    
    Ok(FetchResponse {
        screenshot,
        content: final_content,
        content_type,
    })
}

// Extract title from HTML
fn extract_title(html: &str) -> String {
    let title_start = html.find("<title>");
    let title_end = html.find("</title>");
    
    match (title_start, title_end) {
        (Some(start), Some(end)) => {
            html[start + 7..end].trim().to_string()
        },
        _ => "Untitled".to_string(),
    }
}

// Extract text content from HTML
fn extract_text_content(html: &str) -> String {
    // Simple text extraction - in a real implementation this would be more robust
    let mut text = String::new();
    let mut in_tag = false;
    
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    
    // Clean up whitespace
    text = text.replace('\n', " ");
    while text.contains("  ") {
        text = text.replace("  ", " ");
    }
    
    text.trim().to_string()
}

// Apply syntax highlighting to content
fn apply_syntax_highlighting(
    content: &str, 
    format: &ContentFormat,
    theme_name: Option<&str>
) -> Result<String, Error> {
    // Only apply syntax highlighting to appropriate formats
    match format {
        ContentFormat::Json => {
            let ss = SyntaxSet::load_defaults_newlines();
            let ts = ThemeSet::load_defaults();
            
            let syntax = ss.find_syntax_by_extension("json")
                .ok_or_else(|| Error::Other("Failed to find JSON syntax".into()))?;
                
            let theme = match theme_name {
                Some(name) if ts.themes.contains_key(name) => &ts.themes[name],
                _ => &ts.themes["base16-ocean.dark"] // Default theme
            };
            
            let html = highlighted_html_for_string(content, &ss, syntax, theme)
                .map_err(|e| Error::Other(format!("Failed to highlight JSON: {}", e)))?;
                
            Ok(html)
        },
        ContentFormat::Markdown => {
            // Simple markdown highlighting (in a real implementation this would be more sophisticated)
            Ok(content.to_string())
        },
        ContentFormat::Txt => Ok(content.to_string()),
    }
}

// Called by mcpx to understand how and why to use this tool
pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult{
        tools: vec![
            ToolDescription {
                name: "fetch".into(),
                description: "Retrieve and transform web content from a specified URL through a reliable multi-stage process with fallbacks".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to fetch",
                        },
                        "screenshot_format": {
                            "type": "string",
                            "description": "Format for the screenshot (base64 or sixel)",
                            "enum": ["base64", "sixel"],
                            "default": "base64"
                        },
                        "content_format": {
                            "type": "string",
                            "description": "Format for the content (markdown, json, or txt)",
                            "enum": ["markdown", "json", "txt"],
                            "default": "markdown"
                        },
                        "syntax_highlighting": {
                            "type": "boolean",
                            "description": "Whether to apply syntax highlighting to the content",
                            "default": false
                        },
                        "theme": {
                            "type": "string",
                            "description": "Theme to use for syntax highlighting",
                            "default": "base16-ocean.dark"
                        }
                    },
                    "required": ["url"],
                })
                .as_object()
                .unwrap()
                .clone(),
            },
        ],
    })
}
