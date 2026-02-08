use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};
use std::pin::Pin;
use std::sync::Arc;

use crate::models::{InputEvent, OutputEvent, Message, ToolCall, FunctionCall};
use crate::tools::Tool;
use tracing::{error, warn};
use std::time::Duration;

pub struct Agent {
    api_key: String,
    model: String,
    tools: Arc<Vec<Box<dyn Tool>>>,
    history: Vec<Message>,
    client: Client,
}

impl Agent {
    pub fn new(api_key: String, model: String, tools: Vec<Box<dyn Tool>>) -> Self {
        Self {
            api_key,
            model,
            tools: Arc::new(tools),
            history: Vec::new(),
            client: Client::new(),
        }
    }

    pub async fn run(&self, event: InputEvent) -> anyhow::Result<Pin<Box<dyn Stream<Item = OutputEvent> + Send>>> {
        let mut history = self.history.clone();
        
        match event {
            InputEvent::UserInputEvent(text) => {
                history.push(Message {
                    role: "user".to_string(),
                    content: Some(text),
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
        }

        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let tools = self.tools.clone();
        
        let client = self.client.clone();
        
        // This is a bit complex in Rust due to async stream lifetimes.
        // We'll use async-stream or just a manual stream.
        let stream = async_stream::stream! {
            let mut current_history = history;
            
            loop {
                let mut body = json!({
                    "model": model,
                    "messages": current_history,
                    "stream": true,
                });

                if !tools.is_empty() {
                    let tools_json: Vec<Value> = tools.iter().map(|t| {
                        json!({
                            "type": "function",
                            "function": {
                                "name": t.name(),
                                "description": t.description(),
                                "parameters": t.input_schema()
                            }
                        })
                    }).collect();
                    body["tools"] = json!(tools_json);
                }

                let mut retries = 0;
                let max_retries = 3;
                let mut backoff = 1;

                let res = loop {
                    let response = client.post("https://openrouter.ai/api/v1/chat/completions")
                        .header("Authorization", format!("Bearer {}", api_key))
                        .json(&body)
                        .send()
                        .await;

                    match response {
                        Ok(r) if r.status().is_success() => break Some(r),
                        Ok(r) => {
                            let status = r.status();
                            // Try to read body for error details without consuming it if we need to retry? 
                            // Actually reqwest response content is stream, so we can't read it easily and then use it again unless we clone?
                            // But here we are failing, so we don't need the success stream.
                            let error_text = r.text().await.unwrap_or_default();
                            warn!("API Error check: Status: {}, Body: {}", status, error_text);
                            yield OutputEvent::Error(format!("API Error: {}. Retrying...", status));
                        }
                        Err(e) => {
                            warn!("Network Request Error: {}", e);
                            yield OutputEvent::Error(format!("Network Error: {}. Retrying...", e));
                        }
                    }

                    if retries >= max_retries {
                        error!("Max retries reached for LLM API call.");
                        yield OutputEvent::Error("Max retries reached. Stopping.".to_string());
                        break None;
                    }

                    tokio::time::sleep(Duration::from_secs(backoff)).await;
                    retries += 1;
                    backoff *= 2;
                };

                let res = match res {
                    Some(r) => r,
                    None => break,
                };

                let mut stream = res.bytes_stream();
                let mut full_content = String::new();
                let mut tool_calls_accum: Vec<ToolCallAccum> = Vec::new();

                while let Some(chunk_result) = stream.next().await {
                    let chunk = match chunk_result {
                        Ok(c) => c,
                        Err(_) => break,
                    };
                    
                    let text = String::from_utf8_lossy(&chunk);
                    for line in text.lines() {
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                break;
                            }
                            
                            if let Ok(val) = serde_json::from_str::<Value>(data) {
                                if let Some(delta) = val["choices"][0]["delta"].as_object() {
                                    if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                        full_content.push_str(content);
                                        yield OutputEvent::OutputText(content.to_string());
                                    }
                                    
                                    if let Some(tool_calls) = delta.get("tool_calls").and_then(|tc| tc.as_array()) {
                                        for tc in tool_calls {
                                            let index = tc["index"].as_u64().unwrap_or(0) as usize;
                                            if index >= tool_calls_accum.len() {
                                                tool_calls_accum.push(ToolCallAccum::default());
                                            }
                                            
                                            if let Some(id) = tc["id"].as_str() {
                                                tool_calls_accum[index].id = Some(id.to_string());
                                            }
                                            if let Some(name) = tc["function"]["name"].as_str() {
                                                tool_calls_accum[index].name.push_str(name);
                                            }
                                            if let Some(args) = tc["function"]["arguments"].as_str() {
                                                tool_calls_accum[index].arguments.push_str(args);
                                                yield OutputEvent::OutputToolCallDelta(args.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // If no tool calls, we are done
                if tool_calls_accum.is_empty() {
                    break;
                }

                // Append assistant message with tool calls to history
                let assistant_tool_calls: Vec<ToolCall> = tool_calls_accum.iter().map(|tc| ToolCall {
                    id: tc.id.clone().unwrap_or_default(),
                    r#type: "function".to_string(),
                    function: FunctionCall {
                        name: tc.name.clone(),
                        arguments: tc.arguments.clone(),
                    },
                }).collect();

                current_history.push(Message {
                    role: "assistant".to_string(),
                    content: if full_content.is_empty() { None } else { Some(full_content.clone()) },
                    tool_calls: Some(assistant_tool_calls.clone()),
                    tool_call_id: None,
                });

                // Execute tool calls and add results to history
                for tc in assistant_tool_calls {
                    yield OutputEvent::OutputToolCall {
                        id: tc.id.clone(),
                        name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
                    };

                    // Find the tool
                    let tool = tools.iter().find(|t| t.name() == tc.function.name);
                    
                    let result = if let Some(t) = tool {
                        match t.call(&tc.function.arguments).await {
                            Ok(res) => res,
                            Err(e) => {
                                let err_msg = format!("Error executing tool: {}", e);
                                error!("{}", err_msg);
                                yield OutputEvent::Error(err_msg.clone());
                                err_msg
                            }
                        }
                    } else {
                        let err_msg = format!("Unknown tool: {}", tc.function.name);
                        error!("{}", err_msg);
                        yield OutputEvent::Error(err_msg.clone());
                        err_msg
                    };

                    current_history.push(Message {
                        role: "tool".to_string(),
                        content: Some(result),
                        tool_calls: None,
                        tool_call_id: Some(tc.id),
                    });
                }
                
                // Loop continues to call LLM again with tool results
            }
        };

        Ok(Box::pin(stream))
    }
}

#[derive(Default)]
struct ToolCallAccum {
    id: Option<String>,
    name: String,
    arguments: String,
}
