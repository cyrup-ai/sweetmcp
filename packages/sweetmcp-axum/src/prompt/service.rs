use extism::convert::Json;
use futures::StreamExt;
use minijinja::Environment;
use rpc_router::{HandlerError, HandlerResult};
use serde_json::json;
use tokio::sync::{mpsc, oneshot};

use crate::plugin::PluginManager; // Updated path
use crate::types::{
    GetPromptRequest, ListPromptsRequest, Prompt, PromptMessage, PromptMessageContent, PromptResult,
};

pub fn prompts_list_stream(plugin_manager: PluginManager) -> crate::prompt::model::PromptStream {
    let (tx, rx) = mpsc::channel(16);

    tokio::spawn(async move {
        // Lock-free iteration over DashMap
        for entry in plugin_manager.prompt_info.iter() {
            let (_, (_, metadata)) = (entry.key(), entry.value());
            // Explicitly type the Ok variant if needed, though it should infer correctly.
            // This error might be a symptom of something else if this doesn't fix it.
            let result: HandlerResult<Prompt> = Ok(metadata.clone());
            if tx.send(result).await.is_err() {
                break;
            }
        }
    });

    crate::prompt::model::PromptStream::new(rx)
}

// Future-based prompts_get
pub fn prompts_get_pending(
    request: GetPromptRequest,
    plugin_manager: PluginManager,
) -> crate::prompt::model::PendingPromptResult {
    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        let prompt_id = request.id.clone();

        let (plugin_name, prompt_metadata) = {
            // Lock-free search in DashMap
            let found = plugin_manager.prompt_info.iter().find(|entry| {
                let (_, (_, prompt)) = (entry.key(), entry.value());
                prompt.id == prompt_id
            });
            match found {
                Some(entry) => {
                    let (_, (name, metadata)) = (entry.key(), entry.value());
                    (name.clone(), metadata.clone())
                }
                None => {
                    let _ = tx.send(Err(HandlerError::new(format!(
                        "Prompt '{}' not found",
                        prompt_id
                    ))));
                    return;
                }
            }
        };

        let template_content: String = {
            // Lock-free access to plugin using DashMap
            let mut plugin_entry = match plugin_manager.plugins.get_mut(&plugin_name) {
                Some(entry) => entry,
                None => {
                    let _ = tx.send(Err(HandlerError::new(format!(
                        "Internal error: Plugin '{}' not found",
                        plugin_name
                    ))));
                    return;
                }
            };

            match plugin_entry.call::<Json<serde_json::Value>, String>(
                "mcp_get_prompt_template",
                Json(json!({ "id": prompt_id })),
            ) {
                Ok(template) => template,
                Err(_) => {
                    let _ = tx.send(Err(HandlerError::new(format!(
                        "Plugin '{}' failed to provide template for prompt '{}'",
                        plugin_name, prompt_id
                    ))));
                    return;
                }
            }
        };

        let mut env = Environment::new();
        if let Err(_) = env.add_template(&prompt_id, &template_content) {
            let _ = tx.send(Err(HandlerError::new(format!(
                "Failed to load prompt template '{}'",
                prompt_id
            ))));
            return;
        }

        let tmpl = match env.get_template(&prompt_id) {
            Ok(t) => t,
            Err(e) => {
                let _ = tx.send(Err(HandlerError::new(format!(
                    "Failed to get prompt template '{}': {}",
                    prompt_id, e
                ))));
                return;
            }
        };

        // For this example, we do not support arguments (as PromptArgument is Option)
        // You may want to extend this to support arguments if needed.

        let rendered_text = match tmpl.render(minijinja::context!()) {
            Ok(text) => text,
            Err(_) => {
                let _ = tx.send(Err(HandlerError::new(format!(
                    "Failed to render prompt template '{}'",
                    prompt_id
                ))));
                return;
            }
        };

        let prompt = Prompt {
            id: prompt_metadata.id.clone(),
            name: prompt_metadata.name.clone(),
            description: prompt_metadata.description.clone(),
            arguments: prompt_metadata.arguments.clone(),
            messages: Some(vec![PromptMessage {
                role: "user".to_string(),
                content: PromptMessageContent {
                    type_: "text".to_string(),
                    text: Some(rendered_text),
                    data: None,
                    mime_type: None,
                },
            }]),
        };

        let response = PromptResult { prompt };

        let _ = tx.send(Ok(response));
    });

    crate::prompt::model::PendingPromptResult { rx }
}

/// Router-compatible async handler for prompts/list
pub async fn prompts_list_handler(
    plugin_manager: PluginManager,
    request: Option<ListPromptsRequest>,
) -> HandlerResult<Vec<Prompt>> {
    // Use PromptService instead of calling functions directly
    let service = PromptService::new(plugin_manager);
    let stream = service.list(request.unwrap_or(ListPromptsRequest { filter: None }));

    // Collect results from stream
    let mut prompts = Vec::new();
    let mut stream = std::pin::Pin::new(Box::new(stream));

    // Use StreamExt::next for clarity
    while let Some(result) = StreamExt::next(&mut stream).await {
        match result {
            Ok(prompt) => prompts.push(prompt),
            Err(e) => return Err(e),
        }
    }

    Ok(prompts)
}

/// Router-compatible async handler for prompts/get
pub async fn prompts_get_handler(
    plugin_manager: PluginManager,
    request: GetPromptRequest,
) -> HandlerResult<PromptResult> {
    // Use PromptService instead of calling functions directly
    let service = PromptService::new(plugin_manager);
    let pending = service.get(request);

    // Await the result
    pending.await
}

// Restore PromptService struct and impl
#[derive(Clone)]
pub struct PromptService {
    plugin_manager: PluginManager,
}

impl PromptService {
    pub fn new(plugin_manager: PluginManager) -> Self {
        Self { plugin_manager }
    }

    // Changed to return PromptStream
    pub fn list(&self, _req: ListPromptsRequest) -> crate::prompt::model::PromptStream {
        // Delegate to the stream-based function
        prompts_list_stream(self.plugin_manager.clone())
    }

    // Changed to return PendingPromptResult
    pub fn get(&self, req: GetPromptRequest) -> crate::prompt::model::PendingPromptResult {
        // Delegate to the future-based function
        prompts_get_pending(req, self.plugin_manager.clone())
    }
}
