use super::*;
use llm_models::gguf_presets::GgufPreset;

pub use crate::backends::TestBackendConfig;

const PROMPT: &str =
    "write a buzzfeed style listicle for the given input: Boy howdy, how ya'll doing? Actually make it a blog post, I'm feeling fancy today.";
const MAX_TOKENS: [usize; 4] = [100, 200, 400, 800];

pub struct SpeedBenchmark {
    pub prompt: String,
    pub max_tokens: Vec<usize>,
    pub models: Vec<GgufPreset>,
    pub backends: Vec<TestBackendConfig>,
    pub start_time: std::time::Instant,
    pub duration: std::time::Duration,
    pub backend_results: Vec<BackendResult>,
}

impl Default for SpeedBenchmark {
    fn default() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            duration: std::time::Duration::default(),
            models: Vec::new(),
            backends: Vec::new(),
            prompt: PROMPT.to_string(),
            max_tokens: MAX_TOKENS.to_vec(),
            backend_results: Vec::new(),
        }
    }
}

impl SpeedBenchmark {
    pub fn new() -> Self {
        Self::default()
    }

    pub(super) fn finalize(&mut self) {
        self.duration = self.start_time.elapsed();
        for result in self.backend_results.iter_mut() {
            result.finalize();
        }
    }
}

pub struct BackendResult {
    pub backend: String,
    start_time: std::time::Instant,
    pub duration: std::time::Duration,
    pub model_results: Vec<ModelResult>,
}

impl BackendResult {
    pub(super) fn new(backend: &TestBackendConfig) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            backend: backend.to_string(),
            duration: std::time::Duration::default(),
            model_results: Vec::new(),
        }
    }

    fn finalize(&mut self) {
        self.duration = self.start_time.elapsed();
        for result in self.model_results.iter_mut() {
            result.finalize();
        }
    }
}

pub struct ModelResult {
    pub model_id: String,
    start_time: std::time::Instant,
    pub duration: std::time::Duration,
    pub total_completion_tokens: usize,
    pub average_prompt_tokens_per_second: f32,
    pub average_completion_tokens_per_second: f32,
    pub runs: Vec<RunResult>,
}

impl ModelResult {
    pub(super) fn new(model_id: &str) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            model_id: model_id.to_string(),
            duration: std::time::Duration::default(),
            total_completion_tokens: 0,
            average_prompt_tokens_per_second: 0.0,
            average_completion_tokens_per_second: 0.0,
            runs: Vec::new(),
        }
    }

    fn finalize(&mut self) {
        self.duration = self.start_time.elapsed();
        self.total_completion_tokens = self.runs.iter().map(|r| r.response_tokens).sum();

        // Calculate average completion tokens per second, handling None
        let valid_gen_runs: Vec<f32> = self
            .runs
            .iter()
            .filter_map(|r| r.generation_tok_per_secs)
            .collect();
        self.average_completion_tokens_per_second = if !valid_gen_runs.is_empty() {
            valid_gen_runs.iter().sum::<f32>() / valid_gen_runs.len() as f32
        } else {
            0.0
        };

        // Calculate average prompt tokens per second, handling None
        let valid_prompt_runs: Vec<f32> = self
            .runs
            .iter()
            .filter_map(|r| r.prompt_tok_per_sec)
            .collect();
        self.average_prompt_tokens_per_second = if !valid_prompt_runs.is_empty() {
            valid_prompt_runs.iter().sum::<f32>() / valid_prompt_runs.len() as f32
        } else {
            0.0
        };
    }
}

pub struct RunResult {
    pub duration: std::time::Duration,
    pub requested_tokens: usize,
    pub response_tokens: usize,
    pub prompt_tok_per_sec: Option<f32>, // Changed to Option<f32>
    pub generation_tok_per_secs: Option<f32>, // Changed to Option<f32>
    pub response: CompletionResponse,
}

impl RunResult {
    pub(super) fn new(max_tok: usize, response: CompletionResponse) -> Self {
        // Calculate total_time based on start/end if not directly available
        let total_time = response
            .timing_usage
            .end_time
            .duration_since(response.timing_usage.start_time);
        Self {
            duration: total_time,
            requested_tokens: max_tok,
            response_tokens: response.token_usage.completion_tokens,
            prompt_tok_per_sec: response.timing_usage.prompt_tok_per_sec, // Store Option directly
            generation_tok_per_secs: response.timing_usage.generation_tok_per_sec, // Store Option directly
            response,
        }
    }
}

impl std::fmt::Display for SpeedBenchmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f, "SpeedBenchmark duration: {:?}", self.duration)?;
        // writeln!(f, " Prompt: '{}'", self.prompt)?;
        writeln!(f, " Runs at N max_tokens: {:?}", self.max_tokens)?;
        for result in self.backend_results.iter() {
            write!(f, "{}", result)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for BackendResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f, "  Backend: '{}'", self.backend)?;
        writeln!(f, "  BackendResult duration: {:?}", self.duration)?;
        for result in self.model_results.iter() {
            writeln!(f)?;
            write!(f, "{}", result)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for ModelResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "   ModelResult for model_id: {}", self.model_id)?;
        writeln!(f, "    duration: {:?}", self.duration)?;
        writeln!(
            f,
            "    total_completion_tokens: {}",
            self.total_completion_tokens
        )?;
        writeln!(
            f,
            "    average_prompt_tokens_per_second: {}",
            self.average_prompt_tokens_per_second
        )?;
        writeln!(
            f,
            "    average_completion_tokens_per_second: {}",
            self.average_completion_tokens_per_second
        )?;
        for (i, run) in self.runs.iter().enumerate() {
            writeln!(f)?;
            writeln!(f, "     run {}:", i + 1)?;
            write!(f, "{}", run)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for RunResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "       duration: {:?}", self.duration)?;
        writeln!(f, "       requested_tokens: {:?}", self.requested_tokens)?;
        writeln!(f, "       response_tokens: {:?}", self.response_tokens)?;
        writeln!(
            f,
            "       prompt_tok_per_sec: {:?}",
            self.prompt_tok_per_sec
        )?;
        writeln!(
            f,
            "       generation_tok_per_secs: {:?}",
            self.generation_tok_per_secs
                .map(|v| format!("{:.2}", v))
                .unwrap_or_else(|| "N/A".to_string())
        )
    }
}
