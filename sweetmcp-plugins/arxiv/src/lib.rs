mod plugin;

use chrono::{DateTime, Utc};
use extism_pdk::*;
use plugin::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, Role, TextAnnotation,
    ToolDescription,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Debug, Serialize, Deserialize)]
struct Paper {
    paper_id: String,
    title: String,
    authors: Vec<String>,
    abstract_text: String,
    url: String,
    pdf_url: String,
    published_date: DateTime<Utc>,
    updated_date: DateTime<Utc>,
    source: String,
    categories: Vec<String>,
    keywords: Vec<String>,
    doi: String,
}

pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "arxiv_search" => search(input),
        "arxiv_download_pdf" => download_pdf(input),
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

fn search(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let query = match args.get("query") {
        Some(v) if v.is_string() => v
            .as_str()
            .ok_or_else(|| Error::msg("query parameter must be a valid string"))?,
        _ => return Err(Error::msg("query parameter is required")),
    };

    let max_results = args
        .get("max_results")
        .and_then(|v| v.as_u64())
        .unwrap_or(10);

    let req = HttpRequest {
        url: format!(
            "http://export.arxiv.org/api/query?search_query={}&max_results={}&sortBy=submittedDate&sortOrder=descending",
            urlencoding::encode(query),
            max_results
        ),
        headers: [(
            "User-Agent".to_string(),
            "hyper-mcp/1.0 (https://github.com/tuananh/hyper-mcp)".to_string(),
        )]
        .into_iter()
        .collect(),
        method: Some("GET".to_string()),
    };

    let res = http::request::<()>(&req, None)?;

    let body = res.body();
    let xml = String::from_utf8_lossy(body.as_slice());

    let feed = match feed_rs::parser::parse(xml.as_bytes()) {
        Ok(feed) => feed,
        Err(e) => return Err(Error::msg(format!("Failed to parse arXiv feed: {}", e))),
    };

    let mut papers = Vec::new();
    for entry in feed.entries {
        let paper_id = entry.id.split("/abs/").last().unwrap_or("").to_string();

        let authors = entry
            .authors
            .iter()
            .map(|author| author.name.clone())
            .collect();

        let categories = entry
            .categories
            .iter()
            .map(|cat| cat.term.clone())
            .collect();

        papers.push(Paper {
            paper_id,
            title: entry.title.map(|t| t.content).unwrap_or_default(),
            authors,
            abstract_text: entry.content.and_then(|c| c.body).unwrap_or_default(),
            url: entry
                .links
                .iter()
                .find(|l| l.rel == Some("alternate".to_string()))
                .map(|l| l.href.clone())
                .unwrap_or_default(),
            pdf_url: entry
                .links
                .iter()
                .find(|l| l.media_type.as_deref() == Some("application/pdf"))
                .map(|l| l.href.clone())
                .unwrap_or_default(),
            published_date: entry.published.unwrap_or_default(),
            updated_date: entry.updated.unwrap_or_default(),
            source: "arxiv".to_string(),
            categories,
            keywords: Vec::new(),
            doi: String::new(),
        });
    }

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(serde_json::to_string(&papers)?),
            mime_type: Some("application/json".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

fn download_pdf(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();
    let paper_id = match args.get("paper_id") {
        Some(v) if v.is_string() => v
            .as_str()
            .ok_or_else(|| Error::msg("paper_id parameter must be a valid string"))?,
        _ => return Err(Error::msg("paper_id parameter is required")),
    };

    // Get the path parameter with default to /tmp
    let save_path = args
        .get("save_path")
        .and_then(|v| v.as_str())
        .unwrap_or("/tmp");

    // Clean up the paper ID in case it contains the full URL
    let clean_paper_id = if paper_id.contains("/") {
        paper_id.split('/').next_back().unwrap_or(paper_id)
    } else {
        paper_id
    };

    let url = format!("https://arxiv.org/pdf/{}", clean_paper_id);

    let req = HttpRequest {
        url,
        headers: [
            (
                "User-Agent".to_string(),
                "Mozilla/5.0 (compatible; hyper-mcp/1.0)".to_string(),
            ),
            ("Accept".to_string(), "application/pdf".to_string()),
        ]
        .into_iter()
        .collect(),
        method: Some("GET".to_string()),
    };

    let res = match http::request::<()>(&req, None) {
        Ok(r) => r,
        Err(e) => return Err(Error::msg(format!("HTTP request failed: {}", e))),
    };

    let pdf_data = res.body();
    if pdf_data.is_empty() {
        return Err(Error::msg("Received empty PDF data from arXiv"));
    }

    let file_path = format!("{}/{}.pdf", save_path.trim_end_matches('/'), clean_paper_id);
    match std::fs::write(&file_path, &pdf_data) {
        Ok(_) => (),
        Err(e) => {
            return Err(Error::msg(format!(
                "Failed to write PDF to {}: {}",
                file_path, e
            )));
        }
    }

    // let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(pdf_data);

    // TODO: actually return a resource
    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: Some(TextAnnotation {
                audience: vec![Role::User, Role::Assistant],
                priority: 1.0,
            }),
            text: Some(format!("PDF saved to: {}", file_path)),
            mime_type: None,
            data: None,
            r#type: ContentType::Text,
        }],
    })
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult {
        tools: vec![
            ToolDescription {
                name: "arxiv_search".into(),
                description: "Search for academic papers on arXiv preprint repository. Use this tool when you need to:
- Find recent research papers on specific topics or by authors
- Gather citations and references for academic writing
- Stay updated with latest developments in scientific fields
- Explore related work for research projects
- Build literature reviews and research surveys
Perfect for academic research, literature reviews, citation gathering, and staying current with scientific advances. Returns metadata including title, authors, abstract, and paper ID.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query",
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Maximum number of results to return (default: 10)",
                        }
                    },
                    "required": ["query"],
                })
                .as_object()
                .expect("JSON schema should be valid object")
                .clone(),
            },
            ToolDescription {
                name: "arxiv_download_pdf".into(),
                description: "Download full PDF papers from arXiv repository. Use this tool when you need to:
- Obtain complete paper content for detailed analysis
- Build local research paper collections
- Access figures, equations, and detailed methodologies
- Archive papers for offline reading
- Extract specific sections or data from papers
Perfect for in-depth research, paper archiving, and detailed academic analysis. Note: Respects arXiv rate limits and terms of service.".into(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "paper_id": {
                            "type": "string",
                            "description": "The arXiv paper ID",
                        },
                        "save_path": {
                            "type": "string",
                            "description": "Path to save the PDF file (default: /tmp)",
                        }
                    },
                    "required": ["paper_id"],
                })
                .as_object()
                .expect("JSON schema should be valid object")
                .clone(),
            },
        ],
    })
}
