// Internal imports
use super::*;
use crate::{
    llms::LlmBackend,
    requests::{logit_bias::LogitBias, stop_sequence::StopSequences, CompletionError},
};
use llm_prompt::LlmPrompt;

pub struct CompletionRequest {
    pub start_time: std::time::Instant,
    pub stop_sequences: StopSequences,
    pub grammar_string: Option<String>,
    pub logit_bias: Option<LogitBias>,
    pub prompt: LlmPrompt,
    pub config: CompletionRequestConfig, // Changed from RequestConfig
    pub backend: std::sync::Arc<LlmBackend>,
    pub llm_interface_errors: Vec<CompletionError>,
}

impl Clone for CompletionRequest {
    fn clone(&self) -> Self {
        Self {
            start_time: self.start_time,
            stop_sequences: self.stop_sequences.clone(),
            grammar_string: self.grammar_string.clone(),
            logit_bias: self.logit_bias.clone(),
            prompt: self.prompt.clone(),
            config: self.config.clone(),
            backend: std::sync::Arc::clone(&self.backend),
            llm_interface_errors: Vec::new(),
        }
    }
}

impl CompletionRequest {
    pub fn new(backend: std::sync::Arc<LlmBackend>) -> CompletionRequest {
        CompletionRequest {
            start_time: std::time::Instant::now(),
            stop_sequences: Default::default(),
            logit_bias: None,
            // Initialize CompletionRequestConfig instead of RequestConfig
            config: CompletionRequestConfig::default(),
            prompt: backend.new_prompt(),
            grammar_string: None,
            backend: std::sync::Arc::clone(&backend),
            llm_interface_errors: Vec::new(),
        }
    }

    pub fn reset_completion_request(&mut self) {
        self.prompt.reset_prompt();
        self.stop_sequences.sequences.clear();
        self.grammar_string = None;
        self.logit_bias = None;
    }

    pub async fn request(&mut self) -> crate::Result<CompletionResponse, CompletionError> {
        self.llm_interface_errors.clear();
        self.start_time = std::time::Instant::now();
        self.backend
            .build_logit_bias(&mut self.logit_bias)
            .map_err(|e| CompletionError::RequestBuilderError(e.to_string()))?;

        let total_prompt_tokens = self
            .backend
            .get_total_prompt_tokens(&self.prompt)
            .map_err(|e| CompletionError::RequestBuilderError(e.to_string()))?;

        self.config
            .set_max_tokens_for_request(total_prompt_tokens)
            .map_err(|e| CompletionError::RequestTokenLimitError(
                llm_prompt::RequestTokenLimitError::GenericPromptError { e }
            ))?;

        let mut retry_count: u8 = 0;

        loop {
            if retry_count >= self.config.retry_after_fail_n_times {
                let llm_interface_error = CompletionError::ExceededRetryCount {
                    message: format!("Request failed after {retry_count} attempts."),
                    errors: std::mem::take(&mut self.llm_interface_errors),
                };
                tracing::error!(?llm_interface_error);
                crate::error!("{}", llm_interface_error);
                return Err(llm_interface_error);
            }
            tracing::info!("{}", self);
            match self.backend.completion_request(self).await {
                Err(e) => {
                    tracing::warn!(?e);
                    retry_count += 1;
                    match e {
                        CompletionError::RequestBuilderError { .. }
                        | CompletionError::StopReasonUnsupported { .. }
                        | CompletionError::ClientError { .. } => {
                            return Err(e);
                        }

                        _ => (),
                    }
                    self.llm_interface_errors.push(e);
                    continue;
                }
                Ok(res) => {
                    tracing::info!("{}", res);
                    if self.stop_sequences.required {
                        if matches!(
                            res.finish_reason,
                            CompletionFinishReason::MatchingStoppingSequence(_)
                        ) {
                            return Ok(res);
                        } else {
                            let llm_interface_error = match res.finish_reason {
                                CompletionFinishReason::NonMatchingStoppingSequence(s) => {
                                    if let Some(s) = s {
                                        CompletionError::NonMatchingStopSequence(s.clone())
                                    } else {
                                        CompletionError::NoRequiredStopSequence
                                    }
                                }
                                _ => CompletionError::NoRequiredStopSequence,
                            };
                            tracing::warn!(?llm_interface_error);
                            self.llm_interface_errors.push(llm_interface_error);
                            retry_count += 1;
                            if self.config.increase_limit_on_fail {
                                self.config
                                    .increase_token_limit(total_prompt_tokens, None)?;
                            }
                            continue;
                        };
                    };
                    match res.finish_reason {
                        CompletionFinishReason::NonMatchingStoppingSequence(_)
                        | CompletionFinishReason::MatchingStoppingSequence(_) => return Ok(res),
                        CompletionFinishReason::StopLimit => {
                            if self.config.increase_limit_on_fail {
                                let llm_interface_error = CompletionError::StopLimitRetry;
                                tracing::warn!(?llm_interface_error);
                                self.llm_interface_errors.push(llm_interface_error);
                                self.config
                                    .increase_token_limit(total_prompt_tokens, None)?;
                                retry_count += 1;
                                continue;
                            }
                            return Ok(res);
                        }
                        CompletionFinishReason::Eos => return Ok(res),
                        CompletionFinishReason::ToolUse(_) => return Ok(res),
                        CompletionFinishReason::ToolCall(_) => return Ok(res),
                    }
                }
            };
        }
    }

    pub fn set_base_req_stop_sequences(
        &mut self,
        stop_word_done: &Option<String>,
        stop_word_no_result: &Option<String>,
    ) {
        if stop_word_done.is_some() || stop_word_no_result.is_some()
        // || step.stop_word_steps_done.is_some()
        {
            self.stop_sequences.required = true;
            self.stop_sequences.sequences.clear();
        }
        if let Some(stop_word_done) = &stop_word_done {
            self.stop_sequences.set_stop_word_done(stop_word_done);
        }

        if let Some(no_result_stop_word) = &stop_word_no_result {
            self.stop_sequences
                .set_stop_word_no_result(no_result_stop_word);
        }

        // if step.stop_word_steps_done {
        //     self.set_req_stop_word_steps_done();
        // }
    }
}

impl std::fmt::Display for CompletionRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f, "CompletionRequest:")?;

        writeln!(f, "  prompt: {}", self.prompt)?;
        writeln!(f, "  stop_sequences: {:?}", self.stop_sequences.to_vec())?;
        if let Some(logit_bias) = &self.logit_bias {
            writeln!(f, "  logit_bias: {}", logit_bias)?;
        }
        writeln!(f, "  grammar_string: {:?}", self.grammar_string)?;
        // Display fields from CompletionRequestConfig
        writeln!(f, "  config:")?;
        writeln!(
            f,
            "    max_request_tokens: {}",
            self.config.max_request_tokens
        )?;
        writeln!(
            f,
            "    actual_request_tokens: {:?}",
            self.config.actual_request_tokens
        )?;
        writeln!(f, "    temperature: {}", self.config.temperature)?;
        writeln!(f, "    top_p: {:?}", self.config.top_p)?;
        writeln!(f, "    top_k: {:?}", self.config.top_k)?;
        writeln!(
            f,
            "    frequency_penalty: {:?}",
            self.config.frequency_penalty
        )?;
        writeln!(
            f,
            "    presence_penalty: {:?}",
            self.config.presence_penalty
        )?;
        writeln!(f, "    n_choices: {:?}", self.config.n_choices)?;
        writeln!(f, "    stream_response: {}", self.config.stream_response)?;
        writeln!(f, "    tools: {:?}", self.config.tools)?;
        writeln!(f, "    tool_choice: {:?}", self.config.tool_choice)?;
        writeln!(
            f,
            "    retry_after_fail_n_times: {}",
            self.config.retry_after_fail_n_times
        )?;
        writeln!(
            f,
            "    increase_limit_on_fail: {}",
            self.config.increase_limit_on_fail
        )?;
        write!(f, "    cache_prompt: {}", self.config.cache_prompt)
    }
}
