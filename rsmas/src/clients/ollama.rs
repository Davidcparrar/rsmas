use serde::{Deserialize, Serialize};

use agent_core::ChatCompletions;
use agent_core::types::{Message, Role, ToolCall};
use serde_json::Value;

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<Value>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
enum OllamaRole {
    System,
    User,
    Assistant,
    Tool,
}

impl From<Role> for OllamaRole {
    fn from(r: Role) -> Self {
        match r {
            Role::System => OllamaRole::System,
            Role::User => OllamaRole::User,
            Role::Assistant => OllamaRole::Assistant,
            Role::Tool => OllamaRole::Tool,
        }
    }
}

impl From<OllamaRole> for Role {
    fn from(r: OllamaRole) -> Self {
        match r {
            OllamaRole::System => Role::System,
            OllamaRole::User => Role::User,
            OllamaRole::Assistant => Role::Assistant,
            OllamaRole::Tool => Role::Tool,
        }
    }
}
#[derive(Serialize, Deserialize, Clone)]
struct OllamaMessage {
    role: OllamaRole,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OllamaToolCall>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct OllamaToolCall {
    function: OllamaToolCallFunction,
}

#[derive(Serialize, Deserialize, Clone)]
struct OllamaToolCallFunction {
    name: String,
    arguments: Value,
}

#[derive(Deserialize)]
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
        let ollama_messages: Vec<OllamaMessage> = messages
            .iter()
            .map(|m| OllamaMessage {
                role: m.role.clone().into(),
                content: m.content.clone(),
                tool_calls: m.tool_calls.as_ref().map(|tcs| {
                    tcs.iter()
                        .map(|tc| OllamaToolCall {
                            function: OllamaToolCallFunction {
                                name: tc.name.clone(),
                                arguments: tc.arguments.clone(),
                            },
                        })
                        .collect()
                }),
            })
            .collect();

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

        let response: OllamaResponse =
            serde_json::from_str(&response_str).expect("Failed to parse Ollama response");

        // Step 3: Convert response to Message format
        let OllamaResponse { message } = response;
        let OllamaMessage {
            role,
            content,
            tool_calls,
        } = message;

        let tool_calls = tool_calls.map(|tcs| {
            tcs.into_iter()
                .map(|tc| ToolCall {
                    name: tc.function.name,
                    arguments: tc.function.arguments,
                })
                .collect()
        });

        Message {
            role: role.into(),
            content: content,
            tool_calls,
        }
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
