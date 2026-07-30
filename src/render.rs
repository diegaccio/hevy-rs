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
        OutputFormat::Text => text(user),
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

fn text(value: &Value) {
    if let Some(items) = value.get("items").and_then(Value::as_array) {
        println!(
            "Page: {} of {}",
            value.get("page").and_then(Value::as_u64).unwrap_or(0),
            value.get("page_count").and_then(Value::as_u64).unwrap_or(0)
        );
        if value.get("all") == Some(&Value::Bool(true)) {
            let pages = value
                .get("pages_fetched")
                .and_then(Value::as_array)
                .map(|pages| {
                    pages
                        .iter()
                        .filter_map(Value::as_u64)
                        .map(|page| page.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            println!("Complete retrieval requested; pages fetched: {pages}");
        }
        for item in items {
            let title = item
                .get("title")
                .or_else(|| item.get("id"))
                .and_then(Value::as_str)
                .unwrap_or("(unnamed)");
            let id = item.get("id").and_then(Value::as_str);
            match id {
                Some(id) if id != title => println!("- {title} ({id})"),
                _ => println!("- {title}"),
            }
        }
    } else if value.get("name").is_some() {
        user_text(value);
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(value).expect("JSON values serialize")
        );
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
