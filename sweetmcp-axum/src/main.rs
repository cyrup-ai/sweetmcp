use anyhow::Result;
use sweetmcp_axum::{config, plugin::load_plugins, ui};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Parse CLI args just to get config/log path/level and insecure flag
    let cli = ui::parse_cli_args();

    // 2. Initialize Logger
    config::init_logger(cli.log_path.as_deref(), Some(&cli.log_level))?;

    // 3. Load config - either from file or use default
    let config = if cli.config.exists() {
        let config_content = tokio::fs::read_to_string(&cli.config).await?;
        config::validate_config(&config_content)?;
        config::parse_config::<sweetmcp_axum::config::Config>(&config_content, &cli.config)?
    } else {
        // Use default config with no plugins
        sweetmcp_axum::config::Config {
            plugins: Vec::new(),
            database: None,
        }
    };

    // 4. Load plugins
    let plugin_manager: sweetmcp_axum::plugin::PluginManager =
        load_plugins(&config.plugins, cli.insecure_skip_signature).await;

    // 6. Pass control to UI for command dispatch, passing all needed state
    ui::run_ui_with_state(cli, config, plugin_manager).await
}
