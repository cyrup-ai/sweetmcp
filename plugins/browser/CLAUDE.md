# CONVENTIONS

## async FTW w/ "Hidden Box/Pin"

- ❌ NEVER use `async_trait` or `async fn` in traits
- ❌ NEVER return `Box<dyn Future>` or `Pin<Box<dyn Future>>` from client interfaces
- ✅ Provide synchronous interfaces with `.await()` called internally
- ✅ Hide async complexity behind `channel` and `task` `spawn`
- ✅ Return intuitive, domain-specific types (e.g., `AgentResponse`, `TranscriptionStream`)
- ✅ Example: Method returns `TranscriptionStream` (not `Box<dyn Stream>`) that user consumes with `.next().await`.

## Yo **CAVEMAN!** Read the damn docs

- Let's face it. You are a brilliant engineer who's been out of the game a couple years. That's like 20 years in any other field.
- GO GET THE LATEST DOCS!! Don't assume your tools from before the wheel are the best tools or the right syntax.
- `cargo docs {{package_id}} --open` and MAKE A LITTLE GUIDE.md in `docs/` for yourself with specific snippets that match the operations this project is specifically doing. Link it in `CONVENTIONS.md`, `README.md`, `ARCHITECTURE.md`.

## `cargo` rules & `Cargo.toml`

- !! DO NOT edit `Cargo.toml` directly !!
- Always use the latest version of each dependency unless an exception in writing is granted.
  - `cargo search {{package_id}} limit 1`
  - `cargo add` will save you lots of time as it will ensure the latest version is imported.
- use `cargo` to add, remove, update or upgrade packages.
- Learn `cargo edit` and `cargo workspace` and you'll be good to go.
- Lint (after EVERY change): `cargo fmt && cargo check --message-format short --quiet`
- Build: `cargo build`, Run: `cargo run`
- Test: Always use `nextest` and `cargo test`

### TODO: Cargo Workspace Versioning

Currently, most crates in this workspace use explicit versions (0.1.0) rather than inheriting from the workspace.package settings. There's an ongoing effort to standardize this by:

1. Converting all crates to use version.workspace = true and edition.workspace = true
2. Several crates have compilation issues that need to be fixed before this can be completed:
   - krater: Issues with async/Send trait bounds in the updater.rs file
   - cli-compositor: Multiple compilation errors related to dependencies and API usage
   - cyrup-cli: Issues with async_trait and EventListener trait
   - mcp_server: Issues with Json types and conversions

See TODOs in individual Cargo.toml files for more details.

## Error Handling

- Use Result<T,E> with custom errors
- No unwrap() except in tests
- Handle all Result/Option values explicitly

## Style & Structure

- No single file should be more than 300 lines long. Decompose once we hit that size into elegant modules that fully handle concerns.
- Rust official style: snake_case for variables/functions
- Tests in `tests/` directory only
- Use `tracing` for logs with appropriate levels
- ❌ NO suppression of compiler or clippy warnings
- ✅ All code MUST pass `cargo check --message-format short --quiet -- -D warnings` without exception. Ask if you believe there's a valid exception and document it in writing after approval.

## Be a Software Artisan

- Focus on interface first.
  - Who is using this product? How can we make this drop in easy for them to adopt?
  - How are they using it? What is intuitive in this context?
  - Ask questions before making up features we don't need.
- WRITE THE *MINIMAL AMOUNT OF CODE* NEEDED TO IMPACT A CHANGE (but do it fully and correctly)
  - Do not add features that are not requested.
  - NEVER EVER ADD `examples/*` unless Dave asks for them.
  - DO ADD tests in nextest. Focus on the key elements that prove it is really working for the user of the software.
  - DO NOT say "it's all good" or "completed" unless you have **tested like an end-user** (ie. `cargo run` for a bin) and verified the feature.
  - DO NOT add more than one binary per crate.

## "REAL WORLD" Rules

- ✅ All warnings must be fully resolved, not suppressed.
- DO NOT use annotations to suppress warnings of any type.
- DO NOT use private _variable naming to hide warnings.
  - Unused code is either:
    1. a bug that needs implementation to function
    2. a half-assed non-production implementation
    3. a mess that makes it hard to read and understand
- NEVER leave **"TODO: in a real world situation ..."** or *"In production we'll handle this differently ..."* or similar suggestions.
- *WRITE PRODUCTION QUALITY CODE ALL THE TIME!*. The future is now. This is production. You are the best engineers I know. Rise up.
- ASK, ASK, ASK -- I love your initiative but writing full modules that are all wrong is costly and time consuming to cleanup. Just ask and don't assume anything. I'll hurry along when it's time :)

## SurrealDB (awesome)

- Use SurrealDB for all database operations
- The syntax in version 2.2.1 has changed significantly
- use `kv-surrealkv` local file databases and `kv-tikv` for clustered/distributed databases.
- use the appropriate table type for the job (document, relational, graph, time series, vector)
- use `surrealdb-migrations` version 2.2+ for perfectly versioned migrations. This is really essential for distributed file-based data.
- use our `cyrup-ai/surrealdb-client` to get up and running fast with elegant traits and base implementations.

## Preferred Software

- `dioxus` (pure Rust front-end)
- `axum` (elite tokio based server)
- `floneum/floneum` ask "Kalosm" (local agents with superpowers)
- `surrealdb` (swiss army knife of fast, indexed storage and ML support)
- `livekit` for open-source real-time audio/video
- `clap`, `ratatui`, `crossterm` ... just amazing cli tools
- `serde` for serialization/deserialization
- `tokio` for asynchronous programming and io bound parallelism
- `rayon` for cpu bound parallelism
- `nextest` for hella-fast and lovely test execution
- `chromiumoxide` for web browser automation
