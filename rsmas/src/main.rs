use schemars::JsonSchema;
use serde::Deserialize;

fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

#[derive(Deserialize, JsonSchema)]
struct TempParams {
    celsius: f64,
}

fn main() {
    let result = celsius_to_fahrenheit(22.0);
    println!("{}", result);
    let t = TempParams { celsius: 22.0 };
    println!("{}", t.celsius)
}
