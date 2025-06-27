use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

// CmsNode model for SurrealDB CMS nodes
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmsNode {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

// CmsNodeContent for reading node content
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmsNodeContent {
    pub uri: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub text: Option<String>,
    pub blob: Option<String>,
}

// ReadCmsNodeResult for returning node content
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReadCmsNodeResult {
    pub content: CmsNodeContent,
}

/// Media resource model for media table/entities (Graphlit-style)
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaResource {
    pub uri: Url,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    // Add fields as needed for media (e.g., duration, resolution, etc.)
}
