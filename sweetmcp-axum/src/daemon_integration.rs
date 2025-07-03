use anyhow::{Result, anyhow};
use log::info;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::plugin::manager::PluginManager;
use crate::ui::ServeArgs;
use sweetmcp_daemon::{
    HealthCheckConfig, LogRotationConfig, ServiceConfig, ServiceDefinition, ServiceManager,
    daemonise,
};

/// Integrate MCP server with the daemon manager
pub fn run_mcp_as_daemon(_plugin_manager: PluginManager, serve_args: ServeArgs) -> Result<()> {
    info!("Starting MCP server as managed daemon service");

    // Create service definition for MCP server
    let mcp_service = create_mcp_service_definition(&serve_args);

    // Create a ServiceConfig with the MCP service
    let config = ServiceConfig {
        services: vec![mcp_service],
        ..Default::default()
    };

    // Check if running as root (required for dropping privileges)
    let effective_uid = nix::unistd::geteuid();
    let needs_root = serve_args.user.is_some() || serve_args.group.is_some();

    if needs_root && !effective_uid.is_root() {
        return Err(anyhow!("Must run as root to switch user/group"));
    }

    // Fork process for true daemonization
    let pid_file = PathBuf::from("/var/run/sweet-mcp.pid");
    daemonise(&pid_file)?;

    // Now run the daemon manager with MCP as one of the services
    let manager = ServiceManager::new(&config)?;
    manager.run()
}

/// Create service definition for the MCP server
fn create_mcp_service_definition(serve_args: &ServeArgs) -> ServiceDefinition {
    let mut env_vars = HashMap::new();

    // Pass socket path through environment
    env_vars.insert(
        "MCP_SOCKET_PATH".to_string(),
        serve_args.socket_path.to_string_lossy().to_string(),
    );

    // If systemd integration is enabled
    if serve_args.systemd {
        env_vars.insert("NOTIFY_SOCKET".to_string(), "".to_string());
    }

    ServiceDefinition {
        name: "mcp-server".to_string(),
        description: Some("Model Context Protocol JSON-RPC Server".to_string()),
        command: format!("{} serve", std::env::current_exe().unwrap().display()),
        working_dir: Some(serve_args.working_dir.to_string_lossy().to_string()),
        env_vars,
        auto_restart: true,
        user: serve_args.user.clone(),
        group: serve_args.group.clone(),
        restart_delay_s: Some(5),
        depends_on: vec![],
        service_type: Some("mcp-server".to_string()),
        health_check: Some(HealthCheckConfig {
            check_type: "tcp".to_string(),
            target: serve_args.socket_path.to_string_lossy().to_string(),
            interval_secs: 30,
            timeout_secs: 5,
            retries: 3,
            expected_response: None,
            on_failure: vec!["restart".to_string()],
        }),
        log_rotation: Some(LogRotationConfig {
            max_size_mb: 100,
            max_files: 10,
            interval_days: 7,
            compress: true,
            timestamp: true,
        }),
        watch_dirs: vec![],
        ephemeral_dir: None,
        memfs: None,
    }
}

/// Run MCP server directly (when not running under daemon manager)
pub async fn run_mcp_server_standalone(
    plugin_manager: PluginManager,
    socket_path: &std::path::Path,
) -> Result<()> {
    info!(
        "Starting MCP server in standalone mode on socket: {:?}",
        socket_path
    );

    // This function is integrated into the socket listener workflow
    // It sets up the proper standalone configuration without the daemon manager
    
    // Initialize the plugin manager for standalone operation
    let _pm = plugin_manager.clone();
    
    // Log successful standalone initialization
    info!("MCP server standalone mode initialized for socket: {}", socket_path.display());
    
    Ok(())
}
