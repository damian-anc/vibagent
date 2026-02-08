mod models;
mod tools;
mod agent;

use anyhow::Result;
use dotenv::dotenv;
use std::env;
use crate::models::InputEvent;
use crate::tools::CalculatorTool;
use crate::agent::Agent;
use futures::{StreamExt, Stream};
use std::pin::Pin;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    let api_key = env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY must be set in environment or .env file");
    
    let model = "arcee-ai/trinity-large-preview:free"; // Or any other model
    
    let agent = Agent::new(api_key, model.to_string(), vec![
        Box::new(CalculatorTool),
        Box::new(crate::tools::RunCommand),
        Box::new(crate::tools::WebSearchTool),
    ]);
    
    println!("Agent initialized. Type something (e.g., 'what is 234 + 567?')");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let mut stream: Pin<Box<dyn Stream<Item = crate::models::OutputEvent> + Send>> = agent.run(InputEvent::UserInputEvent(input.trim().to_string())).await?;
    
    while let Some(event) = stream.next().await {
        match event {
            crate::models::OutputEvent::OutputText(text) => print!("{}", text),
            crate::models::OutputEvent::OutputToolCall { name, arguments, .. } => {
                println!("\n[Tool Call: {} with {}]", name, arguments);
            },
            crate::models::OutputEvent::OutputToolCallDelta(_delta) => {
                // For now just ignore deltas or print dots
                print!(".");
            }
        }
        // Flush stdout to see streaming
        use std::io::Write;
        std::io::stdout().flush()?;
    }
    
    println!("\nDone.");
    Ok(())
}
