use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};
use tracing::warn;
use std::time::Duration;
use super::Tool;

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
