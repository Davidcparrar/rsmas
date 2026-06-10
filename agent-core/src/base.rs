// Traits
use crate::Message;
use serde_json::Value;

pub trait ChatCompletions {
    fn create(&self, messages: &[Message], tools: &[Value]) -> Message;

    // fn create_stream(&self, messages: &[&str], tools: Option<&[Box<dyn Tool>]>) -> String;
}

pub trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    fn execute(&self, args: &Value) -> String;
}
