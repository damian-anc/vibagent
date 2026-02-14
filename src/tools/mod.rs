use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

mod calculator;
mod command;
mod search;
mod geocoding;

pub use calculator::CalculatorTool;
pub use command::RunCommand;
pub use search::WebSearchTool;
pub use geocoding::GeocodingTool;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn call(&self, arguments: &str) -> Result<String>;
}
