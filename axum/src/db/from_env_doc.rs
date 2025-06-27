impl DatabaseConfig {
    /// Construct a DatabaseConfig from environment variables.
    ///
    /// Supported environment variables (in order of precedence):
    /// - DB_ENGINE: "memory", "localkv", "surrealkv", "tikv", "websocket"
    /// - DB_PATH: file path for LocalKv/SurrealKv
    /// - DB_URL: URL for remote engines (WebSocket, TiKv)
    /// - DB_NAMESPACE or MCP_DB_WS_NS or MCP_DB_TIKV_NS
    /// - DB_DATABASE or MCP_DB_WS_DB or MCP_DB_TIKV_DB
    /// - DB_USERNAME or MCP_DB_WS_USER or MCP_DB_TIKV_USER
    /// - DB_PASSWORD or MCP_DB_WS_PASS or MCP_DB_TIKV_PASS
    /// - DB_RUN_MIGRATIONS: "true" or "false"
    /// For legacy SurrealDB setups:
    /// - MCP_DB_WS_ENDPOINT, MCP_DB_TIKV_ENDPOINT as fallbacks for DB_URL
    pub fn from_env() -> Self {
        let engine = std::env::var("DB_ENGINE")
            .ok()
            .and_then(|e| match e.to_lowercase().as_str() {
                "memory" => Some(StorageEngine::Memory),
                "localkv" => Some(StorageEngine::LocalKv),
                "surrealkv" => Some(StorageEngine::SurrealKv),
                "tikv" => Some(StorageEngine::TiKv),
                "websocket" => Some(StorageEngine::WebSocket),
                _ => None,
            })
            .unwrap_or(StorageEngine::default());

        let path = std::env::var("DB_PATH").ok();
        let url = std::env::var("DB_URL")
            .ok()
            .or_else(|| std::env::var("MCP_DB_WS_ENDPOINT").ok())
            .or_else(|| std::env::var("MCP_DB_TIKV_ENDPOINT").ok());
        let namespace = std::env::var("DB_NAMESPACE")
            .ok()
            .or_else(|| std::env::var("MCP_DB_WS_NS").ok())
            .or_else(|| std::env::var("MCP_DB_TIKV_NS").ok())
            .or_else(|| Some("mcp".to_string()));
        let database = std::env::var("DB_DATABASE")
            .ok()
            .or_else(|| std::env::var("MCP_DB_WS_DB").ok())
            .or_else(|| std::env::var("MCP_DB_TIKV_DB").ok())
            .or_else(|| Some("chat_sessions".to_string()));
        let username = std::env::var("DB_USERNAME")
            .ok()
            .or_else(|| std::env::var("MCP_DB_WS_USER").ok())
            .or_else(|| std::env::var("MCP_DB_TIKV_USER").ok());
        let password = std::env::var("DB_PASSWORD")
            .ok()
            .or_else(|| std::env::var("MCP_DB_WS_PASS").ok())
            .or_else(|| std::env::var("MCP_DB_TIKV_PASS").ok());
        let run_migrations = std::env::var("DB_RUN_MIGRATIONS")
            .ok()
            .map(|v| v == "true")
            .unwrap_or(true);

        DatabaseConfig {
            engine,
            path,
            url,
            namespace,
            database,
            username,
            password,
            run_migrations,
        }
    }
}
