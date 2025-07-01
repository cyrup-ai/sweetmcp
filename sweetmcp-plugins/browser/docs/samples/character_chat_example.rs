use kalosm::language::*;

#[tokio::main]
async fn main() {
    // Get character details from user
    let description = prompt_input("What is your character like? ").unwrap();
    let character_name = prompt_input("What is your character's name? ").unwrap();
    let character_description =
        format!("{description} The assistant will never reveal that it is an AI or assistant.");

    // Initialize model
    let model = Llama::new_chat().await.unwrap();
    
    // Create constraints that parses the character response format then stops 
    // on the end of the assistant's response
    let constraints = LiteralParser::new(format!("(Responding as {}) ", character_name))
        .then(model.default_assistant_constraints());
    
    // Create a chat session with the model and the character description
    let mut chat = model.chat().with_system_prompt(character_description);

    // Chat with the user in an interactive loop
    loop {
        // Get user input and apply constraints to the response
        let mut output_stream =
            chat(&prompt_input("\n> ").unwrap()).with_constraints(constraints.clone());
        
        // Print the character response
        print!("Bot: ");
        output_stream.to_std_out().await.unwrap();
    }
}

// Source: https://github.com/floneum/floneum/blob/main/interfaces/kalosm/examples/chat-with-character.rs
