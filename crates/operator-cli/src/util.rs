use serde::Serialize;

/// CLI result type — accepts any std error.
pub type CliResult = Result<(), Box<dyn std::error::Error>>;

/// Output JSON to stdout.
pub fn output_json<T: Serialize>(value: &T) {
    println!("{}", serde_json::to_string_pretty(value).unwrap());
}

/// Output human-readable (pretty) format.
pub fn output_pretty<T: Serialize>(value: &T) {
    let json = serde_json::to_value(value).unwrap();
    format_json_pretty(&json, 0);
    println!();
}

fn format_json_pretty(value: &serde_json::Value, indent: usize) {
    let prefix = "  ".repeat(indent);
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                match val {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        println!("{}{}:", prefix, key);
                        format_json_pretty(val, indent + 1);
                    }
                    _ => {
                        println!("{}{}: {}", prefix, key, format_scalar(val));
                    }
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                print!("{}[{}]: ", prefix, i);
                match val {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        println!();
                        format_json_pretty(val, indent + 1);
                    }
                    _ => {
                        println!("{}", format_scalar(val));
                    }
                }
            }
        }
        _ => {
            println!("{}{}", prefix, format_scalar(value));
        }
    }
}

fn format_scalar(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}
