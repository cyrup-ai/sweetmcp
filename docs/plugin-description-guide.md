# Plugin Description Writing Guide

Based on the MCP Plugin Prompting Guidelines in CLAUDE.md, here's how to write effective plugin descriptions that help agents understand when and how to use your tools.

## Description Format Template

```rust
description: "[Primary function]. Use this tool when you need to:
- [Use case 1] 
- [Use case 2]
- [Use case 3]
Perfect for [common scenarios and applications]."
```

## Key Principles

1. **Be Specific**: Include concrete use cases, not just generic descriptions
2. **Guide Discovery**: Help agents understand when NOT to use a tool as well as when to use it
3. **Provide Context**: Explain the value and common applications
4. **Use Action Words**: Start descriptions with verbs (Generate, Validate, Retrieve, etc.)
5. **Include Examples**: Reference common scenarios in descriptions

## Examples of Good Descriptions

### Single Tool Plugin
```rust
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
            input_schema: // ... schema details
        }],
    })
}
```

### Multi-Tool Plugin
```rust
pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult {
        tools: vec![
            ToolDescription {
                name: "get_public_ip".into(),
                description: "Get your current public IP address. Use this tool when you need to:
- Determine your external IP address for network configuration
- Troubleshoot connectivity issues
- Set up port forwarding or firewall rules
- Verify VPN connection status
Perfect for network diagnostics and configuration tasks.".into(),
                input_schema: // ... schema details
            },
            ToolDescription {
                name: "validate_ip".into(),
                description: "Validate IP address format and classify type. Use this tool when you need to:
- Check if an IP address string is properly formatted
- Determine if an address is IPv4 or IPv6
- Validate user input before network operations
- Parse IP addresses from logs or configuration files
Perfect for input validation and network programming.".into(),
                input_schema: // ... schema details
            },
        ],
    })
}
```

## Common Patterns by Tool Category

### File/Data Operations
```
"[Operation] file contents. Use this tool when you need to:
- Process [specific file types]
- Extract [specific data]
- [Specific use case]
Perfect for [file type] processing, data extraction, and [domain] workflows."
```

### Network/API Tools
```
"[Network operation]. Use this tool when you need to:
- [Connectivity task]
- [API interaction]
- [Data retrieval task]
Perfect for [protocol] operations, API integration, and network diagnostics."
```

### Processing/Transformation Tools
```
"[Transform/process] data using [method]. Use this tool when you need to:
- Convert [input format] to [output format]
- Process [data type] for [purpose]
- Generate [output type] from [input type]
Perfect for data transformation, [domain] workflows, and automated processing."
```

### Security/Cryptography Tools
```
"[Security operation] using [algorithm/method]. Use this tool when you need to:
- [Security task like hashing/encryption]
- Verify [integrity/authenticity]
- Generate [secure output]
Perfect for security operations, authentication, and data protection."
```

## Bad Examples to Avoid

### Too Generic ❌
```rust
description: "Read file contents"  // No context or use cases
```

### Missing Use Cases ❌
```rust
description: "Generate QR codes from text"  // What for? When to use?
```

### No Value Proposition ❌
```rust
description: "Search for papers on arXiv"  // Why? What scenarios?
```

## Improved Versions ✅

### File Operations
```rust
description: "Read file contents from the filesystem. Use this tool when you need to:
- Analyze source code or configuration files
- Extract data from logs or CSV files
- Review documentation or markdown files
- Load templates or data files for processing
Perfect for file inspection, data extraction, and content analysis workflows."
```

### Search Operations
```rust
description: "Search academic papers on arXiv repository. Use this tool when you need to:
- Find recent research on specific topics
- Gather citations for academic writing
- Stay updated with latest publications in your field
- Discover related work for research projects
Perfect for literature reviews, research planning, and academic reference gathering."
```

### Generation Operations
```rust
description: "Generate QR codes as PNG images. Use this tool when you need to:
- Create scannable codes for URLs or contact information
- Generate codes for event tickets or authentication
- Encode data for mobile-friendly sharing
- Create QR codes for marketing materials or signage
Perfect for mobile integration, contactless sharing, and bridging physical-digital experiences."
```

## Testing Your Descriptions

Ask yourself:
1. Would an AI agent know WHEN to use this tool?
2. Would it know what problems this tool solves?
3. Are the use cases specific enough to guide selection?
4. Is the value proposition clear?

## Special Cases

### Complex Tools with Multiple Operations
For tools like `time` that have multiple sub-operations, list them clearly:

```rust
description: "Time operations plugin. It provides the following operations:
- `get_time_utc`: Returns current UTC time (no parameters)
- `parse_time`: Parse RFC2822 time strings to timestamps
- `time_offset`: Add/subtract time offsets from timestamps

Always use this tool to compute time operations, especially when working with:
- Time zone conversions
- Date calculations
- Scheduling operations
- Time-based comparisons"
```

### Tools with Prerequisites
If your tool requires specific conditions:

```rust
description: "Execute database queries. Use this tool when you need to:
- Query structured data from SQL databases
- Perform data analysis on large datasets
- Generate reports from database tables
NOTE: Requires database connection to be configured.
Perfect for data analysis, reporting, and database administration."
```

## Final Checklist

- [ ] Starts with primary function (verb + object)
- [ ] Includes "Use this tool when you need to:"
- [ ] Lists 3-5 specific use cases
- [ ] Ends with "Perfect for..." value proposition
- [ ] Uses concrete examples, not abstract descriptions
- [ ] Mentions any limitations or prerequisites
- [ ] Distinguishes from similar tools if applicable