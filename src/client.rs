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

pub struct Pagination {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub all: bool,
}

pub fn get_user(api_key: &str) -> Result<Value, AppError> {
    let base_url =
        env::var("HEVY_API_BASE_URL").unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_owned());
    let url = format!("{}/v1/user/info", base_url.trim_end_matches('/'));
    let client = Client::builder()
        .build()
        .map_err(|_| AppError::transport("Could not initialize the HTTP client."))?;

    response_to_user(send_read_with_retries(&client, &url, api_key)?)
}

pub fn get_workout_count(api_key: &str) -> Result<Value, AppError> {
    get_read_value(api_key, "/v1/workouts/count")
}

pub fn create_workout(api_key: &str, payload: &Value) -> Result<Value, AppError> {
    mutate_resource(api_key, "workouts", None, payload)
}

pub fn update_workout(api_key: &str, workout_id: &str, payload: &Value) -> Result<Value, AppError> {
    mutate_resource(api_key, "workouts", Some(workout_id), payload)
}

pub fn create_routine(api_key: &str, payload: &Value) -> Result<Value, AppError> {
    mutate_resource(api_key, "routines", None, payload)
}

pub fn create_body_measurement(api_key: &str, payload: &Value) -> Result<Value, AppError> {
    mutate_resource(api_key, "body_measurements", None, payload)
}

pub fn update_body_measurement(
    api_key: &str,
    date: &str,
    payload: &Value,
) -> Result<Value, AppError> {
    mutate_resource(api_key, "body_measurements", Some(date), payload)
}

pub fn create_exercise_template(api_key: &str, payload: &Value) -> Result<Value, AppError> {
    mutate_resource(api_key, "exercise_templates", None, payload)
}

pub fn create_routine_folder(api_key: &str, payload: &Value) -> Result<Value, AppError> {
    mutate_resource(api_key, "routine_folders", None, payload)
}

pub fn update_routine(api_key: &str, routine_id: &str, payload: &Value) -> Result<Value, AppError> {
    mutate_resource(api_key, "routines", Some(routine_id), payload)
}

fn mutate_resource(
    api_key: &str,
    resource: &str,
    resource_id: Option<&str>,
    payload: &Value,
) -> Result<Value, AppError> {
    let base_url =
        env::var("HEVY_API_BASE_URL").unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_owned());
    let mut url = reqwest::Url::parse(&base_url)
        .map_err(|_| AppError::transport("Could not construct the Hevy API request."))?;
    let mut segments = vec!["v1", resource];
    if let Some(resource_id) = resource_id {
        segments.push(resource_id);
    }
    url.path_segments_mut()
        .map_err(|_| AppError::transport("Could not construct the Hevy API request."))?
        .extend(segments);
    let client = Client::builder()
        .build()
        .map_err(|_| AppError::transport("Could not initialize the HTTP client."))?;
    let request = match resource_id {
        Some(_) => client.put(url).header("api-key", api_key).json(payload),
        None => client.post(url).header("api-key", api_key).json(payload),
    };
    let resource_name = resource.trim_end_matches('s');
    let response = request.send().map_err(|_| {
        AppError::transport(format!(
            "The {resource_name} mutation outcome is unknown. Reconcile the affected {resource_name} before retrying."
        ))
    })?;
    if resource == "body_measurements" && response.status() == StatusCode::CONFLICT {
        return Err(AppError::api(
            "A body measurement already exists for that date. Retrieve it and use update to replace all measurement fields.",
            response.status(),
            request_id(&response),
        ));
    }
    response_to_json(response)
}

pub fn list_routines(api_key: &str, pagination: Pagination) -> Result<Value, AppError> {
    list_paginated(api_key, "/v1/routines", pagination, "routines", None)
}

pub fn list_body_measurements(api_key: &str, pagination: Pagination) -> Result<Value, AppError> {
    list_paginated(
        api_key,
        "/v1/body_measurements",
        pagination,
        "body_measurements",
        None,
    )
}

pub fn get_body_measurement(api_key: &str, date: &str) -> Result<Value, AppError> {
    get_resource(api_key, "body_measurements", date)
}

pub fn get_routine(api_key: &str, routine_id: &str) -> Result<Value, AppError> {
    get_resource(api_key, "routines", routine_id)
}

pub fn get_workout(api_key: &str, workout_id: &str) -> Result<Value, AppError> {
    get_resource(api_key, "workouts", workout_id)
}

pub fn list_exercise_templates(api_key: &str, pagination: Pagination) -> Result<Value, AppError> {
    list_paginated(
        api_key,
        "/v1/exercise_templates",
        pagination,
        "exercise_templates",
        None,
    )
}

pub fn get_exercise_template(api_key: &str, exercise_template_id: &str) -> Result<Value, AppError> {
    get_resource(api_key, "exercise_templates", exercise_template_id)
}

pub fn list_routine_folders(api_key: &str, pagination: Pagination) -> Result<Value, AppError> {
    list_paginated(
        api_key,
        "/v1/routine_folders",
        pagination,
        "routine_folders",
        None,
    )
}

pub fn get_routine_folder(api_key: &str, folder_id: &str) -> Result<Value, AppError> {
    get_resource(api_key, "routine_folders", folder_id)
}

pub fn get_exercise_history(
    api_key: &str,
    exercise_template_id: &str,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<Value, AppError> {
    let base_url =
        env::var("HEVY_API_BASE_URL").unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_owned());
    let mut url = reqwest::Url::parse(&base_url)
        .map_err(|_| AppError::transport("Could not construct the Hevy API request."))?;
    url.path_segments_mut()
        .map_err(|_| AppError::transport("Could not construct the Hevy API request."))?
        .extend(["v1", "exercise_history", exercise_template_id]);
    {
        let mut query = url.query_pairs_mut();
        if let Some(start) = start {
            query.append_pair("start_date", start);
        }
        if let Some(end) = end {
            query.append_pair("end_date", end);
        }
    }
    let client = Client::builder()
        .build()
        .map_err(|_| AppError::transport("Could not initialize the HTTP client."))?;
    response_to_json(send_read_with_retries(&client, url.as_str(), api_key)?)
}

fn get_resource(api_key: &str, resource: &str, resource_id: &str) -> Result<Value, AppError> {
    let base_url =
        env::var("HEVY_API_BASE_URL").unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_owned());
    let mut url = reqwest::Url::parse(&base_url)
        .map_err(|_| AppError::transport("Could not construct the Hevy API request."))?;
    url.path_segments_mut()
        .map_err(|_| AppError::transport("Could not construct the Hevy API request."))?
        .extend(["v1", resource, resource_id]);
    let client = Client::builder()
        .build()
        .map_err(|_| AppError::transport("Could not initialize the HTTP client."))?;
    response_to_json(send_read_with_retries(&client, url.as_str(), api_key)?)
}

fn get_read_value(api_key: &str, path: &str) -> Result<Value, AppError> {
    let base_url =
        env::var("HEVY_API_BASE_URL").unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_owned());
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    let client = Client::builder()
        .build()
        .map_err(|_| AppError::transport("Could not initialize the HTTP client."))?;
    response_to_json(send_read_with_retries(&client, &url, api_key)?)
}

pub fn list_workouts(api_key: &str, pagination: Pagination) -> Result<Value, AppError> {
    list_paginated(api_key, "/v1/workouts", pagination, "workouts", None)
}

pub fn list_workout_events(
    api_key: &str,
    pagination: Pagination,
    since: Option<&str>,
) -> Result<Value, AppError> {
    list_paginated(api_key, "/v1/workouts/events", pagination, "events", since)
}

fn list_paginated(
    api_key: &str,
    path: &str,
    pagination: Pagination,
    item_key: &str,
    since: Option<&str>,
) -> Result<Value, AppError> {
    let base_url =
        env::var("HEVY_API_BASE_URL").unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_owned());
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    let client = Client::builder()
        .build()
        .map_err(|_| AppError::transport("Could not initialize the HTTP client."))?;

    if pagination.all {
        return get_all_pages(
            &client,
            &url,
            api_key,
            pagination.page_size,
            item_key,
            since,
        );
    }

    let response = send_read_with_retries(
        &client,
        &with_query(&url, pagination.page, pagination.page_size, since)?,
        api_key,
    )?;
    normalize_collection(response_to_json(response)?, item_key)
}

fn get_all_pages(
    client: &Client,
    url: &str,
    api_key: &str,
    page_size: Option<u32>,
    item_key: &str,
    since: Option<&str>,
) -> Result<Value, AppError> {
    let mut items = Vec::new();
    let mut pages_fetched = Vec::new();
    let mut page = 1;
    let mut page_count = 1;

    while page <= page_count {
        let response = send_read_with_retries(
            client,
            &with_query(url, Some(page), page_size, since)?,
            api_key,
        )?;
        let collection = response_to_json(response)?;
        page_count = collection
            .get("page_count")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| {
                AppError::api_message("The Hevy API returned an invalid paginated response.")
            })?;
        let page_items = collection
            .get(item_key)
            .and_then(Value::as_array)
            .ok_or_else(|| {
                AppError::api_message("The Hevy API returned an invalid paginated response.")
            })?;
        items.extend(page_items.iter().cloned());
        pages_fetched.push(page);
        page += 1;
    }

    Ok(serde_json::json!({
        "items": items,
        "page": 1,
        "page_count": page_count,
        "all": true,
        "pages_fetched": pages_fetched,
    }))
}

fn with_query(
    url: &str,
    page: Option<u32>,
    page_size: Option<u32>,
    since: Option<&str>,
) -> Result<String, AppError> {
    if page.is_none() && page_size.is_none() && since.is_none() {
        return Ok(url.to_owned());
    }

    let mut url = reqwest::Url::parse(url)
        .map_err(|_| AppError::transport("Could not construct the Hevy API request."))?;
    {
        let mut query = url.query_pairs_mut();
        if let Some(page) = page {
            query.append_pair("page", &page.to_string());
        }
        if let Some(page_size) = page_size {
            query.append_pair("pageSize", &page_size.to_string());
        }
        if let Some(since) = since {
            query.append_pair("since", since);
        }
    }
    Ok(url.into())
}

fn normalize_collection(collection: Value, item_key: &str) -> Result<Value, AppError> {
    let page = collection
        .get("page")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            AppError::api_message("The Hevy API returned an invalid paginated response.")
        })?;
    let page_count = collection
        .get("page_count")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            AppError::api_message("The Hevy API returned an invalid paginated response.")
        })?;
    let items = collection
        .get(item_key)
        .and_then(Value::as_array)
        .ok_or_else(|| {
            AppError::api_message("The Hevy API returned an invalid paginated response.")
        })?;

    Ok(serde_json::json!({ "items": items, "page": page, "page_count": page_count }))
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
    let body = response_to_json(response)?;
    Ok(body
        .get("data")
        .filter(|data| data.is_object())
        .cloned()
        .unwrap_or(body))
}

fn response_to_json(response: Response) -> Result<Value, AppError> {
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

    response.json().map_err(|_| {
        AppError::api(
            "The Hevy API returned an invalid JSON response.",
            status,
            request_id,
        )
    })
}
