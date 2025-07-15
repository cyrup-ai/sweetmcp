use anyhow::Result;
use clap::Args;
use colorful::Colorful;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Input};
use llm_client::{
    backend_builders::{anthropic::AnthropicBackendBuilder, openai::OpenAiBackendBuilder},
    LlmClient,
};
use llm_interface::{
    llms::LlmBackend,
    requests::{CompletionRequest, CompletionResponse, RequestConfig, RequestConfigBuilder},
};
use llm_prompt::LlmPrompt;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

use crate::config::Config;

#[derive(Args, Debug)]
pub struct ChatArgs {
    /// Initial prompt (if not provided, enters interactive mode)
    prompt: Option<String>,
    
    /// Provider to use [openai, anthropic, mistral_rs, llama_cpp, generic]
    #[arg(short, long)]
    provider: Option<String>,
    
    /// Model to use (provider-specific)
    #[arg(short, long)]
    model: Option<String>,
    
    /// Enable interactive chat loop
    #[arg(short, long)]
    r#loop: bool,
    
    /// System prompt
    #[arg(short, long)]
    system: Option<String>,
    
    /// Temperature (0.0-2.0)
    #[arg(short, long)]
    temperature: Option<f32>,
    
    /// Maximum tokens to generate
    #[arg(long)]
    max_tokens: Option<usize>,
    
    /// Top-p sampling (0.0-1.0)
    #[arg(long)]
    top_p: Option<f32>,
    
    /// Frequency penalty (-2.0-2.0)
    #[arg(long)]
    frequency_penalty: Option<f32>,
    
    /// Presence penalty (-2.0-2.0)
    #[arg(long)]
    presence_penalty: Option<f32>,
    
    /// Stop sequences (can be used multiple times)
    #[arg(long)]
    stop: Vec<String>,
    
    /// Save chat history to file
    #[arg(long)]
    save_history: Option<PathBuf>,
    
    /// Load chat history from file
    #[arg(long)]
    load_history: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatHistory {
    messages: Vec<ChatMessage>,
    provider: String,
    model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

pub async fn execute(args: ChatArgs, config: Config) -> Result<()> {
    let term = Term::stdout();
    
    // Determine provider and model
    let provider = args.provider
        .or(config.default_provider.clone())
        .ok_or_else(|| anyhow::anyhow!("No provider specified and no default set"))?;
    
    let model = args.model
        .or_else(|| config.default_models.get(&provider).cloned())
        .ok_or_else(|| anyhow::anyhow!("No model specified and no default for provider {}", provider))?;
    
    // Build prompt
    let mut prompt_builder = match provider.as_str() {
        "anthropic" => PromptBuilder::new_anthropic(),
        "openai" => PromptBuilder::new_openai(),
        _ => PromptBuilder::new_generic_openai(),
    };
    
    // Add system prompt if provided
    if let Some(system) = args.system.or(config.chat.system_prompt) {
        prompt_builder.add_system_message(&system);
    }
    
    // Load history if requested
    if let Some(history_path) = args.load_history {
        let history = load_history(&history_path).await?;
        for msg in history.messages {
            match msg.role.as_str() {
                "system" => prompt_builder.add_system_message(&msg.content),
                "user" => prompt_builder.add_user_message(&msg.content),
                "assistant" => prompt_builder.add_assistant_message(&msg.content),
                _ => {}
            }
        }
    }
    
    // Create backend
    let backend = create_backend(&provider, &model, &args, &config).await?;
    
    // Configure request
    let mut request_config = RequestConfig::new(backend.model_ctx_size(), backend.inference_ctx_size());
    
    // Apply configuration
    request_config.temperature = args.temperature.unwrap_or(config.defaults.temperature);
    if let Some(max_tokens) = args.max_tokens.or(config.defaults.max_tokens) {
        request_config.requested_response_tokens = Some(max_tokens);
    }
    if let Some(top_p) = args.top_p.or(config.defaults.top_p) {
        request_config.top_p = Some(top_p);
    }
    if let Some(freq_penalty) = args.frequency_penalty.or(config.defaults.frequency_penalty) {
        request_config.frequency_penalty = Some(freq_penalty);
    }
    if let Some(pres_penalty) = args.presence_penalty.or(config.defaults.presence_penalty) {
        request_config.presence_penalty = pres_penalty;
    }
    
    
    // Create client
    let client = LlmClient::new(backend);
    
    // Handle single prompt or interactive mode
    if let Some(prompt) = args.prompt {
        // Single prompt mode
        prompt_builder.add_user_message(&prompt);
        let response = process_completion(&client, prompt_builder.build(), request_config, &config).await?;
        
        if let Some(save_path) = args.save_history {
            let history = ChatHistory {
                messages: vec![
                    ChatMessage { role: "user".to_string(), content: prompt },
                    ChatMessage { role: "assistant".to_string(), content: response },
                ],
                provider: provider.clone(),
                model: model.clone(),
            };
            save_history(&save_path, &history).await?;
        }
    } else if args.r#loop {
        // Interactive loop mode
        interactive_chat_loop(client, prompt_builder, request_config, &config, &provider, &model, args.save_history).await?;
    } else {
        // Single interactive prompt
        let prompt = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your prompt")
            .interact_text()?;
        
        prompt_builder.add_user_message(&prompt);
        process_completion(&client, prompt_builder.build(), request_config, stream, &config).await?;
    }
    
    Ok(())
}

async fn interactive_chat_loop(
    client: LlmClient,
    mut prompt_builder: PromptBuilder,
    request_config: RequestConfig,
    config: &Config,
    provider: &str,
    model: &str,
    save_path: Option<PathBuf>,
) -> Result<()> {
    let term = Term::stdout();
    let mut messages = Vec::new();
    
    println!("{}", "Interactive Chat Mode".color(colorful::Color::Cyan).bold());
    println!("Provider: {} | Model: {}", provider.green(), model.green());
    println!("Commands: /exit, /clear, /save <file>, /load <file>, /help");
    println!("{}", "─".repeat(60).color(colorful::Color::DarkGray));
    println!();
    
    loop {
        let input = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("You")
            .allow_empty(true)
            .interact_text()?;
        
        if input.trim().is_empty() {
            continue;
        }
        
        // Handle commands
        if input.starts_with('/') {
            match handle_command(&input, &mut prompt_builder, &mut messages).await {
                Ok(CommandResult::Continue) => continue,
                Ok(CommandResult::Exit) => break,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red(), e);
                    continue;
                }
            }
        }
        
        // Add user message
        prompt_builder.add_user_message(&input);
        messages.push(ChatMessage { role: "user".to_string(), content: input.clone() });
        
        // Get response
        let response = process_completion(&client, prompt_builder.build(), request_config.clone(), config).await?;
        
        // Add assistant message
        prompt_builder.add_assistant_message(&response);
        messages.push(ChatMessage { role: "assistant".to_string(), content: response });
        
        println!();
    }
    
    // Save history if requested
    if let Some(path) = save_path {
        let history = ChatHistory {
            messages,
            provider: provider.to_string(),
            model: model.to_string(),
        };
        save_history(&path, &history).await?;
        println!("{} Chat history saved to: {}", "✓".green(), path.display());
    }
    
    Ok(())
}

async fn process_completion(
    client: &LlmClient,
    prompt: Prompt,
    config: RequestConfig,
    app_config: &Config,
) -> Result<String> {
    let start = std::time::Instant::now();
    
    print!("{}: ", "Assistant".color(colorful::Color::Blue).bold());
    std::io::stdout().flush()?;
    
    let mut request = CompletionRequest::new(client.backend.clone());
    request.prompt = prompt;
    request.config = config;
    
    // Stream the response token by token
    let response = request.request().await?;
    
    println!("{}", response.content);
    
    // Show metadata if configured
    if app_config.chat.show_timing {
        let elapsed = start.elapsed();
        println!("\n{} Generated in {:.2}s", "⏱".color(colorful::Color::DarkGray), elapsed.as_secs_f32());
    }
    
    if app_config.chat.show_token_usage {
        if let Some(usage) = response.token_usage.total_tokens {
            println!("{} Tokens: {} prompt + {} completion = {} total",
                "🔢".color(colorful::Color::DarkGray),
                response.token_usage.prompt_tokens,
                response.token_usage.completion_tokens,
                usage
            );
        }
    }
    
    Ok(response.content)
}

async fn create_backend(
    provider: &str,
    model: &str,
    args: &ChatArgs,
    config: &Config,
) -> Result<Arc<LlmBackend>> {
    match provider {
        "openai" => {
            let mut builder = OpenAiBackendBuilder::default();
            // API key will be loaded from OPENAI_API_KEY env var automatically
            
            use llm_models::OpenAiModelTrait;
            builder.set_model(model)?;
            let client = builder.init()?;
            Ok(client.backend)
        }
        "anthropic" => {
            let mut builder = AnthropicBackendBuilder::default();
            // API key will be loaded from ANTHROPIC_API_KEY env var automatically
            
            use llm_models::AnthropicModelTrait;
            builder.set_model(model)?;
            let client = builder.init()?;
            Ok(client.backend)
        }
        // Add other providers...
        _ => anyhow::bail!("Provider {} not yet implemented", provider),
    }
}

#[derive(Debug)]
enum CommandResult {
    Continue,
    Exit,
}

async fn handle_command(
    command: &str,
    prompt_builder: &mut PromptBuilder,
    messages: &mut Vec<ChatMessage>,
) -> Result<CommandResult> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    match parts.get(0).map(|s| *s) {
        Some("/exit") | Some("/quit") => Ok(CommandResult::Exit),
        Some("/clear") => {
            prompt_builder.clear();
            messages.clear();
            println!("{} Chat history cleared", "✓".green());
            Ok(CommandResult::Continue)
        }
        Some("/help") => {
            println!("{}", "Available commands:".yellow());
            println!("  /exit, /quit - Exit chat mode");
            println!("  /clear - Clear chat history");
            println!("  /save <file> - Save chat history");
            println!("  /load <file> - Load chat history");
            println!("  /help - Show this help");
            Ok(CommandResult::Continue)
        }
        Some("/save") => {
            if let Some(path) = parts.get(1) {
                // Save current messages
                let history = ChatHistory {
                    messages: messages.clone(),
                    provider: "unknown".to_string(), // Would need to pass these through
                    model: "unknown".to_string(),
                };
                save_history(path.into(), &history).await?;
                println!("{} Chat saved to: {}", "✓".green(), path);
            } else {
                println!("{} Usage: /save <filename>", "Error:".red());
            }
            Ok(CommandResult::Continue)
        }
        Some("/load") => {
            if let Some(path) = parts.get(1) {
                let history = load_history(&PathBuf::from(path)).await?;
                prompt_builder.clear();
                messages.clear();
                
                for msg in history.messages {
                    match msg.role.as_str() {
                        "user" => prompt_builder.add_user_message(&msg.content),
                        "assistant" => prompt_builder.add_assistant_message(&msg.content),
                        _ => {}
                    }
                    messages.push(msg);
                }
                println!("{} Loaded {} messages", "✓".green(), messages.len());
            } else {
                println!("{} Usage: /load <filename>", "Error:".red());
            }
            Ok(CommandResult::Continue)
        }
        _ => {
            println!("{} Unknown command. Type /help for available commands.", "Error:".red());
            Ok(CommandResult::Continue)
        }
    }
}

async fn save_history(path: &PathBuf, history: &ChatHistory) -> Result<()> {
    let json = serde_json::to_string_pretty(history)?;
    fs::write(path, json).await?;
    Ok(())
}

async fn load_history(path: &PathBuf) -> Result<ChatHistory> {
    let json = fs::read_to_string(path).await?;
    let history = serde_json::from_str(&json)?;
    Ok(history)
}