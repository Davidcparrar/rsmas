use serde_json::Value;
use std::fmt;

#[derive(Clone, Debug)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::Tool => write!(f, "tool"),
        }
    }
}

impl From<&str> for Role {
    fn from(s: &str) -> Self {
        match s {
            "system" => Role::System,
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "tool" => Role::Tool,
            other => panic!("unknown role: {other}"),
        }
    }
}

#[derive(Clone)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

#[derive(Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl Message {
    pub fn new(role: Role, content: String, tool_calls: Option<Vec<ToolCall>>) -> Self {
        Self {
            role,
            content,
            tool_calls,
        }
    }
}
