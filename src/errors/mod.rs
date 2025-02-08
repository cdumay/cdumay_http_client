use cdumay_context::Context;
use cdumay_error::Error;
use reqwest::blocking::Response;

pub mod client;
pub mod http;
pub mod rest;

pub fn http_resp_serialise(resp: Response, context: Option<Context>) -> Error {
    http::from_status(
        resp.status(),
        resp.text().unwrap_or_default(),
        context.unwrap_or_default().into(),
    )
}

pub fn http_error_serialize(error: &reqwest::Error, context: Option<Context>) -> Error {
    let context = context.unwrap_or_default();
    if let Some(code) = error.status() {
        return http::from_status(code, error.to_string(), context.into());
    }
    if error.is_builder() {
        return client::ClientBuilderError::new()
            .set_message(error.to_string())
            .set_details(context.into())
            .into();
    }
    if error.is_body() || error.is_decode() {
        return client::InvalidContent::new()
            .set_message(error.to_string())
            .set_details(context.into())
            .into();
    }
    if error.is_connect() || error.is_timeout() {
        return client::NetworkError::new()
            .set_message(error.to_string())
            .set_details(context.into())
            .into();
    }
    if error.is_request() {
        return client::NetworkError::new()
            .set_message(error.to_string())
            .set_details(context.into())
            .into();
    }
    client::UnexpectedError::new()
        .set_message(error.to_string())
        .set_details(context.into())
        .into()
}
