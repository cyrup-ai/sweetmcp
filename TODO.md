# SweetMCP Auto-Integration System TODO

## Foundation Components

### 1. Enhanced Certificate Management
Create wildcard self-signed certificate with multiple SAN entries in $XDG_CONFIG_HOME/sweetmcp/wildcard.cyrup.pem. Primary *.cyrup.dev with SAN for *.cyrup.ai, *.cyrup.pro, *.cyrup.cloud. Certificate should be non-expiring for maximum convenience.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 2. Act as an Objective QA Rust developer and rate the work performed previously on certificate management requirements. Verify the wildcard certificate was generated correctly with all required SAN entries, stored in the correct location, and is properly formatted for cross-platform use.

### 3. Multi-Domain Host File Management  
Add host entries for sweetmcp.cyrup.dev, sweetmcp.cyrup.ai, sweetmcp.cyrup.cloud, sweetmcp.cyrup.pro all pointing to 127.0.0.1. Handle cross-platform host file locations and permissions with proper sudo elevation.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 4. Act as an Objective QA Rust developer and rate the work performed previously on host file management requirements. Verify all 4 domain entries were added correctly, point to 127.0.0.1, and work across different operating systems.

### 5. Cross-Platform Trust Store Integration
Import the wildcard certificate into OS trust stores (macOS Keychain, Linux ca-certificates, Windows certlm) using sudo privileges. Support all major operating systems with proper error handling.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 6. Act as an Objective QA Rust developer and rate the work performed previously on trust store integration requirements. Verify the certificate was imported successfully into each OS trust store and is properly trusted for HTTPS connections.

## Tool Integration Framework

### 7. Tool Integration Strategy Pattern
Create ToolIntegrationStrategy trait with methods: detect(), configure(), validate(). Design registry pattern to hold all strategies and enable easy addition of new tools.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 8. Act as an Objective QA Rust developer and rate the work performed previously on strategy pattern requirements. Verify the trait design is extensible, the registry works correctly, and new strategies can be easily added.

### 9. Tool Scanner Background Service
Create daemon background service that scans for installed tools every 15 minutes and on startup. Integrate with existing Pingora background service architecture. Should detect newly installed tools and trigger automatic configuration.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 10. Act as an Objective QA Rust developer and rate the work performed previously on tool scanner service requirements. Verify the background service runs correctly, scans at proper intervals, and integrates well with the existing daemon architecture.

## POC Tool Implementations

### 11. Claude Desktop Strategy Implementation
Implement ToolIntegrationStrategy for Claude Desktop. Detect installation across platforms, configure MCP server entry pointing to SweetMCP Pingora endpoint via HTTPS. Add SweetMCP as additional server, do not replace existing configurations.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 12. Act as an Objective QA Rust developer and rate the work performed previously on Claude Desktop integration requirements. Verify detection works across platforms, configuration is added correctly, and existing MCP servers are preserved.

### 13. Windsurf Strategy Implementation
Implement ToolIntegrationStrategy for Windsurf. Detect installation, configure MCP integration pointing to SweetMCP HTTPS endpoint. Handle Windsurf-specific configuration format and file locations.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 14. Act as an Objective QA Rust developer and rate the work performed previously on Windsurf integration requirements. Verify Windsurf detection works, configuration format is correct, and integration points to the proper SweetMCP endpoint.

## Future Strategy Stubs

### 15. VSCode Strategy Trait Stub
Create trait implementation stub for VSCode with detect(), configure(), validate() methods. Document expected configuration approach for future implementation.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 16. Act as an Objective QA Rust developer and rate the work performed previously on VSCode strategy stub requirements. Verify the stub implementation is properly structured and documented for future development.

### 17. Zed Strategy Trait Stub
Create trait implementation stub for Zed editor with detect(), configure(), validate() methods. Document expected configuration approach for future implementation.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 18. Act as an Objective QA Rust developer and rate the work performed previously on Zed strategy stub requirements. Verify the stub implementation is properly structured and documented for future development.

### 19. Cursor.AI Strategy Trait Stub
Create trait implementation stub for Cursor.AI with detect(), configure(), validate() methods. Document expected configuration approach for future implementation.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 20. Act as an Objective QA Rust developer and rate the work performed previously on Cursor.AI strategy stub requirements. Verify the stub implementation is properly structured and documented for future development.

### 21. Raycast Strategy Trait Stub
Create trait implementation stub for Raycast with detect(), configure(), validate() methods. Document expected configuration approach for future implementation.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 22. Act as an Objective QA Rust developer and rate the work performed previously on Raycast strategy stub requirements. Verify the stub implementation is properly structured and documented for future development.

### 23. Warp Terminal Strategy Trait Stub
Create trait implementation stub for Warp terminal with detect(), configure(), validate() methods. Document expected configuration approach for future implementation.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 24. Act as an Objective QA Rust developer and rate the work performed previously on Warp strategy stub requirements. Verify the stub implementation is properly structured and documented for future development.

### 25. Lapce Strategy Trait Stub
Create trait implementation stub for Lapce editor with detect(), configure(), validate() methods. Document expected configuration approach for future implementation.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 26. Act as an Objective QA Rust developer and rate the work performed previously on Lapce strategy stub requirements. Verify the stub implementation is properly structured and documented for future development.

### 27. Goose Strategy Trait Stub
Create trait implementation stub for Goose with detect(), configure(), validate() methods. Document expected configuration approach for future implementation.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 28. Act as an Objective QA Rust developer and rate the work performed previously on Goose strategy stub requirements. Verify the stub implementation is properly structured and documented for future development.

### 29. OpenAI Codex Strategy Trait Stub
Create trait implementation stub for OpenAI Codex with detect(), configure(), validate() methods. Document expected configuration approach for future implementation.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 30. Act as an Objective QA Rust developer and rate the work performed previously on OpenAI Codex strategy stub requirements. Verify the stub implementation is properly structured and documented for future development.

## Integration and Testing

### 31. Enhanced Installer Integration
Update installer.rs to call certificate generation, host file management, trust store integration, and tool scanner initialization during daemon installation.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 32. Act as an Objective QA Rust developer and rate the work performed previously on installer integration requirements. Verify all new components are properly integrated into the installation process and work correctly during daemon setup.

### 33. Cross-Platform Validation
Test certificate generation, host file management, trust store integration, and tool detection across macOS, Linux, and Windows. Ensure all components work correctly on different operating systems.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 34. Act as an Objective QA Rust developer and rate the work performed previously on cross-platform validation requirements. Verify all functionality works correctly across different operating systems and edge cases are handled properly.

### 35. End-to-End Integration Test
Create comprehensive test that installs daemon, generates certificates, configures host entries, scans for tools, and validates that Claude Desktop and Windsurf are automatically configured when detected.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 36. Act as an Objective QA Rust developer and rate the work performed previously on end-to-end integration test requirements. Verify the complete workflow functions as designed and delivers the expected user experience of zero-friction tool integration.