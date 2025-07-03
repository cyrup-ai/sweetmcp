mod plugin;

use base64::Engine;
use extism_pdk::*;
use plugin::types::*;
use serde_json::json;
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};

// Called when the tool is invoked.
pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    extism_pdk::log!(
        LogLevel::Info,
        "called with args: {:?}",
        input.params.arguments
    );
    let args = input.params.arguments.unwrap_or_default();

    let data = match args.get("data") {
        Some(v) => v.as_str().unwrap(),
        None => return Err(Error::msg("`data` is required")),
    };

    let algorithm = match args.get("algorithm") {
        Some(v) => v.as_str().unwrap(),
        None => return Err(Error::msg("`algorithm` is required")),
    };

    let result = match algorithm {
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(data.as_bytes());
            format!("{:x}", hasher.finalize())
        }
        "sha512" => {
            let mut hasher = Sha512::new();
            hasher.update(data.as_bytes());
            format!("{:x}", hasher.finalize())
        }
        "sha384" => {
            let mut hasher = Sha384::new();
            hasher.update(data.as_bytes());
            format!("{:x}", hasher.finalize())
        }
        "sha224" => {
            let mut hasher = Sha224::new();
            hasher.update(data.as_bytes());
            format!("{:x}", hasher.finalize())
        }
        "sha1" => {
            let mut hasher = Sha1::new();
            hasher.update(data.as_bytes());
            format!("{:x}", hasher.finalize())
        }
        "md5" => {
            format!("{:x}", md5::compute(data))
        }
        "base32" => base32::encode(base32::Alphabet::Rfc4648 { padding: true }, data.as_bytes()),
        "base64" | _ => base64::engine::general_purpose::STANDARD.encode(data),
    };

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(result),
            mime_type: Some("text/plain".into()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult {
        tools: vec![ToolDescription {
            name: "hash".into(),
            description: "Generate cryptographic hashes and encoded formats from input data. Use this tool when you need to:
- Create SHA hashes for security verification (sha256, sha512, sha384, sha224, sha1)
- Generate MD5 checksums for file integrity
- Encode data in base64 format for transmission
- Encode data in base32 format for URLs or identifiers
Perfect for data integrity checks, password verification, API authentication, and encoding binary data for text protocols.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "data": {
                        "type": "string",
                        "description": "data to convert to hash or encoded format"
                    },
                    "algorithm": {
                        "type": "string",
                        "description": "algorithm to use for hashing or encoding",
                        "enum": ["sha256", "sha512", "sha384", "sha224", "sha1", "md5", "base32", "base64"]
                    }
                },
                "required": ["data", "algorithm"]
            }).as_object().unwrap().clone(),
        }],
    })
}
