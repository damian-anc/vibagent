mod models;
mod tools;
mod agent;
mod server;

use anyhow::Result;
use dotenv::dotenv;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "vibagent=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let api_key = env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY must be set in environment or .env file");
    
    let model = env::var("AGENT_MODEL")
        .unwrap_or_else(|_| "arcee-ai/trinity-large-preview:free".to_string());
    
    let app = server::app(api_key, model);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://localhost:3000");
    axum::serve(listener, app).await?;
    
    Ok(())
}
