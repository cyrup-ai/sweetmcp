// Example showing the SEXY fluent builder with semantic description builder

use sweetmcp_hash::fluent::*;
use serde_json::Value;
use extism_pdk::Error;

/// Example: Using the semantic description builder
fn build_hash_description() -> String {
    DescriptionBuilder::new()
        .does("Generate cryptographic hashes and encoded formats from input data")
        .when("Create SHA hashes for security verification (sha256, sha512, sha384, sha224, sha1)")
        .when("Generate MD5 checksums for file integrity")
        .when("Encode data in base64 format for transmission")
        .when("Encode data in base32 format for URLs or identifiers")
        .when("Verify data integrity before storage or transmission")
        .perfect_for("data integrity checks, password verification, API authentication, and encoding binary data for text protocols")
        .build()
}

/// Example: Multi-operation tool with semantic descriptions
struct TimeToolDescription;

impl TimeToolDescription {
    fn build() -> String {
        DescriptionBuilder::new()
            .does("Perform time operations and calculations")
            .operation("get_time_utc", "Returns current UTC time (no parameters)")
            .operation("parse_time", "Parse RFC2822 time strings to timestamps")
            .operation("time_offset", "Add/subtract time offsets from timestamps")
            .always_for("compute time operations, especially when working with time zone conversions, date calculations, or scheduling")
            .not_for("historical dates before 1970 (Unix epoch limitation)")
            .perfect_for("time zone conversions, date calculations, scheduling operations, and time-based comparisons")
            .build()
    }
}

/// Example: Browser tool with prerequisites
struct BrowserToolDescription;

impl BrowserToolDescription {
    fn build() -> String {
        DescriptionBuilder::new()
            .does("Automate browser interactions and web scraping")
            .when("Navigate to websites and interact with web elements")
            .when("Extract data from dynamic JavaScript-rendered pages")
            .when("Take screenshots for visual verification")
            .when("Fill out forms or click buttons programmatically")
            .when("Wait for dynamic content to load before extraction")
            .requires("Chrome or Chromium browser installed on the system")
            .not_for("simple static HTML scraping (use fetch tool instead)")
            .perfect_for("web automation, testing, dynamic content extraction, and visual regression testing")
            .build()
    }
}

/// The complete plugin definition using all fluent builders
fn main() {
    // SEXY: Clean, semantic, type-safe plugin definition
    let plugin = McpPlugin::named("crypto-suite")
        .described("Comprehensive cryptographic operations and encoding suite")
        .provides::<HashTool>()           // const-generic registration
        .provides::<EncryptTool>()        // add more tools fluently
        .provides::<SignTool>()           
        .expose();                        // ready for MCP clients

    println!("Plugin ready!");
    
    // Show the generated descriptions
    println!("\nHash tool description:");
    println!("{}", build_hash_description());
    
    println!("\nTime tool description:");
    println!("{}", TimeToolDescription::build());
    
    println!("\nBrowser tool description:");
    println!("{}", BrowserToolDescription::build());
}

// Example tool implementations
struct HashTool;
struct EncryptTool;
struct SignTool;

impl McpTool for HashTool {
    const NAME: &'static str = "hash";
    const DESCRIPTION: &'static str = "Generate cryptographic hashes and encoded formats from input data. Use this tool when you need to:\n- Create SHA hashes for security verification (sha256, sha512, sha384, sha224, sha1)\n- Generate MD5 checksums for file integrity\n- Encode data in base64 format for transmission\n- Encode data in base32 format for URLs or identifiers\n- Verify data integrity before storage or transmission. Perfect for data integrity checks, password verification, API authentication, and encoding binary data for text protocols.";
    
    fn schema() -> Value {
        SchemaBuilder::new()
            .requires_string("data", "data to convert to hash or encoded format")
            .requires_enum("algorithm", "algorithm to use", 
                          &["sha256", "sha512", "md5", "base64"])
            .build()
    }
    
    fn execute(args: Value) -> Result<CallToolResult, Error> {
        // Business logic here
        Ok(ContentBuilder::text("hash_result"))
    }
}

impl McpTool for EncryptTool {
    const NAME: &'static str = "encrypt";
    const DESCRIPTION: &'static str = "Encrypt data using AES-256-GCM. Use this tool when you need to:\n- Protect sensitive data before storage\n- Secure data for transmission\n- Implement client-side encryption. Perfect for data protection and secure communication.";
    
    fn schema() -> Value {
        SchemaBuilder::new()
            .requires_string("data", "data to encrypt")
            .requires_string("key", "encryption key (base64)")
            .build()
    }
    
    fn execute(args: Value) -> Result<CallToolResult, Error> {
        Ok(ContentBuilder::text("encrypted_data"))
    }
}

impl McpTool for SignTool {
    const NAME: &'static str = "sign";
    const DESCRIPTION: &'static str = "Create digital signatures. Use this tool when you need to:\n- Sign documents or data\n- Verify authenticity\n- Implement non-repudiation. Perfect for digital signatures and authentication.";
    
    fn schema() -> Value {
        SchemaBuilder::new()
            .requires_string("data", "data to sign")
            .requires_string("private_key", "private key for signing")
            .build()
    }
    
    fn execute(args: Value) -> Result<CallToolResult, Error> {
        Ok(ContentBuilder::text("signature"))
    }
}