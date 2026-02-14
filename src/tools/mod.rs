use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

mod calculator;
mod command;
mod search;

pub use calculator::CalculatorTool;
pub use command::RunCommand;
pub use search::WebSearchTool;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn call(&self, arguments: &str) -> Result<String>;
}
