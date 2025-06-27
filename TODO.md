# SweetMCP Panic Elimination TODO List

## Unwrap/Expect Removal Tasks (101 Total Panic Points)

### 1. auth.rs Panic Elimination
- [ ] Replace JWT creation unwrap() at line 301 with proper error handling
  - Convert to Result<String, AuthError> propagation
  - Handle encoding failures with descriptive error messages
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on JWT creation error handling. Verify: 1) No unwrap() remains, 2) Error properly propagated, 3) Error messages are descriptive, 4) No functionality lost. Use sequential thinking.

- [ ] Replace JWT verification unwrap() at line 303 with match expression
  - Handle invalid token format errors
  - Handle signature verification failures
  - Handle expired token cases
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on JWT verification error handling. Verify: 1) All error cases handled, 2) Security not compromised, 3) Clear error messages, 4) Proper error types used. Use sequential thinking.

### 2. circuit_breaker.rs Panic Elimination
- [ ] Replace all Mutex lock unwrap() calls with poisoned lock handling
  - Identify all mutex.lock().unwrap() patterns
  - Convert to match with PoisonError recovery
  - Log poisoned lock occurrences for debugging
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on Mutex lock handling. Verify: 1) All unwrap() on locks removed, 2) Poisoned locks properly recovered, 3) State consistency maintained, 4) Performance not degraded. Use sequential thinking.

### 3. config.rs Panic Elimination
- [ ] Replace parse_duration test unwrap() calls (lines 234-237) with proper assertions
  - Convert to assert_eq! with expect messages for test clarity
  - Ensure test failures provide diagnostic information
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on test assertion improvements. Verify: 1) Tests still validate correctly, 2) Failure messages are helpful, 3) No test coverage lost, 4) Tests remain readable. Use sequential thinking.

- [ ] Replace Config::from_env() unwrap() at line 249 with error propagation
  - Convert test to return Result<(), Box<dyn Error>>
  - Use ? operator for cleaner error handling
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on config test error handling. Verify: 1) Test properly propagates errors, 2) Environment variable errors are clear, 3) Test maintains its validation purpose, 4) No test functionality lost. Use sequential thinking.

### 4. crypto.rs Panic Elimination
- [ ] Replace all RwLock unwrap() calls with poisoned lock recovery
  - Handle token storage lock failures
  - Ensure cryptographic operations never panic
  - Add error context for debugging
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on crypto RwLock handling. Verify: 1) All lock unwraps removed, 2) Cryptographic security maintained, 3) Error recovery doesn't leak sensitive data, 4) Performance acceptable. Use sequential thinking.

### 5. edge.rs Panic Elimination
- [ ] Replace load counter Mutex unwrap() calls (lines 102, 157, 398) with proper handling
  - Convert to match expressions
  - Handle poisoned mutex by resetting counter state
  - Ensure load tracking remains accurate
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on load counter mutex handling. Verify: 1) All mutex unwraps removed, 2) Load counting remains accurate, 3) Poisoned lock recovery is safe, 4) No race conditions introduced. Use sequential thinking.

### 6. main.rs Panic Elimination
- [ ] Audit and replace all unwrap() calls in main() with proper error handling
  - Use anyhow::Result<()> for main return type
  - Convert all unwraps to ? operator where appropriate
  - Add context to errors for better diagnostics
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on main.rs error handling. Verify: 1) Application starts gracefully, 2) All startup errors properly reported, 3) Exit codes are meaningful, 4) No silent failures. Use sequential thinking.

### 7. mcp_bridge.rs Panic Elimination
- [ ] Replace all protocol bridging unwrap() calls with error propagation
  - Handle message parsing failures
  - Handle protocol conversion errors
  - Ensure bridge remains stable under errors
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on MCP bridge error handling. Verify: 1) Protocol errors handled gracefully, 2) Bridge doesn't drop connections on errors, 3) Error messages help debugging, 4) Protocol compliance maintained. Use sequential thinking.

### 8. mdns_discovery.rs Panic Elimination
- [ ] Replace all socket operation unwrap() calls with proper error handling
  - Handle socket binding failures
  - Handle multicast join failures
  - Handle packet send/receive errors
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on mDNS socket error handling. Verify: 1) Network errors handled gracefully, 2) Service discovery continues despite errors, 3) Resource cleanup on errors, 4) No network loops on errors. Use sequential thinking.

### 9. metric_picker.rs Panic Elimination
- [ ] Replace partial_cmp unwrap() at line 90 with safe comparison
  - Handle NaN values in load metrics
  - Provide fallback for comparison failures
  - Ensure metric selection remains deterministic
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on metric comparison handling. Verify: 1) NaN values handled safely, 2) Metric selection remains stable, 3) No infinite loops possible, 4) Performance not impacted. Use sequential thinking.

### 10. metrics.rs Panic Elimination
- [ ] Replace all Prometheus metric unwrap() calls with error handling
  - Handle metric registration failures
  - Handle metric update failures
  - Ensure metrics don't crash the application
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on metrics error handling. Verify: 1) Metric failures don't crash app, 2) Metrics remain accurate when possible, 3) Error recovery is automatic, 4) Performance overhead minimal. Use sequential thinking.

### 11. peer_discovery.rs Panic Elimination
- [ ] Replace RwLock unwrap() at line 88 with poisoned lock handling
  - Handle peer registry lock failures
  - Ensure peer list consistency
  - Add recovery mechanism for poisoned locks
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on peer registry lock handling. Verify: 1) Lock failures handled gracefully, 2) Peer discovery continues working, 3) No peer list corruption, 4) Distributed system stability maintained. Use sequential thinking.

### 12. rate_limit.rs Panic Elimination
- [ ] Replace all time calculation unwrap() calls with saturating operations
  - Handle time arithmetic overflows
  - Handle system time errors
  - Ensure rate limiting remains functional
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on rate limit time handling. Verify: 1) Time errors handled safely, 2) Rate limiting remains accurate, 3) No DOS vulnerabilities introduced, 4) Performance maintained. Use sequential thinking.

### 13. shutdown.rs Panic Elimination
- [ ] Replace shutdown signal unwrap() at line 251 with error handling
  - Handle signal registration failures
  - Ensure graceful shutdown works regardless
  - Add fallback shutdown mechanisms
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on shutdown signal handling. Verify: 1) Shutdown remains graceful, 2) All cleanup runs on shutdown, 3) No resource leaks on error, 4) Exit is always clean. Use sequential thinking.

- [ ] Replace tempdir() unwrap() calls in tests (lines 503, 523, 534, 541, 548, 563, 579)
  - Convert tests to return Result
  - Handle temp directory creation failures
  - Ensure test cleanup even on failures
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on test temp directory handling. Verify: 1) Tests handle filesystem errors, 2) Cleanup always runs, 3) Test failures are informative, 4) No temp file leaks. Use sequential thinking.

### 14. tls/ocsp.rs Panic Elimination
- [ ] Replace HTTP client builder unwrap() with fallback client
  - Handle client configuration failures
  - Ensure OCSP validation continues with defaults
  - Add appropriate logging for failures
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on OCSP HTTP client handling. Verify: 1) Client creation never fails, 2) OCSP validation remains functional, 3) Timeouts still enforced, 4) Security not compromised. Use sequential thinking.

- [ ] Replace RwLock unwrap() calls for cache and nonce pool access
  - Handle poisoned locks in OCSP cache
  - Ensure nonce generation continues on lock failures
  - Maintain cache consistency
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on OCSP lock handling. Verify: 1) Cache operations handle lock failures, 2) Nonce generation never fails, 3) OCSP validation continues working, 4) No security vulnerabilities. Use sequential thinking.

### 15. tls/tls_manager.rs Panic Elimination
- [ ] Replace NonZeroU32::new unwrap() calls with const assertions
  - Use NonZeroU32::new_unchecked in const context with compile-time validation
  - Ensure PBKDF2 iterations are always valid
  - Maintain cryptographic security parameters
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on NonZeroU32 handling. Verify: 1) Values are compile-time validated, 2) No runtime panics possible, 3) Security parameters unchanged, 4) Code remains clear. Use sequential thinking.

- [ ] Replace certificate parsing unwrap() and expect() calls with error propagation
  - Handle malformed certificate data
  - Provide meaningful error messages for certificate issues
  - Ensure TLS setup fails safely
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on certificate parsing error handling. Verify: 1) All parsing errors handled, 2) Error messages help diagnose issues, 3) TLS security maintained, 4) No silent TLS failures. Use sequential thinking.

## Global Pattern Replacements

### 16. Mutex/RwLock Pattern Replacement
- [ ] Create a generic utility function for safe lock handling
  - Implement recover_poisoned_lock<T> helper
  - Use throughout codebase for consistency
  - Log all poison recoveries for debugging
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on lock utility implementation. Verify: 1) Utility handles all lock types, 2) Recovery is safe and consistent, 3) Logging is helpful, 4) Performance overhead minimal. Use sequential thinking.

### 17. Time Arithmetic Pattern Replacement
- [ ] Replace all duration arithmetic unwrap() with saturating operations
  - Use checked_add/sub with saturation fallback
  - Handle time going backwards gracefully
  - Ensure monotonic behavior where needed
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on time arithmetic safety. Verify: 1) No time panics possible, 2) Time calculations remain accurate, 3) Monotonic guarantees preserved, 4) Performance acceptable. Use sequential thinking.

### 18. Option Unwrap Pattern Replacement
- [ ] Replace all Option::unwrap() with explicit match or ok_or_else
  - Provide context-specific error messages
  - Use ? operator where appropriate
  - Ensure None cases are handled logically
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on Option handling. Verify: 1) All None cases handled, 2) Error messages are helpful, 3) Logic remains correct, 4) No silent failures. Use sequential thinking.

### 19. Result Unwrap Pattern Replacement
- [ ] Replace all Result::unwrap() with ? operator or match
  - Add error context using anyhow::Context where needed
  - Ensure error types are properly converted
  - Maintain error chain for debugging
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the work performed on Result error propagation. Verify: 1) All errors properly propagated, 2) Error context is meaningful, 3) Error chains preserved, 4) Recovery attempted where sensible. Use sequential thinking.

## Final Verification

### 20. Comprehensive Panic Audit
- [ ] Run `find src -name "*.rs" | xargs grep -n "\.unwrap()\|\.expect("` and verify zero results
  - Document any remaining cases that are genuinely safe (e.g., const contexts)
  - Ensure no new unwrap() calls were introduced during refactoring
  - Verify all panic points eliminated
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the overall panic elimination project. Verify: 1) Zero unwrap/expect remain, 2) Error handling is consistent, 3) Application stability improved, 4) No functionality lost. Run panic-inducing scenarios. Use sequential thinking.

### 21. Error Recovery Testing
- [ ] Create test suite for error conditions
  - Test each error path introduced
  - Verify graceful degradation
  - Ensure no error cascades cause issues
  - DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

- [ ] Act as an Objective QA Rust developer - rate the error recovery test coverage. Verify: 1) All error paths tested, 2) Recovery works as designed, 3) No error amplification, 4) System remains stable under errors. Use sequential thinking.

## Completion Criteria

- All 101 unwrap()/expect() calls replaced with proper error handling
- Application cannot panic from normal operations
- All errors handled gracefully with appropriate recovery or propagation
- Error messages provide sufficient context for debugging
- Performance impact of error handling is negligible
- Test coverage includes error paths
- Documentation updated to reflect error handling strategies