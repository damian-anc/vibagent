use vibagent::tools::{StationLookupTool, Tool};
use tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tool = StationLookupTool::new("data/ghcnd_stations.db");
    
    let args = r#"{"lat": 37.7749, "lon": -122.4194}"#;
    println!("Calling station_lookup with args: {}", args);
    
    match tool.call(args).await {
        Ok(result) => {
            println!("Result:\n{}", result);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
    
    Ok(())
}
