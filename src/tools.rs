use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::io::{self, Write};
use std::process::Command;
use tracing::warn;
use std::time::Duration;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn call(&self, arguments: &str) -> Result<String>;
}

pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Perform basic mathematical calculations. Supports addition, subtraction, multiplication, and division."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "The mathematical expression to evaluate, e.g., '2 + 2' or '10 / 2'."
                }
            },
            "required": ["expression"]
        })
    }

    async fn call(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let expression = args["expression"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'expression' argument"))?;

        // Simple evaluation logic for now. 
        // In a real app, we might use a crate like `meval` or `evalexpr`.
        // For this demo, let's keep it very simple or try to support basic arithmetic.
        
        let result = eval_arithmetic(expression)?;
        Ok(result.to_string())
    }
}

fn eval_arithmetic(expr: &str) -> Result<f64> {
    // Very naive parser for demonstration
    let expr = expr.replace(' ', "");
    
    if let Some(pos) = expr.find('+') {
        let left: f64 = expr[..pos].parse()?;
        let right: f64 = expr[pos + 1..].parse()?;
        return Ok(left + right);
    }
    if let Some(pos) = expr.find('-') {
        let left: f64 = expr[..pos].parse()?;
        let right: f64 = expr[pos + 1..].parse()?;
        return Ok(left - right);
    }
    if let Some(pos) = expr.find('*') {
        let left: f64 = expr[..pos].parse()?;
        let right: f64 = expr[pos + 1..].parse()?;
        return Ok(left * right);
    }
    if let Some(pos) = expr.find('/') {
        let left: f64 = expr[..pos].parse()?;
        let right: f64 = expr[pos + 1..].parse()?;
        if right == 0.0 {
            return Err(anyhow::anyhow!("Division by zero"));
        }
        return Ok(left / right);
    }

    expr.parse::<f64>().map_err(|e| anyhow::anyhow!("Failed to parse expression {}: {}", expr, e))
}

pub struct RunCommand;

#[async_trait]
impl Tool for RunCommand {
    fn name(&self) -> &str {
        "run_command"
    }

    fn description(&self) -> &str {
        "Execute a shell command on the host system. Useful for file system operations, running scripts, or gathering system information."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command line to execute (e.g., 'ls -la', 'cat file.txt')."
                }
            },
            "required": ["command"]
        })
    }

    async fn call(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let command_str = args["command"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' argument"))?;

        println!("\n[CAUTION] The agent wants to execute: '{}'.", command_str);
        print!("Allow this command? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            return Ok("User denied permission to execute command.".to_string());
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(command_str)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(format!("Output:\n{}", stdout))
        } else {
            Ok(format!("Command failed with error:\n{}\nStdout:\n{}", stderr, stdout))
        }
    }
}

pub struct WebSearchTool;

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information using the Brave Search API. Returns a summary of relevant web pages."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to look up."
                }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, arguments: &str) -> Result<String> {
        let args: Value = serde_json::from_str(arguments)?;
        let query = args["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' argument"))?;

        let api_key = std::env::var("BRAVE_API_KEY")
            .map_err(|_| anyhow::anyhow!("BRAVE_API_KEY environment variable not set"))?;

        let client = reqwest::Client::new();
        let url = format!("https://api.search.brave.com/res/v1/web/search?q={}", urlencoding::encode(query));

        let mut retries = 0;
        let max_retries = 3;
        let mut backoff = 1;

        loop {
            let response = client
                .get(&url)
                .header("X-Subscription-Token", &api_key)
                .send()
                .await;

            match response {
                Ok(res) if res.status().is_success() => {
                    let json: Value = res.json().await?;
                    
                    let mut results_summary = String::new();
                    if let Some(web_results) = json["web"]["results"].as_array() {
                        for (i, result) in web_results.iter().take(5).enumerate() {
                            let title = result["title"].as_str().unwrap_or("No Title");
                            let description = result["description"].as_str().unwrap_or("No Description");
                            let url = result["url"].as_str().unwrap_or("No URL");
                            
                            results_summary.push_str(&format!("{}. {}\n   {}\n   URL: {}\n\n", i + 1, title, description, url));
                        }
                    }

                    if results_summary.is_empty() {
                        return Ok("No results found.".to_string());
                    } else {
                        return Ok(format!("Search results for '{}':\n\n{}", query, results_summary));
                    }
                }
                Ok(res) => {
                    warn!("Brave Search API error: {}. Retrying...", res.status());
                }
                Err(e) => {
                    warn!("Brave Search Network error: {}. Retrying...", e);
                }
            }

            if retries >= max_retries {
                return Err(anyhow::anyhow!("Brave Search failed after retries"));
            }

            tokio::time::sleep(Duration::from_secs(backoff)).await;
            retries += 1;
            backoff *= 2;
        }
    }
}
