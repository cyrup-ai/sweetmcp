use futures_util::StreamExt;
use llm_client::*;

/// A basic streaming request example. Shows how to use streaming with any backend.
#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    // Create a client. We'll use an Anthropic model
    // Streaming is supported on all backends
    let llm_client = LlmClient::anthropic().claude_3_haiku().init().unwrap();

    // Or use another backend
    // let llm_client = LlmClient::llama_cpp()
    //     .mistral_7b_instruct_v0_3()
    //     .init()
    //     .await
    //     .unwrap();

    // Text generation with streaming
    let mut streaming_completion = llm_client.basic_completion();

    // Enable streaming - this is the key difference from basic_completion
    streaming_completion.stream_response(true);

    streaming_completion
        .prompt()
        .add_system_message()
        .unwrap()
        .set_content("You're a country robot.");
    streaming_completion
        .prompt()
        .add_user_message()
        .unwrap()
        .set_content("Tell me about the stars in the night sky over Texas. Be poetic but keep it under 200 words.");

    // Instead of getting the full response at once, we stream it
    println!("Streaming response:");

    // Get a stream of response chunks
    let mut stream = streaming_completion.stream().await.unwrap();

    // Process each chunk as it arrives
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                // Print text deltas as they arrive
                if let Some(text) = chunk.text_delta {
                    print!("{}", text);
                    // Flush stdout to see the text immediately
                    std::io::Write::flush(&mut std::io::stdout()).unwrap();
                }

                // Check if this is the final chunk with finish reason
                if let Some(finish_reason) = chunk.finish_reason {
                    println!("\n\nStream finished with reason: {:?}", finish_reason);
                }
            }
            Err(e) => {
                eprintln!("Error in stream: {:?}", e);
                break;
            }
        }
    }

    // You can reuse the streaming completion with different settings
    streaming_completion.temperature(1.0);
    streaming_completion
        .prompt()
        .add_user_message()
        .unwrap()
        .set_content("Now tell me about the stars over Montana!");

    println!("\n=== Second streaming response: ===\n");

    // Stream the second response
    let mut stream = streaming_completion.stream().await.unwrap();
    while let Some(chunk_result) = stream.next().await {
        if let Ok(chunk) = chunk_result {
            if let Some(text) = chunk.text_delta {
                print!("{}", text);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
    }
    println!("\n\nDone!");
}
