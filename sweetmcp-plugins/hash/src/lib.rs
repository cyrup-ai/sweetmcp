use base64::Engine;
use extism_pdk::*;
use serde_json::Value;
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use sweetmcp_plugin_builder::prelude::*;
use sweetmcp_plugin_builder::{CallToolResult, Ready};

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
        "md5" => {
            let digest = md5::compute(data.as_bytes());
            Ok(format!("{:x}", digest))
        }
        "base64" => {
            let encoded = base64::engine::general_purpose::STANDARD.encode(data.as_bytes());
            Ok(encoded)
        }
        "base32" => {
            let encoded =
                base32::encode(base32::Alphabet::Rfc4648 { padding: true }, data.as_bytes());
            Ok(encoded)
        }
        _ => Err(format!("Unsupported algorithm: {}", algorithm)),
    }
}

/// Hash tool using plugin-builder
struct HashTool;

impl McpTool for HashTool {
    const NAME: &'static str = "hash";

    fn description(builder: DescriptionBuilder) -> DescriptionBuilder {
        builder
            .does("Generate cryptographic hashes and encoded formats from input data")
            .when("you need to create SHA hashes for security verification (sha256, sha512, sha384, sha224, sha1)")
            .when("you need to generate MD5 checksums for file integrity")
            .when("you need to encode data in base64 format for transmission")
            .when("you need to encode data in base32 format for URLs or identifiers")
            .when("you need to verify data integrity before storage or transmission")
            .perfect_for("data integrity checks, password verification, API authentication, and encoding binary data for text protocols")
    }

    fn schema(builder: SchemaBuilder) -> Value {
        builder
            .required_string("data", "data to convert to hash or encoded format")
            .required_enum(
                "algorithm",
                "algorithm to use for hashing or encoding",
                &[
                    "sha256", "sha512", "sha384", "sha224", "sha1", "md5", "base32", "base64",
                ],
            )
            .build()
    }

    fn execute(args: Value) -> Result<CallToolResult, Error> {
        let data = args
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::msg("data parameter required"))?;

        let algorithm = args
            .get("algorithm")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::msg("algorithm parameter required"))?;

        match compute_hash(data, algorithm) {
            Ok(result) => Ok(ContentBuilder::text(result)),
            Err(e) => Err(Error::msg(e)),
        }
    }
}

/// Create the plugin instance
#[allow(dead_code)]
fn plugin() -> McpPlugin<Ready> {
    mcp_plugin("hash")
        .description("Cryptographic hashing and encoding operations with support for SHA family, MD5, base64, and base32")
        .tool::<HashTool>()
        .serve()
}

// Generate standard MCP entry points
sweetmcp_plugin_builder::generate_mcp_functions!(plugin);
