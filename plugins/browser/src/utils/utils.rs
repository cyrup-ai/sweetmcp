use std::env;
use std::path::{Path, PathBuf};

use kalosm::language::*;
use kalosm_llama::{Llama, LlamaSource};
use tokio::runtime::Handle;
use tracing::{debug, info};

use crate::Config;
use crate::load_yaml_config;
use crate::utils::errors::UtilsError;

pub fn llama() -> Result<Llama, UtilsError> {
    // Load config from YAML file
    let config = load_yaml_config().unwrap_or_else(|_| {
        debug!("No valid config.yaml found, using default LLM configuration");
        Config::default()
    });
    
    // Log the requested configuration
    info!(
        "Initializing Phi-4 LLM with temperature: {}", 
        config.temperature
    );
    
    // Use the Llama builder with phi-4 source
    // Use the current runtime to execute the async build operation
    let rt_handle = Handle::current();
    let model = rt_handle.block_on(async {
        Llama::builder()
            .with_source(LlamaSource::phi_4())
            .build()
            .await
            .map_err(|e| UtilsError::ModelError(e.to_string()))
    })?;
    
    Ok(model)
}

/// Encode an image file to base64
pub fn encode_image(img_path: Option<&str>) -> Result<Option<String>, UtilsError> {
    if let Some(path) = img_path {
        let image_data = std::fs::read(path)
            .map_err(|e| UtilsError::IoError(e.to_string()))?;
            
        let encoded = base64::encode(&image_data);
        Ok(Some(encoded))
    } else {
        Ok(None)
    }
}

/// Find the latest files with specified extensions in a directory
pub fn get_latest_files(
    directory: &str,
    file_types: &[&str],
) -> Result<std::collections::HashMap<String, Option<String>>, UtilsError> {
    let mut latest_files = std::collections::HashMap::new();
    
    // Initialize with None values
    for &file_type in file_types {
        latest_files.insert(file_type.to_string(), None);
    }
    
    let dir_path = Path::new(directory);
    
    // Create directory if it doesn't exist
    if !dir_path.exists() {
        std::fs::create_dir_all(dir_path)
            .map_err(|e| UtilsError::IoError(format!("Failed to create directory: {}", e)))?;
            
        return Ok(latest_files);
    }
    
    // Find latest files for each type
    for &file_type in file_types {
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            let mut latest_time = std::time::SystemTime::UNIX_EPOCH;
            let mut latest_file: Option<PathBuf> = None;
            
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                
                if let Some(ext) = path.extension() {
                    if ext == file_type.trim_start_matches('.') {
                        if let Ok(metadata) = std::fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                // Make sure the file is not being written (at least 1 second old)
                                if let Ok(elapsed) = modified.elapsed() {
                                    if elapsed.as_secs() > 1 && modified > latest_time {
                                        latest_time = modified;
                                        latest_file = Some(path.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            if let Some(file) = latest_file {
                latest_files.insert(file_type.to_string(), Some(file.to_string_lossy().to_string()));
            }
        }
    }
    
    Ok(latest_files)
}

/// Capture a screenshot from the browser context
pub async fn capture_screenshot(browser_context: &crate::browser::BrowserContext) 
    -> Result<Option<Vec<u8>>, UtilsError> 
{
    // Get the current page from browser context
    let page = browser_context.get_current_page().await
        .map_err(|e| UtilsError::BrowserError(e.to_string()))?;
        
    // Take a screenshot
    let screenshot_data = page.screenshot(None).await
        .map_err(|e| UtilsError::BrowserError(e.to_string()))?;
        
    Ok(Some(screenshot_data))
}
