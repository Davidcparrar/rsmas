pub mod clients;

use agent_core::base::{ChatCompletions, Tool};
use agent_core::types::{Message, Role, ToolCall};
use clients::OllamaClient;
use serde_json::{Value, json};

// Agent {
//
//     name = "assistant",
//     description = "You are a helpful assistant that converts Celsius to Fahrenheit."
//     instructions = "You are helpful, use tools when appropiate"
//     model_client = OllamaChatCompletionsClient(
//         model = "gemma:2b",
//         base_url = "localhost",
//     )
//      tools = [get_weather]
//  }
//
struct Agent {
    name: String,
    description: String,
    instructions: String,
    client: Box<dyn ChatCompletions>,
    tools: Vec<Box<dyn Tool>>,
}

impl Agent {
    pub fn new(
        name: String,
        description: String,
        instructions: String,
        client: Box<dyn ChatCompletions>,
    ) -> Self {
        Self {
            name,
            description,
            instructions,
            client,
            tools: Vec::new(),
        }
    }
    fn add_tool(mut self, tool: impl Tool + 'static) -> Self {
        self.tools.push(Box::new(tool));
        self
    }

    fn run(&mut self, task: impl Into<String>) -> String {
        // Step 1: Prepare messages for the chat completion request
        let mut messages = vec![
            Message::new(Role::System, self.instructions.clone(), None),
            Message::new(Role::User, task.into(), None),
        ];
        //Step 1b: Create tool schemas:
        let tool_schemas: Vec<Value> = self
            .tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name(),
                        "description": t.description(),
                        "parameters": t.parameters_schema(),
                    }
                })
            })
            .collect();
        // Step 2: Call the chat completion API
        // // sketch — not paste-ready
        loop {
            let response = self.client.create(&messages, &tool_schemas);
            if let Some(calls) = &response.tool_calls {
                messages.push(response.clone()); // assistant turn with tool_calls
                for call in calls {
                    let tool = self
                        .tools
                        .iter()
                        .find(|t| t.name() == call.name)
                        .expect("unknown tool");
                    let result = tool.execute(&call.arguments);
                    messages.push(Message::new(Role::Tool, result, None));
                }
                continue;
            }
            messages.push(response.clone());
            // Step 3: Return the response
            return response.content;
        }
    }

    // fn run_stream(&mut self, task: impl Into<String>) -> String {
    //     todo!()
    // }
}

struct GetWeather;

struct GetTime;

impl Tool for GetWeather {
    fn name(&self) -> &str {
        "get_weather"
    }
    fn description(&self) -> &str {
        "Get current weather for a city"
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
        format!("Weather in {city}: 22°C, sunny")
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
    let mut agent = Agent::new(
        "Awesome rust agent - Chucho".to_string(),
        "Conquer the world with wit".to_string(),
        "You are a helpful assistant. When you receive tool results, \
             use them to answer the user's question directly and concisely. \
             Do not describe the tool call itself."
            .to_string(),
        Box::new(OllamaClient {
            model: "llama3.1:8b".to_string(),
            base_url: "http://localhost:11434".to_string(),
        }),
    )
    .add_tool(GetWeather)
    .add_tool(GetTime);

    println!(
        "The {} is an agent\nthat will {}\ngiven that it does {}\nwith {} tools",
        agent.name,
        agent.description,
        agent.instructions,
        agent.tools.len(),
    );

    let result = agent.run("What is the weather in Paris?");
    println!("Result: {}", result);
}
