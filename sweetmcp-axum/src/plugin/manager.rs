use std::{collections::HashMap, sync::Arc};

use extism::convert::Json; // Ensure import exists
use extism::*;
use rpc_router::RpcResource;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::sync::{Mutex, RwLock, oneshot};

use crate::{
    config::PluginConfig,
    container_registry::pull_and_extract_oci_image,
    types::{ClientCapabilities, Prompt},
};

/// The main plugin manager struct, holding all plugin-related state.
#[derive(Clone, RpcResource)]
pub struct PluginManager {
    pub plugins: Arc<RwLock<HashMap<String, Plugin>>>,
    // Cache to map tool names to plugin names
    pub tool_to_plugin: Arc<RwLock<HashMap<String, String>>>,
    // Cache to map prompt names to plugin names and prompt metadata
    pub prompt_info: Arc<RwLock<HashMap<String, (String, Prompt)>>>,
    // Store client capabilities received during initialization
    pub client_capabilities: Arc<RwLock<Option<ClientCapabilities>>>,
    // Channel/Map for sending outgoing requests and receiving responses
    // Key: request_id (String), Value: Sender channel for the response
    pub pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<Value>>>>,
}

impl PluginManager {
    /// Create a new, empty PluginManager.
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            tool_to_plugin: Arc::new(RwLock::new(HashMap::new())),
            prompt_info: Arc::new(RwLock::new(HashMap::new())),
            client_capabilities: Arc::new(RwLock::new(None)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Load, discover, and cache all plugins as described in the config.
/// Returns a fully initialized PluginManager.
pub async fn load_plugins(
    configs: &[PluginConfig],
    insecure_skip_signature: bool,
) -> PluginManager {
    // Added return type annotation
    let manager = PluginManager::new(); // Use immutable manager initially

    for plugin_cfg in configs {
        let wasm_content = if plugin_cfg.path.starts_with("http") {
            match reqwest::get(&plugin_cfg.path).await {
                Ok(resp) => match resp.bytes().await {
                    Ok(bytes) => bytes.to_vec(),
                    Err(e) => {
                        log::error!("Failed to download plugin {}: {}", plugin_cfg.path, e);
                        continue;
                    }
                },
                Err(e) => {
                    log::error!("Failed to download plugin {}: {}", plugin_cfg.path, e);
                    continue;
                }
            }
        } else if plugin_cfg.path.starts_with("oci://") {
            // Match full prefix
            // ref should be like oci://tuananh/qr-code
            // Use map_err or expect for better error handling
            let image_reference = plugin_cfg
                .path
                .strip_prefix("oci://")
                .expect("OCI path should start with oci://"); // Expect acceptable if format is guaranteed
            let target_file_path = "/plugin.wasm";
            let mut hasher = Sha256::new();
            hasher.update(image_reference);
            let hash = hasher.finalize();
            let short_hash = &hex::encode(hash)[..7];
            let cache_dir = dirs::cache_dir()
                .map(|mut path| {
                    path.push("cyrup-mcp"); // Use consistent cache dir name
                    path
                })
                .expect("Failed to determine cache directory"); // Expect acceptable for critical paths
            std::fs::create_dir_all(&cache_dir).ok(); // ok() is fine, ignore error if dir exists

            let local_output_path =
                cache_dir.join(format!("{}-{}.wasm", plugin_cfg.name, short_hash));
            // Use expect for critical path conversion
            let local_output_path_str = local_output_path
                .to_str()
                .expect("Local cache path is not valid UTF-8");

            // Use the CLI flag to determine whether to skip signature verification
            let verify_signature = !insecure_skip_signature;

            if let Err(e) = pull_and_extract_oci_image(
                image_reference,
                target_file_path,
                local_output_path_str, // Use correct variable
                verify_signature,
            )
            .await
            {
                log::error!("Error pulling oci plugin: {}", e);
                continue;
            }
            log::info!(
                "cache plugin `{}` to : {}",
                plugin_cfg.name,
                local_output_path.display() // Ensure .display() is used
            );
            match tokio::fs::read(&local_output_path).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    log::error!(
                        "Failed to read cached plugin {}: {}",
                        local_output_path.display(),
                        e
                    );
                    continue;
                }
            }
        } else {
            match tokio::fs::read(&plugin_cfg.path).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    log::error!("Failed to read plugin file {}: {}", plugin_cfg.path, e);
                    continue;
                }
            }
        };

        let mut manifest = Manifest::new([Wasm::data(wasm_content)]);
        if let Some(runtime_cfg) = &plugin_cfg.env {
            log::info!("runtime_cfg: {:?}", runtime_cfg);
            if let Some(hosts) = &runtime_cfg.allowed_hosts {
                for host in hosts {
                    manifest = manifest.with_allowed_host(host);
                }
            }
            if let Some(paths) = &runtime_cfg.allowed_paths {
                for path in paths {
                    // path will be available in the plugin with exact same path
                    manifest = manifest.with_allowed_path(path.clone(), path.clone());
                }
            }

            // Add plugin configurations if present (using additional_vars)
            for (key, value) in &runtime_cfg.additional_vars {
                // Use additional_vars
                manifest = manifest.with_config_key(key, value);
            }
        }
        let mut plugin = match Plugin::new(&manifest, [], true) {
            Ok(p) => p,
            Err(e) => {
                log::error!(
                    "Failed to initialize plugin '{}' from {}: {}",
                    plugin_cfg.name,
                    plugin_cfg.path,
                    e
                );
                continue; // Skip this plugin
            }
        };

        let plugin_name = plugin_cfg.name.clone();

        // Discover Tools
        match plugin.call::<&str, Json<crate::types::ListToolsResult>>(
            "main_handler",
            &json!({ "name": "describe"}).to_string(),
        ) {
            Ok(Json(parsed)) => {
                // Ensure destructuring
                let mut cache = manager.tool_to_plugin.write().await;
                for tool in parsed.tools {
                    // Ensure access is correct
                    log::info!("Saving tool {}/{} to cache", plugin_name, tool.name);
                    if let Some(existing_plugin) = cache.get(&tool.name) {
                        if existing_plugin != &plugin_name {
                            log::error!(
                                "Tool name collision detected: '{}' is provided by both '{}' and '{}' plugins. Skipping tool from '{}'.",
                                tool.name,
                                existing_plugin,
                                plugin_name,
                                plugin_name
                            );
                            continue;
                        }
                    }
                    cache.insert(tool.name, plugin_name.clone());
                }
            }
            Err(e) => {
                log::warn!(
                    "Plugin '{}' failed to describe tools (main_handler describe): {}. Does it export 'main_handler' or 'describe'?",
                    plugin_name,
                    e
                );
            }
        }

        // Discover Prompts
        match plugin.call::<(), Json<Vec<Prompt>>>("mcp_list_prompts", ()) {
            // Wrap return type in Json<>
            Ok(Json(discovered_prompts)) => {
                // Destructure Json()
                let mut cache = manager.prompt_info.write().await;
                for prompt_data in discovered_prompts {
                    log::info!(
                        "Saving prompt {}/{} to cache",
                        plugin_name,
                        prompt_data.name
                    );
                    if let Some((existing_plugin, _)) = cache.get(&prompt_data.name) {
                        if existing_plugin != &plugin_name {
                            log::error!(
                                "Prompt name collision detected: '{}' is provided by both '{}' and '{}' plugins. Skipping prompt from '{}'.",
                                prompt_data.name,
                                existing_plugin,
                                plugin_name,
                                plugin_name
                            );
                            continue;
                        }
                    }
                    cache.insert(prompt_data.name.clone(), (plugin_name.clone(), prompt_data));
                }
            }
            Err(e) => {
                log::warn!(
                    "Plugin '{}' failed during prompt discovery: {}. Does it export 'mcp_list_prompts'?",
                    plugin_name,
                    e
                );
            }
        }

        // Store the plugin itself
        manager
            .plugins
            .write()
            .await
            .insert(plugin_name.clone(), plugin);
        log::info!("Loaded plugin {} successfully", plugin_name);
    }

    manager
}
