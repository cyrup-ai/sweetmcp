# Claude Development Guidelines for SweetMCP

## Test Organization

**IMPORTANT**: Tests should NEVER be placed in `src/**` files. All tests must be placed in the `tests/` directory (sister to `src/`).

- Unit tests: `tests/unit/<module_name>.rs`
- Integration tests: `tests/integration/<feature_name>.rs`
- The `#[cfg(test)]` blocks in source files should be removed and moved to appropriate test files

## Code Quality Standards

- Always run `cargo fmt` before committing
- Always run `cargo clippy` and fix all warnings
- Always run `cargo test` to ensure all tests pass