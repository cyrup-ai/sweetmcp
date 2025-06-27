use kalosm::language::*;

#[tokio::main]
async fn main() {
    // Initialize Phi-3 model
    let model = Llama::phi_3().await.unwrap();
    
    // Create chat session with system prompt
    let mut chat = model
        .chat()
        .with_system_prompt("You will act as a helpful AI assistant.");

    // Interactive chat loop
    loop {
        chat(&prompt_input("\n> ").unwrap())
            .to_std_out() // Stream tokens to standard output
            .await
            .unwrap();
    }
}

// Source: https://github.com/floneum/floneum/blob/main/interfaces/kalosm/examples/phi-3.rs
// Updated to use system prompt from original example
