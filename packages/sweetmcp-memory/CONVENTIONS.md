# Coding Conventions and Acceptance Standards

## Core Principles

1. **Production-Quality Code Only**
   - All code must be production-ready, deployment-ready, and complete
   - No simulations, fabrications, mocks, or prototypes
   - No comments like "In a real implementation..." or "In production..."
   - Complete, unabridged code solutions only

2. **No Future Improvements**
   - No areas for potential future improvement
   - All identified improvements must be implemented immediately
   - Apply ultrathink, step-by-step sequential reasoning to solve all issues
   - Iterate until there is zero need for future enhancements

3. **Minimal Changes**
   - Make only the minimal changes needed to implement required features
   - Do not remove or simplify features when challenges arise
   - Ask for help and direction when facing challenges

## Async Compliance

1. **Trait Implementation**
   - ❌ NEVER use `async_trait`
   - ❌ NEVER use `async fn` in traits
   - ✅ Always use hidden box pin for trait methods returning futures

2. **Return Types**
   - ❌ NEVER return `Box<dyn Future>` or `Pin<Box<dyn Future>>` from client interfaces
   - ✅ Provide synchronous public interfaces that return `Stream<FriendlyDomainObject>` or `AsyncFriendlyDomainObject`
   - ✅ Hide async complexity behind `channel` and `task` `spawn`
   - ✅ Return intuitive, domain-specific types (e.g., `AgentResponse`, `TranscriptionStream`)

3. **Implementation Pattern**
   ```rust
   // Instead of this:
   trait BadTrait {
       async fn bad_method(&self) -> Result<T>;
   }
   
   // Do this:
   trait GoodTrait {
       fn good_method(&self) -> Pin<Box<dyn Future<Output = Result<T>> + Send + '_>>;
   }
   ```

## Rust Best Practices

1. **Error Handling**
   - Use `thiserror` for defining error types
   - Propagate errors with `?` operator
   - Never use `unwrap()` or `expect()` in production code
   - Provide meaningful error messages

2. **Memory Management**
   - Use `Arc` for shared ownership
   - Use `Mutex` or `RwLock` for shared mutable state
   - Avoid unnecessary cloning
   - Prefer references over owned values when appropriate

3. **Concurrency**
   - Use Tokio for async runtime
   - Properly handle cancellation and timeouts
   - Avoid blocking the async runtime
   - Use channels for communication between tasks

4. **Testing**
   - Write unit tests for all public functions
   - Write integration tests for key workflows
   - Use mocks for external dependencies
   - Test error cases and edge conditions

## SurrealDB Best Practices

1. **Schema Definition**
   - Define explicit schemas for all tables
   - Use appropriate field types
   - Define indexes for frequently queried fields
   - Use vector indexes for similarity search

2. **Query Optimization**
   - Use parameterized queries
   - Leverage SurrealDB's query planner
   - Use appropriate indexes for queries
   - Monitor query performance

3. **Transaction Management**
   - Use transactions for multi-step operations
   - Properly handle transaction errors
   - Implement appropriate retry logic

4. **Graph Capabilities**
   - Leverage SurrealDB's native graph capabilities
   - Use graph traversal for relationship queries
   - Implement proper relationship modeling

## Documentation

1. **Code Documentation**
   - Document all public functions and types
   - Provide examples for complex functions
   - Document error conditions and edge cases
   - Keep documentation up-to-date with code changes

2. **Project Documentation**
   - Maintain comprehensive README
   - Document architecture and design decisions
   - Provide usage examples
   - Include deployment instructions

## Versioning and Compatibility

1. **Library Versions**
   - Use the latest stable versions of all dependencies
   - Check compatibility between dependencies
   - Document version requirements

2. **API Stability**
   - Maintain backward compatibility
   - Properly document breaking changes
   - Follow semantic versioning

## Performance

1. **Optimization**
   - Optimize critical paths
   - Use appropriate data structures
   - Minimize allocations
   - Profile and benchmark performance-critical code

2. **Resource Management**
   - Properly manage connections
   - Implement appropriate caching
   - Handle resource cleanup
   - Monitor resource usage

## Security

1. **Input Validation**
   - Validate all user input
   - Sanitize data before storage
   - Prevent injection attacks

2. **Authentication and Authorization**
   - Implement proper authentication
   - Enforce appropriate access controls
   - Use secure communication channels

## Compliance Verification

All code must be verified against these standards before being considered complete. No exceptions or compromises are permitted.
