use anyhow::{Context, Result};
use clap::Args;
use colorful::Colorful;
use indicatif::{ProgressBar, ProgressStyle};
use llm_client::{
    backend_builders::{anthropic::AnthropicBackendBuilder, openai::OpenAiBackendBuilder},
    LlmClient,
};
use llm_interface::{
    llms::LlmBackend,
    requests::{CompletionRequest, CompletionResponse, RequestConfig, RequestConfigBuilder},
};
use llm_prompt::LlmPrompt;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

use crate::config::Config;

#[derive(Args, Debug)]
pub struct CompletionArgs {
    /// Prompt text (reads from stdin if not provided)
    prompt: Option<String>,
    
    /// Provider to use [openai, anthropic, mistral_rs, llama_cpp, generic]
    #[arg(short, long)]
    provider: Option<String>,
    
    /// Model to use (provider-specific)
    #[arg(short, long)]
    model: Option<String>,
    
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
    
    /// Grammar specification (for local models)
    #[arg(long)]
    grammar: Option<String>,
    
    /// Logit bias as JSON
    #[arg(long)]
    logit_bias: Option<String>,
    
    /// Output format [text, json, raw]
    #[arg(long, default_value = "text")]
    format: OutputFormat,
    
    /// Number of completions to generate
    #[arg(short = 'n', long, default_value = "1")]
    num_completions: usize,
    
    /// Cache prompt for faster subsequent calls (local models)
    #[arg(long)]
    cache_prompt: bool,
    
    /// Read prompt from file
    #[arg(short = 'i', long)]
    input: Option<PathBuf>,
    
    /// Write response to file
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    
    /// Quiet mode (only output completion)
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    Raw,
}

pub async fn execute(args: CompletionArgs, config: Config) -> Result<()> {
    // Get prompt
    let prompt_text = match (&args.prompt, &args.input) {
        (Some(p), _) => p.clone(),
        (None, Some(path)) => fs::read_to_string(path).await
            .with_context(|| format!("Failed to read input file: {}", path.display()))?,
        (None, None) => {
            if atty::is(atty::Stream::Stdin) {
                anyhow::bail!("No prompt provided. Use --prompt, --input, or pipe to stdin");
            }
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };
    
    if prompt_text.trim().is_empty() {
        anyhow::bail!("Empty prompt provided");
    }
    
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
    
    if let Some(system) = args.system {
        prompt_builder.add_system_message(&system);
    }
    
    prompt_builder.add_user_message(&prompt_text);
    
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
    
    // Cache prompt if requested
    if args.cache_prompt {
        request_config.cache_prompt = true;
    }
    
    // Parse logit bias if provided
    if let Some(logit_bias_json) = args.logit_bias {
        // Would parse and apply logit bias here
        // request_config.logit_bias(...);
    }
    
    
    // Create client
    let client = LlmClient::new(backend);
    
    // Generate completions
    let mut responses = Vec::new();
    
    for i in 0..args.num_completions {
        if !args.quiet && args.num_completions > 1 {
            eprintln!("{} Generating completion {}/{}...", 
                "►".green(), i + 1, args.num_completions);
        }
        
        let mut request = CompletionRequest::new(client.backend.clone());
        request.prompt = prompt_builder.build();
        request.config = request_config.clone();
        
        // TODO: Implement actual token-by-token streaming when API supports it
        let response = if !args.quiet {
            // Show progress indicator
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap()
            );
            spinner.set_message("Generating...");
            spinner.enable_steady_tick(std::time::Duration::from_millis(100));
            
            let result = request.request().await?;
            spinner.finish_and_clear();
            result
        } else {
            request.request().await?
        };
        
        responses.push(response);
    }
    
    // Format and output responses
    let output_content = format_output(&responses, &args)?;
    
    // Write output
    if let Some(output_path) = &args.output {
        fs::write(output_path, &output_content).await
            .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;
        if !args.quiet {
            eprintln!("{} Output written to: {}", "✓".green(), output_path.display());
        }
    } else {
        print!("{}", output_content);
        io::stdout().flush()?;
    }
    
    // Show stats if not quiet
    if !args.quiet && config.chat.show_token_usage {
        eprintln!();
        for (i, response) in responses.iter().enumerate() {
            if responses.len() > 1 {
                eprint!("Completion {}: ", i + 1);
            }
            eprintln!("{} {} prompt + {} completion = {} total tokens",
                "Tokens:".color(colorful::Color::DarkGray),
                response.token_usage.prompt_tokens,
                response.token_usage.completion_tokens,
                response.token_usage.total_tokens.unwrap_or(0)
            );
        }
    }
    
    Ok(())
}

fn format_output(responses: &[CompletionResponse], args: &CompletionArgs) -> Result<String> {
    match args.format {
        OutputFormat::Text => {
            if responses.len() == 1 {
                Ok(responses[0].content.clone())
            } else {
                let mut output = String::new();
                for (i, response) in responses.iter().enumerate() {
                    if i > 0 {
                        output.push_str("\n---\n");
                    }
                    output.push_str(&response.content);
                }
                Ok(output)
            }
        }
        OutputFormat::Json => {
            if responses.len() == 1 {
                Ok(serde_json::to_string_pretty(&serde_json::json!({
                    "content": responses[0].content,
                    "finish_reason": format!("{}", responses[0].finish_reason),
                    "model": responses[0].generation_settings.model,
                    "usage": {
                        "prompt_tokens": responses[0].token_usage.prompt_tokens,
                        "completion_tokens": responses[0].token_usage.completion_tokens,
                        "total_tokens": responses[0].token_usage.total_tokens,
                    }
                }))?)
            } else {
                let json_responses: Vec<_> = responses.iter().map(|r| {
                    serde_json::json!({
                        "content": r.content,
                        "finish_reason": format!("{}", r.finish_reason),
                        "model": r.generation_settings.model,
                        "usage": {
                            "prompt_tokens": r.token_usage.prompt_tokens,
                            "completion_tokens": r.token_usage.completion_tokens,
                            "total_tokens": r.token_usage.total_tokens,
                        }
                    })
                }).collect();
                Ok(serde_json::to_string_pretty(&json_responses)?)
            }
        }
        OutputFormat::Raw => {
            // Raw format includes all response data
            Ok(serde_json::to_string_pretty(&responses)?)
        }
    }
}

async fn create_backend(
    provider: &str,
    model: &str,
    args: &CompletionArgs,
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
        "llama_cpp" => {
            // Local llama.cpp backend
            // Would need model path configuration
            anyhow::bail!("llama_cpp provider not yet implemented")
        }
        "mistral_rs" => {
            // Local mistral.rs backend
            // Would need model path configuration
            anyhow::bail!("mistral_rs provider not yet implemented")
        }
        "generic" => {
            // Generic OpenAI-compatible backend
            let api_base = args.api_base.clone()
                .or_else(|| config.api_base_urls.get("generic").cloned())
                .ok_or_else(|| anyhow::anyhow!("No API base URL provided for generic provider"))?;
            
            // Would create generic backend with custom base URL
            anyhow::bail!("generic provider not yet implemented")
        }
        _ => anyhow::bail!("Unknown provider: {}", provider),
    }
}