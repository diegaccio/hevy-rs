use crate::error::{AppError, request_id, retry_after_seconds};
use rand::Rng;
use reqwest::{
    StatusCode,
    blocking::{Client, Response},
};
use serde_json::Value;
use std::{env, thread, time::Duration};

const DEFAULT_API_BASE_URL: &str = "https://api.hevyapp.com";
const MAX_READ_RETRIES: u8 = 3;

pub fn get_user(api_key: &str) -> Result<Value, AppError> {
    let base_url =
        env::var("HEVY_API_BASE_URL").unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_owned());
    let url = format!("{}/v1/user/info", base_url.trim_end_matches('/'));
    let client = Client::builder()
        .build()
        .map_err(|_| AppError::transport("Could not initialize the HTTP client."))?;

    response_to_user(send_read_with_retries(&client, &url, api_key)?)
}

fn send_read_with_retries(client: &Client, url: &str, api_key: &str) -> Result<Response, AppError> {
    for attempt in 0..=MAX_READ_RETRIES {
        match client.get(url).header("api-key", api_key).send() {
            Ok(response)
                if should_retry_status(response.status()) && attempt < MAX_READ_RETRIES =>
            {
                wait_before_retry(attempt, retry_after_seconds(&response));
            }
            Ok(response) if response.status() == StatusCode::TOO_MANY_REQUESTS => {
                return Err(AppError::transport_response(
                    "The Hevy API rate limit was exhausted while reading.",
                    &response,
                ));
            }
            Ok(response) if response.status().is_server_error() => {
                return Err(AppError::transport_response(
                    "The Hevy API remained temporarily unavailable while reading.",
                    &response,
                ));
            }
            Ok(response) => return Ok(response),
            Err(_) if attempt < MAX_READ_RETRIES => wait_before_retry(attempt, None),
            Err(_) => return Err(AppError::transport("Could not reach the Hevy API.")),
        }
    }
    unreachable!("the retry loop always returns")
}

fn should_retry_status(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn wait_before_retry(attempt: u8, retry_after: Option<u64>) {
    let delay = retry_after.map(Duration::from_secs).unwrap_or_else(|| {
        let upper_bound_ms = 500_u64.saturating_mul(1_u64 << attempt);
        Duration::from_millis(rand::rng().random_range(0..=upper_bound_ms))
    });
    thread::sleep(delay);
}

fn response_to_user(response: Response) -> Result<Value, AppError> {
    let status = response.status();
    let request_id = request_id(&response);

    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return Err(AppError::authentication_response(
            "The Hevy API rejected the supplied API key.",
            status,
            request_id,
        ));
    }
    if !status.is_success() {
        return Err(AppError::api(
            "The Hevy API request failed.",
            status,
            request_id,
        ));
    }

    let body: Value = response.json().map_err(|_| {
        AppError::api(
            "The Hevy API returned an invalid JSON response.",
            status,
            request_id,
        )
    })?;

    Ok(body
        .get("data")
        .filter(|data| data.is_object())
        .cloned()
        .unwrap_or(body))
}
