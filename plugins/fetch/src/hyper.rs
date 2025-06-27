use std::error::Error as StdError;
use std::fmt;

use async_trait::async_trait;
use base64::Engine;
use hyper::{Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;

use crate::chromiumoxide::{ContentFetcher, FetchResult};

#[derive(Debug)]
pub enum FetchError {
    Hyper(hyper::Error),
    Http(hyper::http::Error),
    InvalidUri(hyper::http::uri::InvalidUri),
    Other(String),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::Hyper(e) => write!(f, "Hyper error: {}", e),
            FetchError::Http(e) => write!(f, "HTTP error: {}", e),
            FetchError::InvalidUri(e) => write!(f, "Invalid URI: {}", e),
            FetchError::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl StdError for FetchError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            FetchError::Hyper(e) => Some(e),
            FetchError::Http(e) => Some(e),
            FetchError::InvalidUri(e) => Some(e),
            FetchError::Other(_) => None,
        }
    }
}

impl From<hyper::Error> for FetchError {
    fn from(e: hyper::Error) -> Self {
        FetchError::Hyper(e)
    }
}

impl From<hyper::http::Error> for FetchError {
    fn from(e: hyper::http::Error) -> Self {
        FetchError::Http(e)
    }
}

impl From<hyper::http::uri::InvalidUri> for FetchError {
    fn from(e: hyper::http::uri::InvalidUri) -> Self {
        FetchError::InvalidUri(e)
    }
}

pub struct HyperFetcher;

impl HyperFetcher {
    pub async fn fetch(url: &str) -> Result<String, FetchError> {
        // Parse the URL
        let uri: Uri = url.parse()?;

        // Create an HTTPS connector
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, Body>(https);

        // Build the request
        let request = Request::builder()
            .uri(uri)
            .header("User-Agent", "fetch-hyper/1.0")
            .method("GET")
            .body(Body::empty())?;

        // Send the request
        let response = client.request(request).await?;

        // Check if the request was successful
        if !response.status().is_success() {
            return Err(FetchError::Other(format!(
                "Request failed with status: {}",
                response.status()
            )));
        }

        // Get the response body
        let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
        let body = String::from_utf8(body_bytes.to_vec())
            .map_err(|e| FetchError::Other(format!("Failed to decode response body: {}", e)))?;

        Ok(body)
    }

    pub fn clean_html(html: &str) -> String {
        // Use a simple approach to remove script and style tags
        // A more robust approach would use an HTML parser like html5ever
        let mut result = String::new();
        let mut in_script = false;
        let mut in_style = false;

        for line in html.lines() {
            let lower = line.to_lowercase();
            
            if lower.contains("<script") {
                in_script = true;
            }
            
            if lower.contains("<style") {
                in_style = true;
            }
            
            if !in_script && !in_style {
                result.push_str(line);
                result.push('\n');
            }
            
            if lower.contains("</script>") {
                in_script = false;
            }
            
            if lower.contains("</style>") {
                in_style = false;
            }
        }

        result
    }
}

#[async_trait]
impl ContentFetcher for HyperFetcher {
    async fn fetch_content(&self, url: &str) -> Result<FetchResult, Box<dyn StdError + Send + Sync>> {
        // Fetch HTML content using hyper
        let content = Self::fetch(url).await
            .map_err(|e| Box::new(e) as Box<dyn StdError + Send + Sync>)?;
            
        // Clean the HTML content
        let cleaned_content = Self::clean_html(&content);
        
        // Generate a placeholder screenshot since hyper doesn't support screenshots
        let screenshot_base64 = base64::engine::general_purpose::STANDARD.encode(b"placeholder-screenshot-data");
        
        Ok(FetchResult {
            content: cleaned_content,
            screenshot_base64,
            content_type: "text/html".to_string(),
        })
    }
}
