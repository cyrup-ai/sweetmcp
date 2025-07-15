pub mod chat;
pub mod completion;

use anyhow::Result;
use colorful::Colorful;
use llm_models::api_models::*;

pub async fn list_models(provider_filter: Option<String>) -> Result<()> {
    println!("{}", "Available Models:".color(colorful::Color::Cyan).bold());
    println!();
    
    // API Models
    println!("{}", "API Models:".color(colorful::Color::Yellow));
    for provider in ApiProviders::all() {
        if let Some(ref filter) = provider_filter {
            if !provider.as_str().to_lowercase().contains(&filter.to_lowercase()) {
                continue;
            }
        }
        
        println!("  {} {}", "►".green(), provider.as_str().bold());
        let models = provider.models();
        for model in models {
            println!("    • {} - {}", model.id().green(), model.description());
        }
    }
    
    // TODO: List available local models based on downloaded files
    
    Ok(())
}