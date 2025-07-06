// Hash Plugin - SEXY fluent builder with proper semantics and state progression
// Your style: const-generic registration, type-safe chaining, semantic method names

mod fluent;
mod plugin;

use fluent::*;
use plugin::types::*;
use base64::Engine;
use extism_pdk::*;
use serde_json::Value;
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};

/// Hash computation logic
fn compute_hash(data: &str, algorithm: &str) -> Result<String, String> {
    match algorithm {
        "sha256" => {
            let mut hasher = Sha256::new();
            hasher.update(data.as_bytes());
            Ok(format!("{:x}", hasher.finalize()))
        }
        "sha512" => {
            let mut hasher = Sha512::new();
            hasher.update(data.as_bytes());
            Ok(format!("{:x}", hasher.finalize()))
        }
        "sha384" => {
            let mut hasher = Sha384::new();
            hasher.update(data.as_bytes());
            Ok(format!("{:x}", hasher.finalize()))
        }
        "sha224" => {
            let mut hasher = Sha224::new();
            hasher.update(data.as_bytes());
            Ok(format!("{:x}", hasher.finalize()))
        }
        "sha1" => {
            let mut hasher = Sha1::new();
            hasher.update(data.as_bytes());
            Ok(format!("{:x}", hasher.finalize()))
        }
        "md5" => Ok(format!("{:x}", md5::compute(data))),
        "base32" => Ok(base32::encode(base32::Alphabet::Rfc4648 { padding: true }, data.as_bytes())),
        "base64" => Ok(base64::engine::general_purpose::STANDARD.encode(data)),
        _ => Err(format!("Unsupported algorithm: {}", algorithm)),
    }
}

/// Hash tool with const-generic registration (your style!)
struct HashTool;

impl McpTool for HashTool {
    const NAME: &'static str = "hash";
    const DESCRIPTION: &'static str = {
        // Behind the scenes it's just a string, but semantically rich builder!
        const DESC: &str = "Generate cryptographic hashes and encoded formats from input data. Use this tool when you need to:\n- Create SHA hashes for security verification (sha256, sha512, sha384, sha224, sha1)\n- Generate MD5 checksums for file integrity\n- Encode data in base64 format for transmission\n- Encode data in base32 format for URLs or identifiers\n- Verify data integrity before storage or transmission. Perfect for data integrity checks, password verification, API authentication, and encoding binary data for text protocols.";
        DESC
    };

    fn schema() -> Value {
        SchemaBuilder::new()
            .requires_string("data", "data to convert to hash or encoded format")
            .requires_enum("algorithm", "algorithm to use for hashing or encoding", 
                          &["sha256", "sha512", "sha384", "sha224", "sha1", "md5", "base32", "base64"])
            .build()
    }

    fn execute(args: Value) -> Result<CallToolResult, Error> {
        let data = args.get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::msg("data parameter required"))?;
            
        let algorithm = args.get("algorithm")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::msg("algorithm parameter required"))?;

        match compute_hash(data, algorithm) {
            Ok(result) => Ok(ContentBuilder::text(result)),
            Err(e) => Ok(ContentBuilder::error(e)),
        }
    }
}

/// SEXY: Define the plugin with semantic fluent builder
fn plugin() -> McpPlugin<Ready> {
    McpPlugin::named("hash")
        .described("Cryptographic hashing and encoding operations for data integrity and security")
        .provides::<HashTool>()           // const-generic tool registration! 🔥
        .expose()                         // semantic: expose to MCP clients
}

// MCP Protocol Implementation - semantic method dispatch
pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    plugin().call(input)
}

pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    plugin().describe()
}