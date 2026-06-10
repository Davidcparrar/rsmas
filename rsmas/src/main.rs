pub mod agents;
pub mod clients;
use crate::agents::Agent;
use crate::clients::OllamaClient;
use serde_json::{Value, json};

use agent_core::base::Tool;
struct GetWeather;

struct GetTime;

impl Tool for GetWeather {
    fn name(&self) -> &str {
        "get_weather"
    }
    fn description(&self) -> &str {
        "Get the weather for a single city. To get weather for multiple cities, call this tool once per city in parallel."
    }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "city": { "type": "string", "description": "City name" }
            },
            "required": ["city"]
        })
    }
    fn execute(&self, args: &Value) -> String {
        let city = args
            .get("city")
            .and_then(Value::as_str)
            .unwrap_or("unknown");

        serde_json::json!({
            "city": city,
            "temperature_c": 22,
            "condition": "sunny"
        })
        .to_string()
    }
}

impl Tool for GetTime {
    fn name(&self) -> &str {
        "get_time"
    }
    fn description(&self) -> &str {
        "Get the current time"
    }
    fn parameters_schema(&self) -> Value {
        json!({ "type": "object", "properties": {} })
    }
    fn execute(&self, _args: &Value) -> String {
        "Time is 15:48".to_string()
    }
}
// Usage
fn main() {
    let mut agent = Agent::builder("Awesome rust agent - Chucho")
        .description("Conquer the world with wit")
        .instructions(
            "You are a helpful assistant. When you receive tool results, \
                 use them to answer the user's question directly and concisely. \
                 Do not describe the tool call itself.",
        )
        .client(OllamaClient {
            model: "llama3.1:8b".to_string(),
            base_url: "http://localhost:11434".to_string(),
        })
        .tool(GetWeather)
        .tool(GetTime)
        .build();

    let result = agent.run("What is the weather in Paris and Tokyo?");
    println!("Result: {}", result);
}
