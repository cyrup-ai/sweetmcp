// Internal imports
use super::error::map_serialization_error;
use super::{
    config::ApiConfigTrait,
    error::{map_deserialization_error, ClientError, WrappedError},
};
use crate::llms::api::anthropic::completion::res::{parse_sse_event, AnthropicStreamEvent}; // Specific to Anthropic parsing
use bytes::Bytes;
use futures_util::{Stream, StreamExt, TryStreamExt};
use serde::{de::DeserializeOwned, Serialize};
use tokio_util::codec::{FramedRead, LinesCodec};
use tokio_util::io::StreamReader; // Use StreamReader for converting stream to AsyncRead // For SSE line splitting

#[derive(Debug, Clone)]
pub(crate) struct ApiClient<C: ApiConfigTrait> {
    http_client: reqwest::Client,
    pub config: C,
    pub backoff: backoff::ExponentialBackoff,
}

impl<C: ApiConfigTrait> ApiClient<C> {
    pub fn new(config: C) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            config,
            backoff: backoff::ExponentialBackoffBuilder::new()
                .with_max_elapsed_time(Some(std::time::Duration::from_secs(60)))
                .build(),
        }
    }

    /// Make a POST request to {path} and deserialize the response body
    pub(crate) async fn post<I, O>(&self, path: &str, request: I) -> Result<O, ClientError>
    where
        I: Serialize + std::fmt::Debug,
        O: DeserializeOwned,
    {
        // Log the serialized request
        let request_maker = || async {
            let serialized_request =
                serde_json::to_string(&request).map_err(map_serialization_error)?;
            crate::trace!("Serialized post request: {}", serialized_request);
            let request_builder = self
                .http_client
                .post(self.config.url(path))
                // .query(&self.config.query())
                .headers(self.config.headers())
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(serialized_request);
            // crate::trace!("Serialized post request: {:?}", request_builder); // This will log API keys!
            Ok(request_builder.build()?)
        };
        self.execute(request_maker).await
    }

    /// Make a GET request to {path} and deserialize the response body
    pub(crate) async fn get<O>(&self, path: &str) -> Result<O, ClientError>
    where
        O: DeserializeOwned,
    {
        let request_maker = || async {
            crate::trace!("Get request: {}", path);
            let request_builder = self
                .http_client
                .get(self.config.url(path))
                .headers(self.config.headers());

            // crate::trace!("Serialized post request: {:?}", request_builder); // This will log API keys!
            Ok(request_builder.build()?)
        };
        self.execute(request_maker).await
    }

    /// Execute a HTTP request and retry on rate limit
    ///
    /// request_maker serves one purpose: to be able to create request again
    /// to retry API call after getting rate limited. request_maker is async because
    /// reqwest::multipart::Form is created by async calls to read files for uploads.
    async fn execute_raw<M, Fut>(&self, request_maker: M) -> Result<Bytes, ClientError>
    where
        M: Fn() -> Fut,
        Fut: core::future::Future<Output = Result<reqwest::Request, ClientError>>,
    {
        let client = self.http_client.clone();

        backoff::future::retry(self.backoff.clone(), || async {
            let request = request_maker().await.map_err(backoff::Error::Permanent)?;
            let response = client
                .execute(request)
                .await
                .map_err(ClientError::Reqwest)
                .map_err(backoff::Error::Permanent)?;

            let status = response.status();
            let bytes = response
                .bytes()
                .await
                .map_err(ClientError::Reqwest)
                .map_err(backoff::Error::Permanent)?;

            // Deserialize response body from either error object or actual response object
            if !status.is_success() {
                let wrapped_error: WrappedError = serde_json::from_slice(bytes.as_ref())
                    .map_err(|e| map_deserialization_error(e, bytes.as_ref()))
                    .map_err(backoff::Error::Permanent)?;

                if status.as_u16() == 429
                    // API returns 429 also when:
                    // "You exceeded your current quota, please check your plan and billing details."
                    && wrapped_error.error.r#type != Some("insufficient_quota".to_string())
                {
                    // Rate limited retry...
                    tracing::warn!("Rate limited: {}", wrapped_error.error.message);
                    return Err(backoff::Error::Transient {
                        err: ClientError::ApiError(wrapped_error.error),
                        retry_after: None,
                    });
                } else if status.as_u16() == 503 {
                    return Err(backoff::Error::Transient {
                        err: ClientError::ServiceUnavailable {
                            message: wrapped_error.error.message,
                        },
                        retry_after: None,
                    });
                } else {
                    return Err(backoff::Error::Permanent(ClientError::ApiError(
                        wrapped_error.error,
                    )));
                }
            }

            Ok(bytes)
        })
        .await
    }

    /// Execute a HTTP request and retry on rate limit
    ///
    /// request_maker serves one purpose: to be able to create request again
    /// to retry API call after getting rate limited. request_maker is async because
    /// reqwest::multipart::Form is created by async calls to read files for uploads.
    async fn execute<O, M, Fut>(&self, request_maker: M) -> Result<O, ClientError>
    where
        O: DeserializeOwned,
        M: Fn() -> Fut,
        Fut: core::future::Future<Output = Result<reqwest::Request, ClientError>>,
    {
        let bytes = self.execute_raw(request_maker).await?;

        // Deserialize once into a generic Value to allow logging before specific deserialization
        match serde_json::from_slice::<serde_json::Value>(&bytes) {
            Ok(value) => {
                // Log the pretty-printed JSON if possible
                if let Ok(pretty_json) = serde_json::to_string_pretty(&value) {
                    crate::trace!("Serialized response: {}", pretty_json);
                } else {
                    crate::trace!(
                        "Serialized response (raw): {}",
                        String::from_utf8_lossy(&bytes)
                    );
                }
                // Convert the Value into the target type
                serde_json::from_value(value).map_err(|e| map_deserialization_error(e, &bytes))
            }
            Err(e) => {
                // Log raw bytes if initial Value deserialization fails
                crate::error!(
                    "Failed to deserialize response into generic Value: {}. Raw response: {}",
                    e,
                    String::from_utf8_lossy(&bytes)
                );
                // Attempt direct deserialization into target type as fallback
                serde_json::from_slice(&bytes).map_err(|e_direct| {
                    // If direct also fails, report the direct error but log the original Value error too
                    crate::error!("Direct deserialization also failed: {}", e_direct);
                    map_deserialization_error(e_direct, &bytes)
                })
            }
        }
    }

    /// Make a POST request to {path} and stream/deserialize the SSE response body
    /// This is specific to Anthropic's event structure for now.
    pub(crate) async fn post_stream<I>(
        &self,
        path: &str,
        request: I,
    ) -> Result<impl Stream<Item = Result<AnthropicStreamEvent, ClientError>>, ClientError>
    where
        I: Serialize + std::fmt::Debug,
    {
        let request_maker = || async {
            let serialized_request =
                serde_json::to_string(&request).map_err(map_serialization_error)?;
            crate::trace!("Serialized post stream request: {}", serialized_request);
            let request_builder = self
                .http_client
                .post(self.config.url(path))
                .headers(self.config.headers())
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .header(reqwest::header::ACCEPT, "text/event-stream") // Expect SSE
                .body(serialized_request);
            Ok::<reqwest::Request, ClientError>(request_builder.build()?)
        };

        // Execute the request once to get the response stream
        // Retries are generally not handled at this stream level; they should be handled by the caller if needed.
        let request = request_maker().await?;
        let response = self
            .http_client
            .execute(request)
            .await
            .map_err(ClientError::Reqwest)?;

        let status = response.status();

        if !status.is_success() {
            // Read the full error body for non-successful streaming attempts
            let error_bytes = response.bytes().await.map_err(ClientError::Reqwest)?;
            let wrapped_error: WrappedError = serde_json::from_slice(error_bytes.as_ref())
                .map_err(|e| map_deserialization_error(e, error_bytes.as_ref()))?;
            // TODO: Handle potential rate limits (429) or service unavailable (503) specifically?
            // For now, return the API error directly. Consider adding retry logic here or in the caller.
            return Err(ClientError::ApiError(wrapped_error.error));
        }

        // Check if the content type is text/event-stream
        // Capture the content type before consuming the response
        let content_type_str = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap_or("invalid"))
            .unwrap_or("missing")
            .to_string(); // Clone to own the string
            
        let is_sse = content_type_str.starts_with("text/event-stream");
        
        if !is_sse {
            // Read body for better error message if content type is wrong
            let body_bytes = response.bytes().await.unwrap_or_default();
            let body_str = String::from_utf8_lossy(&body_bytes);
            return Err(ClientError::GenericError {
                message: format!(
                    "Expected Content-Type 'text/event-stream' for streaming response, but received '{}'. Body: {}",
                    content_type_str,
                    body_str
                ),
            });
        }

        Ok(Self::process_anthropic_sse_stream(response.bytes_stream()))
    }

    // Helper function to process the byte stream as SSE for Anthropic
    fn process_anthropic_sse_stream(
        byte_stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + Unpin + 'static,
    ) -> impl Stream<Item = Result<AnthropicStreamEvent, ClientError>> {
        // Convert the byte stream into an AsyncRead using StreamReader
        let stream_reader = StreamReader::new(
            byte_stream.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)),
        );

        // Use LinesCodec to split into lines, handling potential UTF-8 errors
        // Use concrete type instead of impl Trait
        let lines_stream = FramedRead::new(
            stream_reader, 
            LinesCodec::new_with_max_length(1024 * 1024) // 1MB max line length
        );

        // Use try_unfold to manage the state (accumulated lines for the current event)
        futures_util::stream::try_unfold(
            (lines_stream, Vec::<String>::new()), // State: (line_stream, current_event_lines)
            |(mut stream, mut current_lines)| async move {
                loop {
                    match stream.next().await {
                        Some(Ok(line)) => {
                            if line.is_empty() {
                                // Empty line signifies end of an event
                                if !current_lines.is_empty() {
                                    match parse_sse_event(&current_lines) {
                                        Ok(Some(event)) => {
                                            // Yield the parsed event and reset lines for next event
                                            let next_state = (stream, Vec::new());
                                            return Ok(Some((event, next_state)));
                                        }
                                        Ok(None) => {
                                            // Ping event might be handled here if parse_sse_event returns None for it
                                            // Or just ignore if parse_sse_event handles ping directly
                                            current_lines.clear(); // Reset for next event
                                        }
                                        Err(e) => {
                                            // Parsing error for the completed event
                                            tracing::error!("SSE Event Parse Error: {}", e);
                                            // Decide whether to yield error or continue. Let's yield.
                                            return Err(ClientError::StreamParseError(e));
                                        }
                                    }
                                }
                                // Reset lines even if they were empty (handles consecutive empty lines)
                                current_lines.clear();
                            } else {
                                // Accumulate lines for the current event
                                current_lines.push(line);
                            }
                        }
                        Some(Err(e)) => {
                            // Error reading lines (e.g., IO error, max length exceeded)
                            tracing::error!("SSE Line Read Error: {}", e);
                            return Err(ClientError::IoError(e.to_string()));
                        }
                        None => {
                            // End of the line stream
                            // Process any remaining lines if the stream ends without a final empty line
                            if !current_lines.is_empty() {
                                match parse_sse_event(&current_lines) {
                                    Ok(Some(event)) => {
                                        // Yield the final event and prepare to stop
                                        let next_state = (stream, Vec::new()); // Empty lines for termination state
                                        return Ok(Some((event, next_state)));
                                    }
                                    Ok(None) => { /* Ignore trailing ping or incomplete event */ }
                                    Err(e) => {
                                        // Error parsing the final lines
                                        tracing::error!("SSE Final Event Parse Error: {}", e);
                                        return Err(ClientError::StreamParseError(e));
                                    }
                                }
                            }
                            // Signal end of stream for unfold
                            return Ok(None);
                        }
                    }
                }
            },
        )
    }
}
