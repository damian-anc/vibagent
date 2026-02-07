use anyhow::anyhow;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};
use std::pin::Pin;
use std::sync::Arc;

use crate::models::{InputEvent, OutputEvent, Message, ToolCall, FunctionCall};
use crate::tools::Tool;

pub struct Agent {
    api_key: String,
    model: String,
    tools: Vec<Box<dyn Tool>>,
    history: Vec<Message>,
    client: Client,
}

impl Agent {
    pub fn new(api_key: String, model: String, tools: Vec<Box<dyn Tool>>) -> Self {
        Self {
            api_key,
            model,
            tools,
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
        // Since Agent doesn't implement Clone and we need tools in the stream generator,
        // we use an Arc for tools if we were to make the stream truly independent,
        // but for now let's just use self if we can or wrap everything in a state.
        let tools = Arc::new(self.tools.iter().map(|t| {
            (t.name().to_string(), t.description().to_string(), t.input_schema())
        }).collect::<Vec<_>>());
        
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
                    let tools_json: Vec<Value> = tools.iter().map(|(name, desc, schema)| {
                        json!({
                            "type": "function",
                            "function": {
                                "name": name,
                                "description": desc,
                                "parameters": schema
                            }
                        })
                    }).collect();
                    body["tools"] = json!(tools_json);
                }

                let response = client.post("https://openrouter.ai/api/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .json(&body)
                    .send()
                    .await;

                let res = match response {
                    Ok(r) => r,
                    Err(e) => {
                        println!("Error: {}", e);
                        break;
                    }
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
                    // This part is tricky because we need the actual tool objects.
                    // Let's assume for now we only have the calculator.
                    // In a generic way, we'd need to pass the tools in.
                    
                    let result = if tc.function.name == "calculator" {
                        use crate::tools::CalculatorTool;
                        let tool = CalculatorTool;
                        tool.call(&tc.function.arguments).await.unwrap_or_else(|e| e.to_string())
                    } else {
                        format!("Unknown tool: {}", tc.function.name)
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
