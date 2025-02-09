/*!
# HTTP Client Module

This module provides a robust and feature-rich HTTP client implementation with support for:
- Automatic retry mechanism
- Custom authentication
- Request/response header management
- Timeout configuration
- SSL verification
- Error handling with detailed context

## Features

- Builder pattern for easy client configuration
- Support for all standard HTTP methods (GET, POST, PUT, DELETE, HEAD)
- Automatic retry with configurable attempts and delay
- Custom authentication support
- Header management
- Query parameter support
- Detailed error reporting with context
- Logging of request/response details

## Examples

### Basic Usage

```rust
use cdumay_http_client::{ClientBuilder, HttpClient};
use std::collections::HashMap;

// Create a new client
let client = HttpClient::new("https://api.example.com", None).unwrap()
    .set_timeout(30)       // 30 seconds timeout
    .set_retry_number(3)   // Retry 3 times
    .set_retry_delay(5);   // 5 seconds between retries

// Make a GET request
let result = client.get(
    "/users".to_string(),
    None,                  // No query parameters
    None,                  // No additional headers
    None,                  // Use default timeout
    None,                  // Use default retry behavior
    None,                  // No context
);

// Make a POST request with data
let result = client.post(
    "/users".to_string(),
    None,                  // No query parameters
    Some("{'name':'John'}".to_string()),
    None,                  // No additional headers
    None,                  // Use default timeout
    None,                  // Use default retry behavior
    None,                  // No context
);
```

### Query Parameters

```rust
use std::collections::HashMap;
use cdumay_http_client::{ClientBuilder, HttpClient};

let mut params = HashMap::new();
params.insert("page".to_string(), "1".to_string());
params.insert("limit".to_string(), "10".to_string());

let client = HttpClient::new("https://api.example.com", None).unwrap();

let result = client.get(
    "/users".to_string(),
    Some(params),
    None,
    None,
    None,
    None,
);
```

### Custom Headers

```rust
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use cdumay_http_client::{ClientBuilder, HttpClient};

let mut headers = HeaderMap::new();
headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

let client = HttpClient::new("https://api.example.com", None).unwrap();

let result = client.post(
    "/users".to_string(),
    None,
    Some("{'name':'John'}".to_string()),
    Some(headers),
    None,
    None,
    None,
);
```

### Error Handling

```rust
use cdumay_error::ErrorKind;
use cdumay_http_client::{ClientBuilder, HttpClient};
use cdumay_http_client::errors::http::{NOT_FOUND, FORBIDDEN};

// Specify error types that should not trigger retry
let no_retry = vec![
    NOT_FOUND,
    FORBIDDEN
];

let client = HttpClient::new("https://api.example.com", None).unwrap();

let result = client.get(
    "/users/123".to_string(),
    None,
    None,
    None,
    Some(no_retry),
    None,
);
```

### Authentication

```rust
use cdumay_http_client::authentication::basic::BasicAuth;
use cdumay_http_client::{HttpClient, ClientBuilder};

let auth = BasicAuth::new(
    "username".to_string(),
    Some("password".to_string())
);

let client = HttpClient::new("https://api.example.com", None).unwrap()
    .set_auth(auth);
```
*/

use cdumay_context::Context;
use cdumay_error::{Error, ErrorKind, Result};
use chrono::Utc;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Method, Url};
use serde_value::Value;
use std::collections::HashMap;
use std::ops::Deref;
use std::thread;
use std::time::Duration;

use crate::authentication::Authentication;
use crate::errors::client::{ClientBuilderError, InvalidHeaderValue, InvalidUrl};
use crate::errors::{http_error_serialize, http_resp_serialise};
use crate::utils::{build_url, merge_headers};

/// Trait for building HTTP clients with configurable settings.
///
/// This trait provides a builder pattern for creating and configuring HTTP clients.
/// Implementations can customize various aspects like timeout, headers, authentication,
/// SSL verification, and retry behavior.
pub trait ClientBuilder {
    /// Creates a new client instance with the specified root URL.
    ///
    /// # Arguments
    ///
    /// * `url_root` - Base URL for all requests made by this client
    ///
    /// # Returns
    ///
    /// Returns `Result<Self>` which is:
    /// - `Ok(Self)` if client creation is successful
    /// - `Err` with an `InvalidUrl` error if URL parsing fails
    fn new(url_root: &str, context: Option<&mut Context>) -> Result<Self>
    where
        Self: Sized;

    /// Sets the request timeout in seconds.
    fn set_timeout(self, timeout: u64) -> Self;

    /// Sets custom headers for all requests.
    fn set_headers(self, headers: HeaderMap) -> Self;

    /// Sets the authentication method for all requests.
    fn set_auth<A: Authentication + 'static>(self, auth: A) -> Self;

    /// Enables or disables SSL certificate verification.
    fn set_ssl_verify(self, ssl_verify: bool) -> Self;

    /// Sets the number of retry attempts for failed requests.
    fn set_retry_number(self, retry_number: u64) -> Self;

    /// Sets the delay between retry attempts in seconds.
    fn set_retry_delay(self, retry_delay: u64) -> Self;
}

/// Base trait for HTTP client implementations.
///
/// This trait defines the core functionality that all HTTP clients must implement,
/// including configuration getters and request handling.
pub trait BaseClient {
    /// Returns the root URL for all requests.
    fn url_root(&self) -> &Url;

    /// Returns the configured timeout in seconds.
    fn timeout(&self) -> &u64;

    /// Returns the configured headers.
    fn headers(&self) -> &HeaderMap;

    /// Returns the configured authentication method, if any.
    fn auth(&self) -> Option<&Box<dyn Authentication>>;

    /// Returns whether SSL verification is enabled.
    fn ssl_verify(&self) -> bool;

    /// Returns the number of retry attempts for failed requests.
    fn retry_number(&self) -> u64;

    /// Returns the delay between retry attempts in seconds.
    fn retry_delay(&self) -> u64;

    /// Internal method to wrap request execution with error handling.
    fn _request_wrapper(&self, req: RequestBuilder) -> Result<Response> {
        Ok(req.send().map_err(|err| http_error_serialize(&err, None))?)
    }

    /// Makes an HTTP request with the specified parameters.
    ///
    /// This method handles all the request logic including:
    /// - URL construction
    /// - Header management
    /// - Authentication
    /// - Retry logic
    /// - Error handling
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method to use
    /// * `path` - Request path relative to the root URL
    /// * `params` - Optional query parameters
    /// * `data` - Optional request body
    /// * `headers` - Optional additional headers
    /// * `timeout` - Optional custom timeout for this request
    /// * `no_retry_on` - Optional list of error kinds that should not trigger retry
    /// * `context` - Optional context for error reporting
    ///
    /// # Returns
    ///
    /// Returns `Result<String>` which is:
    /// - `Ok(String)` containing the response body if successful
    /// - `Err` with detailed error information if the request fails
    fn do_request(
        &self,
        method: Method,
        path: String,
        params: Option<HashMap<String, String>>,
        data: Option<String>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
        context: Option<Context>,
    ) -> Result<String> {
        let start = Utc::now();
        let url = build_url(self.url_root(), path, params)?;
        let mut context = context.unwrap_or_default();
        context.insert("url".into(), Value::String(url.to_string()));
        context.insert("method".into(), Value::String(method.to_string()));
        let cli = Client::builder()
            .timeout(Duration::from_secs(
                timeout.unwrap_or(self.timeout().clone()),
            ))
            .default_headers(merge_headers(self.headers(), headers))
            .build()
            .map_err(|err| http_error_serialize(&err, Some(context.clone().into())))?;

        debug!("{} {}", &method, &url.as_str());
        let mut req = cli.request(method.clone(), url.clone());
        if let Some(auth) = self.auth() {
            if let Some((name, value)) = auth.as_header() {
                req = req.header(name, value);
            }
        }
        if let Some(txt) = data {
            req = req.body::<String>(txt);
        }
        let mut last_error: Option<Error> = None;
        for req_try in 1..=self.retry_number() {
            info!("[{}] - {} (try: {})", method, url, req_try);
            match req.try_clone() {
                Some(req) => {
                    let resp = self._request_wrapper(req)?;
                    let end = { Utc::now() - start }.to_std().unwrap();
                    let human = humantime::format_duration(end).to_string();
                    let length = resp.content_length().unwrap_or(0);
                    match resp.status().is_success() {
                        true => {
                            info!(
                                "{} {} - {} - {} [{}]",
                                &method,
                                &url.as_str(),
                                resp.status(),
                                length,
                                &human
                            );
                            return Ok(resp.text().map_err(|err| {
                                http_error_serialize(&err, Some(context.into()))
                            })?);
                        }
                        false => {
                            error!(
                                "{} {} - {} - {} [{}]",
                                &method,
                                &url.as_str(),
                                resp.status(),
                                length,
                                &human
                            );
                            let mut err_context = context.clone();
                            err_context.insert("try".into(), Value::U64(req_try));
                            let err = http_resp_serialise(resp, Some(err_context));
                            if let Some(kinds) = &no_retry_on {
                                if kinds.contains(&err.kind) {
                                    return Err(err);
                                }
                            }
                            last_error = Some(err);
                        }
                    };
                    thread::sleep(Duration::from_secs(self.retry_delay()));
                }
                None => {
                    return Err(ClientBuilderError::new()
                        .set_message("Internal error, failed to clone request".into())
                        .set_details(context.into())
                        .into())
                }
            }
        }
        match last_error {
            Some(err) => {
                error!(
                    "Failed to perform request {} on {} after {} retries : {}",
                    method,
                    url,
                    self.retry_number(),
                    err
                );
                Err(err)
            }
            None => {
                error!(
                    "Unexpected error, failed to perform request {} on {} after {} retries",
                    method,
                    url,
                    self.retry_number()
                );
                Err(ClientBuilderError::new()
                    .set_message("Internal error, failed to clone request".into())
                    .set_details(context.into())
                    .into())
            }
        }
    }

    /// Makes an HEAD request with the specified parameters.
    ///
    /// This method handles all the request logic including:
    /// - URL construction
    /// - Header management
    /// - Retry logic
    /// - Error handling
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method to use
    /// * `path` - Request path relative to the root URL
    /// * `params` - Optional query parameters
    /// * `headers` - Optional additional headers
    /// * `timeout` - Optional custom timeout for this request
    /// * `no_retry_on` - Optional list of error kinds that should not trigger retry
    /// * `context` - Optional context for error reporting
    ///
    /// # Returns
    ///
    /// Returns `Result<String>` which is:
    /// - `Ok(())` No response is required
    /// - `Err` with detailed error information if the request fails
    fn head(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
        context: Option<Context>,
    ) -> Result<()> {
        self.do_request(
            Method::HEAD,
            path,
            params,
            None,
            headers,
            timeout,
            no_retry_on,
            context,
        )?;
        Ok(())
    }
}

/// HTTP client implementation with retry capabilities and configurable settings.
///
/// This struct provides a concrete implementation of both `ClientBuilder` and
/// `BaseClient` traits, offering a fully-featured HTTP client with:
/// - Automatic retry mechanism
/// - Custom authentication support
/// - Header management
/// - Timeout configuration
/// - SSL verification
///
/// # Examples
///
/// ```rust
/// use cdumay_http_client::{ClientBuilder, HttpClient};
/// use cdumay_http_client::authentication::basic::BasicAuth;
///
/// // Create a client with basic authentication
/// let client = HttpClient::new("https://api.example.com", None).unwrap()
///     .set_timeout(30)
///     .set_auth(BasicAuth::new(
///         "username".to_string(),
///         Some("password".to_string())
///     ))
///     .set_retry_number(3)
///     .set_retry_delay(5);
///
/// // Make a GET request
/// let result = client.get(
///     "/users".to_string(),
///     None,
///     None,
///     None,
///     None,
///     None,
/// );
/// ```
#[derive(Debug)]
pub struct HttpClient {
    url_root: Url,
    timeout: u64,
    headers: HeaderMap,
    auth: Option<Box<dyn Authentication>>,
    ssl_verify: bool,
    retry_number: u64,
    retry_delay: u64,
}

impl ClientBuilder for HttpClient {
    fn new(url_root: &str, context: Option<&mut Context>) -> Result<Self> {
        Ok(HttpClient {
            url_root: Url::parse(url_root.trim_end_matches("/")).map_err(|err| {
                InvalidUrl::new()
                    .set_message(format!("Failed to parse URL: {:?}", err))
                    .set_details({
                        let mut err_context = Context::new();
                        if let Some(ctx) = context {
                            err_context.extend(ctx.deref().clone().into());
                        };
                        err_context.insert("url".to_string(), Value::String(url_root.to_string()));
                        err_context.into()
                    })
            })?,
            timeout: 10,
            headers: {
                let mut headers = HeaderMap::new();
                headers.append(
                    USER_AGENT,
                    HeaderValue::from_str(&format!(
                        "{}/{}",
                        env!("CARGO_PKG_NAME"),
                        env!("CARGO_PKG_VERSION")
                    ))
                    .map_err(|err| InvalidHeaderValue::new().set_message(err.to_string()))?,
                );
                headers
            },
            auth: None,
            ssl_verify: true,
            retry_number: 10,
            retry_delay: 30,
        })
    }

    fn set_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    fn set_headers(mut self, headers: HeaderMap) -> Self {
        self.headers.extend(headers);
        self
    }

    fn set_auth<A: Authentication + 'static>(mut self, auth: A) -> Self {
        self.auth = Some(Box::new(auth));
        self
    }

    fn set_ssl_verify(mut self, ssl_verify: bool) -> Self {
        self.ssl_verify = ssl_verify;
        self
    }

    fn set_retry_number(mut self, retry_number: u64) -> Self {
        if retry_number == 0 {
            panic!("Try number MUST be > 0 !");
        }
        self.retry_number = retry_number;
        self
    }

    fn set_retry_delay(mut self, retry_delay: u64) -> Self {
        self.retry_delay = retry_delay;
        self
    }
}

impl BaseClient for HttpClient {
    fn url_root(&self) -> &Url {
        &self.url_root
    }

    fn timeout(&self) -> &u64 {
        &self.timeout
    }

    fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    fn auth(&self) -> Option<&Box<dyn Authentication>> {
        self.auth.as_ref()
    }

    fn ssl_verify(&self) -> bool {
        self.ssl_verify
    }

    fn retry_number(&self) -> u64 {
        self.retry_number
    }

    fn retry_delay(&self) -> u64 {
        self.retry_delay
    }
}

impl HttpClient {
    /// Makes a GET request.
    ///
    /// # Arguments
    ///
    /// * `path` - Request path relative to the root URL
    /// * `params` - Optional query parameters
    /// * `headers` - Optional additional headers
    /// * `timeout` - Optional custom timeout for this request
    /// * `no_retry_on` - Optional list of error kinds that should not trigger retry
    /// * `context` - Optional context for error reporting
    pub fn get(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
        context: Option<Context>,
    ) -> Result<String> {
        self.do_request(
            Method::GET,
            path,
            params,
            None,
            headers,
            timeout,
            no_retry_on,
            context,
        )
    }

    /// Makes a POST request.
    ///
    /// # Arguments
    ///
    /// * `path` - Request path relative to the root URL
    /// * `params` - Optional query parameters
    /// * `data` - Optional request body
    /// * `headers` - Optional additional headers
    /// * `timeout` - Optional custom timeout for this request
    /// * `no_retry_on` - Optional list of error kinds that should not trigger retry
    /// * `context` - Optional context for error reporting
    pub fn post(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        data: Option<String>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
        context: Option<Context>,
    ) -> Result<String> {
        self.do_request(
            Method::POST,
            path,
            params,
            data,
            headers,
            timeout,
            no_retry_on,
            context,
        )
    }

    /// Makes a PUT request.
    ///
    /// # Arguments
    ///
    /// * `path` - Request path relative to the root URL
    /// * `params` - Optional query parameters
    /// * `data` - Optional request body
    /// * `headers` - Optional additional headers
    /// * `timeout` - Optional custom timeout for this request
    /// * `no_retry_on` - Optional list of error kinds that should not trigger retry
    /// * `context` - Optional context for error reporting
    pub fn put(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        data: Option<String>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
        context: Option<Context>,
    ) -> Result<String> {
        self.do_request(
            Method::PUT,
            path,
            params,
            data,
            headers,
            timeout,
            no_retry_on,
            context,
        )
    }

    /// Makes a DELETE request.
    ///
    /// # Arguments
    ///
    /// * `path` - Request path relative to the root URL
    /// * `params` - Optional query parameters
    /// * `headers` - Optional additional headers
    /// * `timeout` - Optional custom timeout for this request
    /// * `no_retry_on` - Optional list of error kinds that should not trigger retry
    /// * `context` - Optional context for error reporting
    pub fn delete(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
        context: Option<Context>,
    ) -> Result<String> {
        self.do_request(
            Method::DELETE,
            path,
            params,
            None,
            headers,
            timeout,
            no_retry_on,
            context,
        )
    }
}

#[cfg(test)]
mod test {
    use std::sync::Once;

    use simple_logger::SimpleLogger;

    use crate::errors::http::UNPROCESSABLE_ENTITY;
    use crate::{ClientBuilder, HttpClient};

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            let _ = SimpleLogger::new()
                .with_level(log::LevelFilter::Info)
                .init();
        });
    }

    #[test]
    fn test_no_auth() {
        init_logger();
        let cli = HttpClient::new("https://www.rust-lang.org", None).unwrap();
        let result = cli.get("/learn/get-started".into(), None, None, None, None, None);
        assert_eq!(result.unwrap().starts_with("<!doctype html>"), true);
    }

    #[test]
    fn test_err() {
        init_logger();
        let cli = HttpClient::new("https://www.rust-lang.org", None)
            .unwrap()
            .set_retry_number(2)
            .set_retry_delay(1);
        match cli.get("/sdq".into(), None, None, None, None, None) {
            Ok(_) => panic!("No error raised!"),
            Err(err) => assert_eq!(err.kind, UNPROCESSABLE_ENTITY),
        };
    }

    #[test]
    fn test_err_no_retry() {
        init_logger();
        let cli = HttpClient::new("https://www.rust-lang.org", None)
            .unwrap()
            .set_retry_number(2)
            .set_retry_delay(1);
        match cli.get(
            "/sdq".into(),
            None,
            None,
            None,
            Some(vec![UNPROCESSABLE_ENTITY]),
            None,
        ) {
            Ok(_) => panic!("No error raised!"),
            Err(err) => {
                assert_eq!(err.kind, UNPROCESSABLE_ENTITY);
                match err.details {
                    Some(data) => {
                        match data.get("try") {
                            Some(value) => {
                                assert_eq!(serde_value::Value::U64(1), value.clone());
                            }
                            None => panic!("No try in error.extra"),
                        };
                    }
                    None => panic!("Not error extra found !"),
                }
            }
        };
    }
}
