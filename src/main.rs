mod models;
mod tools;
mod agent;
mod server;

use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    let cli = Cli::parse();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                let filter = if cli.verbose {
                    "vibagent=debug,tower_http=debug"
                } else {
                    "vibagent=info,tower_http=info"
                };
                filter.into()
            }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    if cli.verbose {
        tracing::debug!("Verbose logging enabled");
    }

    let api_key = env::var("OPENROUTER_API_KEY")
        .expect("OPENROUTER_API_KEY must be set in environment or .env file");
    
//    let default_model = "arcee-ai/trinity-large-preview:free";
    //let default_model = "z-ai/glm-4.5-air:free";
    let default_model = "qwen/qwen3-235b-a22b-thinking-2507";

    let model = env::var("AGENT_MODEL")
        .unwrap_or_else(|_| default_model.to_string());
    
    let app = server::app(api_key, model);
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    tracing::info!("Server running on http://localhost:3001");
    axum::serve(listener, app).await?;
    
    Ok(())
}
