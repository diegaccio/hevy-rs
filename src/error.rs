use reqwest::{StatusCode, blocking::Response};

#[derive(Debug)]
pub struct AppError {
    pub exit_code: i32,
    pub code: &'static str,
    pub message: String,
    pub status: Option<u16>,
    pub request_id: Option<String>,
    pub retry_after_seconds: Option<u64>,
}

impl AppError {
    pub fn invocation(message: impl Into<String>) -> Self {
        Self::new(2, "invocation", message)
    }

    pub fn authentication(message: impl Into<String>) -> Self {
        Self::new(3, "authentication", message)
    }

    pub fn authentication_response(
        message: impl Into<String>,
        status: StatusCode,
        request_id: Option<String>,
    ) -> Self {
        Self {
            status: Some(status.as_u16()),
            request_id,
            ..Self::authentication(message)
        }
    }

    pub fn api(message: impl Into<String>, status: StatusCode, request_id: Option<String>) -> Self {
        Self {
            status: Some(status.as_u16()),
            request_id,
            ..Self::new(4, "api", message)
        }
    }

    pub fn api_message(message: impl Into<String>) -> Self {
        Self::new(4, "api", message)
    }

    pub fn transport(message: impl Into<String>) -> Self {
        Self::new(5, "transport", message)
    }

    pub fn transport_response(message: impl Into<String>, response: &Response) -> Self {
        Self {
            status: Some(response.status().as_u16()),
            request_id: request_id(response),
            retry_after_seconds: retry_after_seconds(response),
            ..Self::new(5, "transport", message)
        }
    }

    fn new(exit_code: i32, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            exit_code,
            code,
            message: message.into(),
            status: None,
            request_id: None,
            retry_after_seconds: None,
        }
    }
}

pub fn request_id(response: &Response) -> Option<String> {
    response
        .headers()
        .get("x-request-id")
        .or_else(|| response.headers().get("request-id"))
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

pub fn retry_after_seconds(response: &Response) -> Option<u64> {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|seconds| *seconds <= 60)
}
