# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## COMMANDS

- Build: `cargo build [--release]`
- Check: `cargo check --message-format short --quiet -- -D warnings`
- Lint: `cargo fmt && cargo check --message-format short --quiet`
- Test all: `cargo test --all-features`
- Test single: `cargo test -p llm_client test_name`
- Run integration tests: `cargo test --test it`

## CODE STYLE

- File size: Max 300 lines before decomposing into modules
- Naming: snake_case for variables/functions, PascalCase for types
- Imports: Group in logical order (std, external, internal)
- Error handling: Use Result<T,E> with custom error types from thiserror
- Never use unwrap() except in tests
- Use tracing for logs with appropriate levels
- Traits: Never use async_trait or async fn in traits
- Return domain-specific types, not Box<dyn Future>
- Follow Rust official style guide
- No suppression of compiler/clippy warnings