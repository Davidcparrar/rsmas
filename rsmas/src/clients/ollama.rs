use serde::{Deserialize, Serialize};

use agent_core::ChatCompletions;
use agent_core::types::{Message, ToolCall};
use serde_json::Value;

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
enum OllamaMessage {
    System {
        content: String,
    },
    User {
        content: String,
    },
    Assistant {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<OllamaToolCall>>,
    },
    Tool {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_call_id: Option<String>,
    },
}

impl From<Message> for OllamaMessage {
    fn from(m: Message) -> Self {
        match m {
            Message::System { content } => OllamaMessage::System { content },
            Message::User { content } => OllamaMessage::User { content },
            Message::Assistant {
                content,
                tool_calls,
            } => OllamaMessage::Assistant {
                content,
                tool_calls: tool_calls.map(|tcs| tcs.into_iter().map(Into::into).collect()),
            },
            Message::Tool {
                content,
                tool_call_id,
            } => OllamaMessage::Tool {
                content,
                tool_call_id,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct OllamaToolCall {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    id: String,
    function: OllamaToolCallFunction,
}

impl From<ToolCall> for OllamaToolCall {
    fn from(tc: ToolCall) -> Self {
        OllamaToolCall {
            id: tc.id,
            function: OllamaToolCallFunction {
                name: tc.name,
                arguments: tc.arguments,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct OllamaToolCallFunction {
    name: String,
    arguments: Value,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
struct OllamaResponse {
    message: OllamaMessage,
}

pub struct OllamaClient {
    pub model: String,
    pub base_url: String,
}

impl OllamaClient {
    pub fn new(model: &str, base_url: &str) -> Self {
        OllamaClient {
            model: model.to_string(),
            base_url: base_url.to_string(),
        }
    }
}

impl ChatCompletions for OllamaClient {
    fn create(&self, messages: &[Message], tools: &[Value]) -> Message {
        // Step 1: Convert types to provider format
        let ollama_messages: Vec<OllamaMessage> =
            messages.iter().cloned().map(Into::into).collect();

        // Step 2: Make API call
        let request_body = OllamaRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            stream: false,
            tools: tools.to_vec(),
        };

        let url = format!("{}/api/chat", self.base_url);

        println!("\n=== REQUEST → {} ===", url);
        println!(
            "{}",
            serde_json::to_string_pretty(&request_body).unwrap_or_default()
        );

        let response_str = ureq::post(&url)
            .send_json(&request_body)
            .expect("Failed to reach Ollama")
            .body_mut()
            .read_to_string()
            .expect("Failed to read Ollama response");

        println!("\n=== RESPONSE ===");
        let pretty = serde_json::from_str::<Value>(&response_str)
            .map(|v| serde_json::to_string_pretty(&v).unwrap_or(response_str.clone()))
            .unwrap_or(response_str.clone());
        println!("{}", pretty);
        println!("================\n");

        // Step 3: Convert response to Message format
        let response: OllamaResponse =
            serde_json::from_str(&response_str).expect("Failed to parse Ollama response");

        let OllamaResponse { message } = response;

        let canonical = match message {
            OllamaMessage::System { content } => Message::System { content },
            OllamaMessage::User { content } => Message::User { content },
            OllamaMessage::Assistant {
                content,
                tool_calls,
            } => {
                let tool_calls = tool_calls.map(|tcs| {
                    tcs.into_iter()
                        .map(|tc| {
                            let OllamaToolCall { id, function } = tc;
                            let OllamaToolCallFunction { name, arguments } = function;
                            ToolCall {
                                id,
                                name,
                                arguments,
                            }
                        })
                        .collect()
                });
                Message::Assistant {
                    content,
                    tool_calls,
                }
            }
            OllamaMessage::Tool {
                content,
                tool_call_id,
            } => Message::Tool {
                content,
                tool_call_id,
            },
        };

        canonical
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sets_model_and_base_url() {
        let client = OllamaClient::new("qwen2.5:1.5b", "http://localhost:11434");
        assert_eq!(client.model, "qwen2.5:1.5b");
        assert_eq!(client.base_url, "http://localhost:11434");
    }
}
