use agent_core::base::{ChatCompletions, Tool};
use agent_core::types::Message;
use serde_json::Value;

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
pub struct Agent {
    pub name: String,
    pub description: String,
    instructions: String,
    client: Box<dyn ChatCompletions>,
    tools: Vec<Box<dyn Tool>>,
}

pub struct AgentBuilder {
    name: String,
    description: String,
    instructions: String,
    client: Option<Box<dyn ChatCompletions>>,
    tools: Vec<Box<dyn Tool>>,
}

impl AgentBuilder {
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = instructions.into();
        self
    }

    pub fn client(mut self, client: impl ChatCompletions + 'static) -> Self {
        self.client = Some(Box::new(client));
        self
    }

    pub fn tool(mut self, tool: impl Tool + 'static) -> Self {
        self.tools.push(Box::new(tool));
        self
    }

    pub fn build(self) -> Agent {
        Agent {
            name: self.name,
            description: self.description,
            instructions: self.instructions,
            client: self
                .client
                .expect("Agent::builder() requires .client(...) before .build()"),
            tools: self.tools,
        }
    }
}

impl Agent {
    pub fn builder(name: impl Into<String>) -> AgentBuilder {
        AgentBuilder {
            name: name.into(),
            description: String::new(),
            instructions: String::new(),
            client: None,
            tools: Vec::new(),
        }
    }

    pub fn run(&mut self, task: impl Into<String>) -> String {
        // Step 1: Prepare messages for the chat completion request
        let mut messages = vec![
            Message::System {
                content: self.instructions.clone(),
            },
            Message::User {
                content: task.into(),
            },
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
        const MAX_ITERS: usize = 10;
        for _ in 0..MAX_ITERS {
            let response = self.client.create(&messages, &tool_schemas);

            match response {
                Message::Assistant {
                    content,
                    tool_calls: Some(calls),
                } => {
                    messages.push(Message::Assistant {
                        content: content.clone(),
                        tool_calls: Some(calls.clone()),
                    });
                    for call in &calls {
                        let tool = self
                            .tools
                            .iter()
                            .find(|t| t.name() == call.name)
                            .expect("unknown tool");
                        let result = tool.execute(&call.arguments);
                        messages.push(Message::Tool {
                            content: result,
                            tool_call_id: Some(call.id.clone()),
                        });
                    }
                    // fall through to next loop iteration
                }
                Message::Assistant {
                    content,
                    tool_calls: None,
                } => {
                    return content;
                }
                other => {
                    // model returned a non-assistant message — shouldn't happen
                    messages.push(other);
                    return "unexpected response role".to_string();
                }
            }
        }
        "max iterations exceeded".to_string()
    }

    // fn run_stream(&mut self, task: impl Into<String>) -> String {
    //     todo!()
    // }
}
