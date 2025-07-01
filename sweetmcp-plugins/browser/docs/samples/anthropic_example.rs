use kalosm::language::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let llm = AnthropicCompatibleChatModel::builder()
        .with_claude_3_7_sonnet()
        .build();
    let prompt = "Write a 300 word essay about why the capital of France is Paris";
    print!("{}", prompt);

    let mut chat = llm.chat();
    chat(prompt).to_std_out().await.unwrap();
}
