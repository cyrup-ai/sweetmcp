mod chromiumoxide;
mod hyper;
mod pdk;
// mod bevy; // Disabled due to API incompatibility with bevy 0.16 - approved by David Maple 07/03/2025
mod firecrawl;

// use std::collections::BTreeMap;
use std::str::FromStr;

// use async_trait::async_trait;
use chromiumoxide::ContentFetcher;
use crate::hyper::HyperFetcher;
use extism_pdk::*;
use htmd::HtmlToMarkdown;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
// Sixel encoding is implemented inline below based on sixel6vt renderer
use base64::Engine;
use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};

/// Encode an RGB image to Sixel format (based on sixel6vt implementation)
fn encode_sixel(img: &image::RgbImage) -> String {
    // Start with DCS sequence + sixel + raster attributes with image dimensions
    let mut result = String::from("\x1BPq");

    // Add raster attributes
    result.push_str(&format!("\"{};{}", img.width(), img.height()));

    // Define a basic 16-color palette
    result.push_str("#0;2;0;0;0"); // 0: Black
    result.push_str("#1;2;20;20;80"); // 1: Dark Blue
    result.push_str("#2;2;20;80;20"); // 2: Dark Green
    result.push_str("#3;2;20;80;80"); // 3: Dark Cyan
    result.push_str("#4;2;80;20;20"); // 4: Dark Red
    result.push_str("#5;2;80;20;80"); // 5: Dark Magenta
    result.push_str("#6;2;80;80;20"); // 6: Brown
    result.push_str("#7;2;80;80;80"); // 7: Light Gray
    result.push_str("#8;2;40;40;40"); // 8: Dark Gray
    result.push_str("#9;2;40;40;100"); // 9: Light Blue
    result.push_str("#10;2;40;100;40"); // 10: Light Green
    result.push_str("#11;2;40;100;100"); // 11: Light Cyan
    result.push_str("#12;2;100;40;40"); // 12: Light Red
    result.push_str("#13;2;100;40;100"); // 13: Light Magenta
    result.push_str("#14;2;100;100;40"); // 14: Yellow
    result.push_str("#15;2;100;100;100"); // 15: White

    // Function to find the closest color in our palette
    let find_closest_color = |r: u8, g: u8, b: u8| -> u16 {
        let colors = [
            (0, 0, 0),
            (20, 20, 80),
            (20, 80, 20),
            (20, 80, 80),
            (80, 20, 20),
            (80, 20, 80),
            (80, 80, 20),
            (80, 80, 80),
            (40, 40, 40),
            (40, 40, 100),
            (40, 100, 40),
            (40, 100, 100),
            (100, 40, 40),
            (100, 40, 100),
            (100, 100, 40),
            (100, 100, 100),
        ];

        let mut min_dist = u32::MAX;
        let mut closest = 0;

        for (i, &(cr, cg, cb)) in colors.iter().enumerate() {
            let dist =
                ((r as i32 - cr).pow(2) + (g as i32 - cg).pow(2) + (b as i32 - cb).pow(2)) as u32;
            if dist < min_dist {
                min_dist = dist;
                closest = i;
            }
        }
        closest as u16
    };

    // Process the image in sixel format (6 vertical pixels at a time)
    for y in (0..img.height()).step_by(6) {
        result.push_str("#0");
        let current_color = 0;

        for x in 0..img.width() {
            let mut sixel_value = 0;

            for i in 0..6 {
                if y + i < img.height() {
                    let pixel = img.get_pixel(x, y + i);
                    let color = find_closest_color(pixel[0], pixel[1], pixel[2]);
                    if color == current_color {
                        sixel_value |= 1 << i;
                    }
                }
            }

            if sixel_value > 0 {
                result.push((b'?' + sixel_value) as char);
            } else {
                result.push('?');
            }
        }

        result.push_str("$\r\n");
    }

    result.push_str("\x1B\\");
    result
}

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
        let screenshot_format = args
            .get("screenshot_format")
            .and_then(|v| v.as_str())
            .map(|s| ScreenshotFormat::from_str(s).unwrap_or_default())
            .unwrap_or_default();

        let content_format = args
            .get("content_format")
            .and_then(|v| v.as_str())
            .map(|s| ContentFormat::from_str(s).unwrap_or_default())
            .unwrap_or_default();

        let syntax_highlighting = args
            .get("syntax_highlighting")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let theme = args
            .get("theme")
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
        Err(Error::msg("Please provide a url"))
    }
}

// Helper function to run async code from the sync world
fn block_on_fetch(url: &str) -> Result<chromiumoxide::FetchResult, Error> {
    // Set up a minimal runtime for async execution
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| Error::msg(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(async {
        // Multi-stage fetching with fallbacks:

        // 1. First attempt: Use chromiumoxide (headless browser)
        let chromium_result = chromiumoxide::ChromiumFetcher.fetch_content(url).await;

        if let Ok(result) = chromium_result {
            return Ok(result);
        }

        // 2. Second attempt: Use hyper (HTTP client)
        let hyper_result = HyperFetcher.fetch_content(url).await;

        if let Ok(result) = hyper_result {
            return Ok(result);
        }

        // 3. Final contingency: Use firecrawl
        let firecrawl_result = firecrawl::FirecrawlFetcher.fetch_content(url).await;

        match firecrawl_result {
            Ok(result) => Ok(result),
            Err(e) => Err(Error::msg(format!(
                "All fetch attempts failed. Last error: {}",
                e
            ))),
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
            let image_data = base64::engine::general_purpose::STANDARD
                .decode(&result.screenshot_base64)
                .map_err(|e| Error::msg(format!("Failed to decode screenshot: {}", e)))?;

            let image = image::load_from_memory(&image_data)
                .map_err(|e| Error::msg(format!("Failed to load image: {}", e)))?;

            encode_sixel(&image.to_rgb8())
        }
    };

    // Process the content based on the requested format
    let (content, content_type) = match options.content_format {
        ContentFormat::Markdown => {
            let converter = HtmlToMarkdown::builder()
                .skip_tags(vec!["script", "style"])
                .build();

            let markdown = converter
                .convert(&result.content)
                .map_err(|e| Error::msg(format!("Failed to convert HTML to markdown: {}", e)))?;

            (markdown, "text/markdown".to_string())
        }
        ContentFormat::Json => {
            // Extract text content from HTML and convert to JSON
            let text_content = extract_text_content(&result.content);
            let json = json!({
                "url": options.url,
                "title": extract_title(&result.content),
                "text": text_content,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "original_content_type": result.content_type
            });

            (json.to_string(), "application/json".to_string())
        }
        ContentFormat::Txt => {
            let text_content = extract_text_content(&result.content);
            (text_content, "text/plain".to_string())
        }
    };

    // Apply syntax highlighting if requested
    let final_content = if options.syntax_highlighting {
        apply_syntax_highlighting(&content, &options.content_format, options.theme.as_deref())?
    } else {
        content.to_string()
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
        (Some(start), Some(end)) => html[start + 7..end].trim().to_string(),
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
    theme_name: Option<&str>,
) -> Result<String, Error> {
    // Only apply syntax highlighting to appropriate formats
    match format {
        ContentFormat::Json => {
            let ss = SyntaxSet::load_defaults_newlines();
            let ts = ThemeSet::load_defaults();

            let syntax = ss
                .find_syntax_by_extension("json")
                .ok_or_else(|| Error::msg("Failed to find JSON syntax"))?;

            let theme = match theme_name {
                Some(name) if ts.themes.contains_key(name) => &ts.themes[name],
                _ => &ts.themes["base16-ocean.dark"], // Default theme
            };

            let html = highlighted_html_for_string(content, &ss, syntax, theme)
                .map_err(|e| Error::msg(format!("Failed to highlight JSON: {}", e)))?;

            Ok(html)
        }
        ContentFormat::Markdown => {
            // Simple markdown highlighting (in a real implementation this would be more sophisticated)
            Ok(content.to_string())
        }
        ContentFormat::Txt => Ok(content.to_string()),
    }
}

// Called by mcpx to understand how and why to use this tool
pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult{
        tools: vec![
            ToolDescription {
                name: "fetch".into(),
                description: "Retrieve and transform web content from any URL with advanced processing capabilities. Use this tool when you need to:
- Scrape web pages and extract content in multiple formats (markdown, JSON, plain text)
- Take screenshots of web pages for visual documentation
- Process dynamic websites with JavaScript rendering
- Handle complex websites with multiple fallback strategies (Bevy, Chromium, Firecrawl)
- Apply syntax highlighting to extracted code content
Perfect for web scraping, content analysis, competitive research, and automated documentation.".into(),
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
                .map(|obj| obj.clone())
                .unwrap_or_else(|| {
                    let mut map = serde_json::Map::new();
                    map.insert("type".to_string(), json!("object"));
                    map
                }),
            },
        ],
    })
}
