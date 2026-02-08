use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InputEvent {
    UserInputEvent(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OutputEvent {
    OutputText(String),
    OutputToolCall {
        id: String,
        name: String,
        arguments: String,
    },
    OutputToolCallDelta(String),
    Error(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>, // For tool result messages
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}
