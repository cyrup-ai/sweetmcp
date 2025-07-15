use crate::requests::*;
use crate::llms::local::mistral_rs::tool_use::{convert_tools, to_mistral_tool_choice};
use mistralrs::{
    Constraint, DrySamplingParams, NormalRequest, Request as MistralCompletionRequest,
    RequestMessage, Response, SamplingParams,
};

pub fn new(
    request: &CompletionRequest,
    tx: tokio::sync::mpsc::Sender<Response>,
    id: usize,
) -> crate::Result<MistralCompletionRequest, CompletionError> {
    let sampling_params = SamplingParams {
        temperature: Some(request.config.temperature.into()),
        frequency_penalty: request.config.frequency_penalty,
        presence_penalty: Some(request.config.presence_penalty),
        max_len: request.config.actual_request_tokens.map(|val| val as usize),
        top_k: None,
        top_p: request.config.top_p.map(|val| val as f64),
        min_p: None,
        top_n_logprobs: 0,
        stop_toks: None,
        logits_bias: None,
        n_choices: 1,
        dry_params: Some(DrySamplingParams::default()),
    };

    let constraint = Constraint::None;

    let mistral_request = MistralCompletionRequest::Normal(NormalRequest {
        messages: RequestMessage::Completion {
            text: request
                .prompt
                .get_built_prompt_string()
                .map_err(|e| CompletionError::RequestBuilderError(e.to_string()))?,
            echo_prompt: false,
            best_of: 1,
        },
        // messages: RequestMessage::CompletionTokens(
        //     request
        //         .prompt
        //         .get_built_prompt_as_tokens()
        //         .map_err(|e| CompletionError::RequestBuilderError(e.to_string()))?,
        // ),
        sampling_params,
        response: tx,
        return_logprobs: false,
        is_streaming: false,
        id,
        constraint,
        suffix: None,
        adapters: None,
        // Convert tools if present in the request
        tool_choice: req.tools.as_ref().and_then(|t| t.choice.as_ref().map(to_mistral_tool_choice)),
        tools: req.tools.as_ref().map(|t| {
            if !t.definitions.is_empty() {
                Some(convert_tools(&t.definitions))
            } else {
                None
            }
        }).flatten(),
        logits_processors: None,
    });
    Ok(mistral_request)
}
