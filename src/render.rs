use crate::error::AppError;
use clap::ValueEnum;
use serde_json::{Value, json};

#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

pub fn success(user: &Value, format: OutputFormat) {
    match format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string(user).expect("JSON values serialize")
        ),
        OutputFormat::Text => user_text(user),
    }
}

pub fn error(error: &AppError, format: OutputFormat) {
    match format {
        OutputFormat::Text => eprintln!("{}", error.message),
        OutputFormat::Json => {
            let mut output = json!({ "code": error.code, "message": error.message });
            let object = output.as_object_mut().expect("JSON object");
            if let Some(status) = error.status {
                object.insert("status".to_owned(), json!(status));
            }
            if let Some(request_id) = &error.request_id {
                object.insert("request_id".to_owned(), json!(request_id));
            }
            if let Some(retry_after_seconds) = error.retry_after_seconds {
                object.insert("retry_after_seconds".to_owned(), json!(retry_after_seconds));
            }
            eprintln!(
                "{}",
                serde_json::to_string(&output).expect("JSON values serialize")
            );
        }
    }
}

fn user_text(user: &Value) {
    let field = |name| {
        user.get(name)
            .and_then(Value::as_str)
            .unwrap_or("(not provided)")
    };
    println!("ID: {}", field("id"));
    println!("Name: {}", field("name"));
    if user.get("url").and_then(Value::as_str).is_some() {
        println!("URL: {}", field("url"));
    }
    if user.get("email").and_then(Value::as_str).is_some() {
        println!("Email: {}", field("email"));
    }
}
