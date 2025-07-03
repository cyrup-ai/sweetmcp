# Plan for Useful Value & Production Quality

## Pre-planner Orientation

```markdown
<thinking>
    *What is the highest level USER OBJECTIVE?*

    Fix ALL 79 Rust compiler warnings to achieve 0 errors and 0 warnings for production-ready codebase that will be demonstrated to investors for $1M funding. The architecture must remain intact: sweetmcp-daemon (with subcommands) + sweetmcp_server (pingora binary).

    The warnings indicate incomplete implementations, not dead code. Every warning represents missing functionality that needs to be properly wired up without changing the fundamental architecture.
</thinking>
```

## Milestone Analysis

```markdown
<thinking>
  - What milestones have we completed?
    * Fixed some unused imports and variables
    * Identified correct 2-binary architecture (daemon + pingora)
    * Located all warning sources across workspace
  
  - What's the last milestone we completed?
    * Understanding the real architecture from installer.rs (684 lines)
    * Confirmed pingora integrates axum as library, autoconfig runs internally
  
  - What's the current milestone?
    * Remove incorrect binary definitions I added
    * Fix all 79 remaining warnings by implementing missing functionality
    * Achieve 0 warnings, 0 errors for production demo
  
  - What's the scope, the quintessence of "done"?
    * Clean `cargo check` output with zero warnings
    * All functionality implemented and properly wired
    * Architecture restored to correct 2-binary design
    * Production-quality code ready for investor demo
  
  - What should we be able to prove, demonstrate at the end of the current milestone?
    * `cargo check` shows 0 warnings, 0 errors
    * All services start and function correctly
    * Installer works properly with correct architecture
    * No functionality lost, everything properly integrated
</thinking>
```

## TODO.md

### 1. Remove Incorrect Binary Definitions
Remove the extra binary definitions I incorrectly added that break the 2-binary architecture.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 2. QA: Architecture Restoration Review
Act as an Objective QA Rust developer and rate the architecture restoration work. Verify that only 2 binaries exist (sweetmcp-daemon, sweetmcp_server), no extra binaries were created, and the service management system remains intact. Rate compliance with production architecture requirements.

### 3. Fix Service Name Field Usage in AutoConfigService
Fix the unused `name` field in sweetmcp-daemon/src/service/autoconfig.rs by replacing hardcoded "autoconfig" strings with the actual service name from the field.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 4. QA: Service Name Usage Fix Review
Act as an Objective QA Rust developer and rate the service name field implementation. Verify the name field is properly used throughout the service lifecycle, no hardcoded strings remain, and the fix eliminates the unused field warning without breaking functionality.

### 5. Fix Unused Methods in CommandBuilder
Implement usage of the `arg` method in sweetmcp-daemon/src/install/builder.rs:145 by finding where command arguments should be added and connecting the builder pattern properly.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 6. QA: CommandBuilder Methods Review
Act as an Objective QA Rust developer and rate the CommandBuilder method implementation. Verify the arg method is properly integrated into the command building process, eliminates the unused method warning, and maintains the builder pattern correctly.

### 7. Fix Unused get_helper_path Function
Implement usage of `get_helper_path` function in sweetmcp-daemon/src/install/macos.rs:405 by connecting it to the macOS installation process where helper paths are needed.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 8. QA: Helper Path Function Review
Act as an Objective QA Rust developer and rate the helper path function implementation. Verify the function is properly called during macOS installation, eliminates the unused function warning, and provides correct helper path resolution.

### 9. Fix Unused Graph Database Constructors
Implement usage of the four `new` functions in sweetmcp-memory/src/graph/graph_db.rs (lines 45, 70, 95, 120) by connecting them to the graph database operations and async response handling.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 10. QA: Graph Database Constructors Review
Act as an Objective QA Rust developer and rate the graph database constructor implementation. Verify all new functions are properly used in async operations, eliminate unused function warnings, and provide correct async response handling for PendingNode, NodeQuery, NodeUpdate, and NodeStream.

### 11. Fix Unused Config Field in MemoryQueryExecutor
Implement usage of the `config` field in sweetmcp-memory/src/memory/query.rs:111 by adding the missing query configuration logic that uses this field.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 12. QA: Memory Query Config Review
Act as an Objective QA Rust developer and rate the query configuration implementation. Verify the config field is properly used in query execution, eliminates the unused field warning, and provides correct query configuration functionality.

### 13. Fix Unused vector_store Field in HybridRetrieval
Implement usage of the `vector_store` field in sweetmcp-memory/src/memory/retrieval.rs:70 by adding the missing vector store operations that this field should enable.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 14. QA: HybridRetrieval Vector Store Review
Act as an Objective QA Rust developer and rate the vector store implementation in HybridRetrieval. Verify the vector_store field is properly used in retrieval operations, eliminates the unused field warning, and provides correct hybrid retrieval functionality.

### 15. Fix Unused time_decay_factor Field in TemporalRetrieval
Implement usage of the `time_decay_factor` field in sweetmcp-memory/src/memory/retrieval.rs:220 by adding the temporal decay logic that uses this factor.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 16. QA: Temporal Retrieval Decay Factor Review
Act as an Objective QA Rust developer and rate the time decay factor implementation. Verify the time_decay_factor field is properly used in temporal calculations, eliminates the unused field warning, and provides correct temporal retrieval functionality.

### 17. Fix Unused vector_store Field in RetrievalManager
Implement usage of the `vector_store` field in sweetmcp-memory/src/memory/retrieval.rs:256 by connecting it to the retrieval management operations.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 18. QA: RetrievalManager Vector Store Review
Act as an Objective QA Rust developer and rate the vector store implementation in RetrievalManager. Verify the vector_store field is properly used in management operations, eliminates the unused field warning, and provides correct retrieval management functionality.

### 19. Fix Unused config Field in HNSWIndex
Implement usage of the `config` field in sweetmcp-memory/src/vector/vector_index.rs:159 by adding the index configuration logic that uses this field.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 20. QA: HNSW Index Config Review
Act as an Objective QA Rust developer and rate the HNSW index configuration implementation. Verify the config field is properly used in index operations, eliminates the unused field warning, and provides correct vector index configuration functionality.

### 21. Fix Unused content_type Field in FetchResult
Implement usage of the `content_type` field in sweetmcp-plugins/fetch/src/chromiumoxide.rs:36 by adding content type handling to the fetch result processing.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 22. QA: Fetch Result Content Type Review
Act as an Objective QA Rust developer and rate the content type field implementation. Verify the content_type field is properly used in fetch result processing, eliminates the unused field warning, and provides correct content type handling.

### 23. Fix Unused FetchError Enum and HyperFetcher Struct
Implement usage of `FetchError` enum and `HyperFetcher` struct in sweetmcp-plugins/fetch/src/hyper.rs by connecting them to the fetch plugin's HTTP client functionality.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 24. QA: Hyper Fetcher Implementation Review
Act as an Objective QA Rust developer and rate the HyperFetcher and FetchError implementation. Verify the enum and struct are properly used in HTTP operations, eliminate unused warnings, and provide correct HTTP fetch functionality.

### 25. Fix Unused HyperFetcher Methods
Implement usage of `fetch` and `clean_html` methods in sweetmcp-plugins/fetch/src/hyper.rs:75,191 by connecting them to the plugin's fetch operations.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 26. QA: Hyper Fetcher Methods Review
Act as an Objective QA Rust developer and rate the HyperFetcher methods implementation. Verify the fetch and clean_html methods are properly used in fetch operations, eliminate unused method warnings, and provide correct HTML processing functionality.

### 27. Fix Unused FirecrawlError Variants
Implement usage of `Parse`, `Timeout`, and `Internal` variants in sweetmcp-plugins/fetch/src/firecrawl.rs:12-14 by adding proper error handling that uses these variants.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 28. QA: Firecrawl Error Variants Review
Act as an Objective QA Rust developer and rate the FirecrawlError variants implementation. Verify the Parse, Timeout, and Internal variants are properly used in error handling, eliminate unused variant warnings, and provide correct error classification.

### 29. Wire Up Fetch Plugin Integration
Connect the fetch plugin components (HyperFetcher, FetchError, FirecrawlError) to the main plugin system by implementing the proper plugin interface and error propagation.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 30. QA: Fetch Plugin Integration Review
Act as an Objective QA Rust developer and rate the fetch plugin integration. Verify all components are properly connected to the plugin system, eliminate all fetch-related warnings, and provide complete fetch functionality.

### 31. Fix Remaining Unused Enum Variants
Fix any remaining unused enum variants (MissingExecutable, Windows/Linux platform variants, etc.) by implementing the missing functionality that should use these variants.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 32. QA: Remaining Enum Variants Review
Act as an Objective QA Rust developer and rate the remaining enum variants implementation. Verify all enum variants are properly used, eliminate unused variant warnings, and provide correct enum functionality across the codebase.

### 33. Fix Remaining Unused Methods
Fix any remaining unused methods (like `is_running`, `step`, etc.) by implementing the missing functionality that should call these methods.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 34. QA: Remaining Methods Review
Act as an Objective QA Rust developer and rate the remaining methods implementation. Verify all methods are properly used, eliminate unused method warnings, and provide correct method functionality across the codebase.

### 35. Final Compilation and Warning Check
Run comprehensive `cargo check` across entire workspace to verify all warnings are eliminated and no new warnings were introduced.
DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### 36. QA: Final Production Quality Review
Act as an Objective QA Rust developer and rate the final codebase quality. Verify 0 warnings, 0 errors, all functionality properly implemented, architecture intact, and production readiness for investor demonstration. Confirm compliance with all production quality requirements.