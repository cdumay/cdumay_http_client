/*!
# REST Client Module

This module provides a specialized REST client implementation that handles JSON serialization/deserialization
and provides strongly-typed request/response handling. It extends the base HTTP client functionality with
REST-specific features.

## Features

- Automatic JSON serialization/deserialization
- Type-safe request and response handling
- Default JSON content type headers
- Comprehensive error context for JSON parsing failures
- Support for all standard REST methods (GET, POST, PUT, DELETE)
- Generic type parameters for request bodies and responses

## Examples

### Basic Usage

```rust
use cdumay_http_client::{ClientBuilder, RestClient};
use serde::{Deserialize, Serialize};
use cdumay_error::Result;

// Define your data structures
#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(Serialize, Debug)]
struct CreateUser {
    name: String,
    email: String,
}

// Create a REST client
let client = RestClient::new("https://api.example.com", None).unwrap()
    .set_timeout(30)
    .set_retry_number(3);

// GET request with type-safe response
let result: Result<User> = client.get(
    "/users/123".to_string(),
    None,    // No query parameters
    None,    // No additional headers
    None,    // Use default timeout
    None,    // Use default retry behavior
    None,    // No context
);

// POST request with type-safe request body and response
let new_user = CreateUser {
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
};

let result: Result<User> = client.post(
    "/users".to_string(),
    None,
    Some(new_user),
    None,
    None,
    None,
    None,
);
```

### Query Parameters

```rust
use cdumay_http_client::{ClientBuilder, RestClient};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use cdumay_error::Result;

let mut params = HashMap::new();
params.insert("role".to_string(), "admin".to_string());
params.insert("active".to_string(), "true".to_string());

// Define your data structures
#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

let client = RestClient::new("https://api.example.com", None).unwrap();

let result: Result<Vec<User>> = client.get(
    "/users".to_string(),
    Some(params),
    None,
    None,
    None,
    None,
);
```

### Error Handling

```rust
use cdumay_http_client::{ClientBuilder, RestClient};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use cdumay_http_client::errors::http::{NOT_FOUND, FORBIDDEN};

// Specify error types that should not trigger retry
let no_retry = vec![
    NOT_FOUND,
    FORBIDDEN
];

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u64,
    name: String,
    email: String,
}

let client = RestClient::new("https://api.example.com", None).unwrap();

match client.get::<User>(
    "/users/123".to_string(),
    None,
    None,
    None,
    Some(no_retry),
    None,
) {
    Ok(user) => println!("Found user: {:?}", user),
    Err(e) => match e.kind {
        NOT_FOUND => println!("User not found"),
        FORBIDDEN => println!("Access denied"),
        _ => println!("Other error: {}", e),
    }
}
```

### Custom Headers

```rust
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use cdumay_http_client::{ClientBuilder, RestClient};
use serde::{Deserialize, Serialize};
use cdumay_error::Result;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u64,
    name: String,
    email: String,
}

let client = RestClient::new("https://api.example.com", None).unwrap();

let mut headers = HeaderMap::new();
headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.api+json"));

let result: Result<User> = client.get(
    "/users/123".to_string(),
    None,
    Some(headers),
    None,
    None,
    None,
);
```

### Bulk Operations

```rust
use serde_json::Value;
use cdumay_http_client::{ClientBuilder, RestClient};
use serde::{Deserialize, Serialize};
use cdumay_error::Result;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u64,
    name: String,
    email: String,
}

let client = RestClient::new("https://api.example.com", None).unwrap();
// PUT request to update multiple resources
let updates = vec![
    User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
    User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
];

let result: Result<Vec<Value>> = client.put(
    "/users/bulk".to_string(),
    None,
    Some(updates),
    None,
    None,
    None,
    None,
);
```
*/

use crate::authentication::Authentication;
use crate::errors::client::{InvalidHeaderValue, InvalidUrl};
use crate::errors::rest::json_error_serialize;
use crate::{BaseClient, ClientBuilder};
use cdumay_context::Context;
use cdumay_error::{ErrorKind, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, USER_AGENT};
use reqwest::{Method, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Deref;
use serde_value::Value;

/// A specialized REST client that handles JSON serialization/deserialization.
///
/// This client extends the base HTTP client functionality with REST-specific features:
/// - Automatic JSON content type headers
/// - Type-safe request and response handling through generics
/// - Automatic serialization/deserialization of request/response bodies
/// - Enhanced error context for JSON parsing failures
///
/// # Type Parameters
///
/// When making requests, you need to specify the appropriate type parameters:
/// - `D`: The type of data being sent (must implement `Serialize` + `Debug`)
/// - `R`: The type of response expected (must implement `Deserialize`)
///
/// # Examples
///
/// ```rust
/// use cdumay_http_client::{ClientBuilder, RestClient};
/// use serde::{Deserialize, Serialize};
/// use cdumay_error::Result;
///
/// #[derive(Serialize, Deserialize)]
/// struct User {
///     name: String,
///     email: String,
/// }
///
/// let client = RestClient::new("https://api.example.com", None).unwrap();
/// let result: Result<User> = client.get("/users/123".to_string(), None, None, None, None, None);
/// ```
#[derive(Debug)]
pub struct RestClient {
    url_root: Url,
    timeout: u64,
    headers: HeaderMap,
    auth: Option<Box<dyn Authentication>>,
    ssl_verify: bool,
    retry_number: u64,
    retry_delay: u64,
}

impl ClientBuilder for RestClient {
    /// Creates a new REST client with the specified root URL.
    ///
    /// This method initializes a REST client with default settings:
    /// - Content-Type: application/json
    /// - Accept: application/json
    /// - Timeout: 10 seconds
    /// - Retry attempts: 10
    /// - Retry delay: 30 seconds
    /// - SSL verification: enabled
    ///
    /// # Arguments
    ///
    /// * `url_root` - Base URL for all requests
    ///
    /// # Returns
    ///
    /// Returns `Result<RestClient>` which is:
    /// - `Ok(RestClient)` if client creation is successful
    /// - `Err` with an `InvalidUrl` error if URL parsing fails
    fn new(url_root: &str, context: Option<&mut Context>) -> Result<RestClient> {
        Ok(RestClient {
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
                headers.append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                headers.append(ACCEPT, HeaderValue::from_static("application/json"));
                headers
            },
            auth: None,
            ssl_verify: true,
            retry_number: 10,
            retry_delay: 30,
        })
    }

    /// Sets the request timeout in seconds.
    fn set_timeout(mut self, timeout: u64) -> RestClient {
        self.timeout = timeout;
        self
    }

    /// Sets custom headers for all requests.
    fn set_headers(mut self, headers: HeaderMap) -> RestClient {
        self.headers.extend(headers);
        self
    }

    /// Sets the authentication method for all requests.
    fn set_auth<A: Authentication + 'static>(mut self, auth: A) -> RestClient {
        self.auth = Some(Box::new(auth));
        self
    }

    /// Enables or disables SSL certificate verification.
    fn set_ssl_verify(mut self, ssl_verify: bool) -> RestClient {
        self.ssl_verify = ssl_verify;
        self
    }

    /// Sets the number of retry attempts for failed requests.
    fn set_retry_number(mut self, try_number: u64) -> RestClient {
        if try_number == 0 {
            panic!("Try number MUST be > 0 !");
        }
        self.retry_number = try_number;
        self
    }

    /// Sets the delay between retry attempts in seconds.
    fn set_retry_delay(mut self, retry_delay: u64) -> RestClient {
        self.retry_delay = retry_delay;
        self
    }
}

impl BaseClient for RestClient {
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

impl RestClient {
    /// Creates a context object for error reporting.
    ///
    /// This internal method is used to provide detailed context when errors occur,
    /// including the server URL, path, and HTTP method being used.
    fn create_context(&self, path: String, method: Method) -> Context {
        let mut context = Context::default();
        context.insert(
            "server".into(),
            serde_value::Value::String(self.url_root.to_string()),
        );
        context.insert("path".into(), serde_value::Value::String(path));
        context.insert(
            "method".into(),
            serde_value::Value::String(method.to_string()),
        );
        context
    }

    /// Makes a GET request and deserializes the JSON response.
    ///
    /// # Type Parameters
    ///
    /// * `R` - The type to deserialize the response into
    ///
    /// # Arguments
    ///
    /// * `path` - Request path relative to the root URL
    /// * `params` - Optional query parameters
    /// * `headers` - Optional additional headers
    /// * `timeout` - Optional custom timeout for this request
    /// * `no_retry_on` - Optional list of error kinds that should not trigger retry
    /// * `context` - Optional context for error reporting
    ///
    /// # Returns
    ///
    /// Returns `Result<R>` which is:
    /// - `Ok(R)` containing the deserialized response if successful
    /// - `Err` with detailed error information if the request or deserialization fails
    pub fn get<R>(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
        context: Option<Context>,
    ) -> Result<R>
    where
        R: DeserializeOwned,
    {
        Ok(serde_json::from_str(&self.do_request(
            Method::GET,
            path.to_string(),
            params,
            None,
            headers,
            timeout,
            no_retry_on,
            context.clone(),
        )?)
        .map_err(|err| json_error_serialize(err, Some(context.unwrap_or(self.create_context(path, Method::GET)))))?)
    }

    /// Makes a POST request with an optional body and deserializes the JSON response.
    ///
    /// # Type Parameters
    ///
    /// * `D` - The type of data to send in the request body
    /// * `R` - The type to deserialize the response into
    ///
    /// # Arguments
    ///
    /// * `path` - Request path relative to the root URL
    /// * `params` - Optional query parameters
    /// * `data` - Optional request body to serialize as JSON
    /// * `headers` - Optional additional headers
    /// * `timeout` - Optional custom timeout for this request
    /// * `no_retry_on` - Optional list of error kinds that should not trigger retry
    /// * `context` - Optional context for error reporting
    ///
    /// # Returns
    ///
    /// Returns `Result<R>` which is:
    /// - `Ok(R)` containing the deserialized response if successful
    /// - `Err` with detailed error information if the request or deserialization fails
    pub fn post<D, R>(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        data: Option<D>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
        context: Option<Context>,
    ) -> Result<R>
    where
        D: Serialize + Debug,
        R: DeserializeOwned,
    {
        let payload = match data {
            Some(txt) => Some(serde_json::to_string(&txt).map_err(|err| {
                json_error_serialize(err, Some(context.clone().unwrap_or(self.create_context(path.clone(), Method::POST))))
            })?),
            None => None,
        };
        Ok(serde_json::from_str(&self.do_request(
            Method::POST,
            path.to_string(),
            params,
            payload,
            headers,
            timeout,
            no_retry_on,
            context.clone(),
        )?)
        .map_err(|err| json_error_serialize(err, Some(
            context.clone().unwrap_or(self.create_context(path, Method::POST)))))?)
    }

    /// Makes a PUT request with an optional body and deserializes the JSON response.
    ///
    /// # Type Parameters
    ///
    /// * `D` - The type of data to send in the request body
    /// * `R` - The type to deserialize the response into
    ///
    /// # Arguments
    ///
    /// * `path` - Request path relative to the root URL
    /// * `params` - Optional query parameters
    /// * `data` - Optional request body to serialize as JSON
    /// * `headers` - Optional additional headers
    /// * `timeout` - Optional custom timeout for this request
    /// * `no_retry_on` - Optional list of error kinds that should not trigger retry
    /// * `context` - Optional context for error reporting
    ///
    /// # Returns
    ///
    /// Returns `Result<R>` which is:
    /// - `Ok(R)` containing the deserialized response if successful
    /// - `Err` with detailed error information if the request or deserialization fails
    pub fn put<D, R>(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        data: Option<D>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
        context: Option<Context>,
    ) -> Result<R>
    where
        D: Serialize + Debug,
        R: DeserializeOwned,
    {
        let payload = match data {
            Some(txt) => Some(serde_json::to_string(&txt).map_err(|err| {
                json_error_serialize(err, Some(context.clone().unwrap_or(self.create_context(path.clone(), Method::PUT))))
            })?),
            None => None,
        };
        Ok(serde_json::from_str(&self.do_request(
            Method::PUT,
            path.to_string(),
            params,
            payload,
            headers,
            timeout,
            no_retry_on,
            context.clone(),
        )?)
        .map_err(|err| json_error_serialize(err, Some(context.clone().unwrap_or(self.create_context(path, Method::PUT)))))?)
    }

    /// Makes a DELETE request and deserializes the JSON response.
    ///
    /// # Type Parameters
    ///
    /// * `R` - The type to deserialize the response into
    ///
    /// # Arguments
    ///
    /// * `path` - Request path relative to the root URL
    /// * `params` - Optional query parameters
    /// * `headers` - Optional additional headers
    /// * `timeout` - Optional custom timeout for this request
    /// * `no_retry_on` - Optional list of error kinds that should not trigger retry
    /// * `context` - Optional context for error reporting
    ///
    /// # Returns
    ///
    /// Returns `Result<R>` which is:
    /// - `Ok(R)` containing the deserialized response if successful
    /// - `Err` with detailed error information if the request or deserialization fails
    pub fn delete<R>(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
        context: Option<Context>,
    ) -> Result<R>
    where
        R: DeserializeOwned,
    {
        Ok(serde_json::from_str(&self.do_request(
            Method::DELETE,
            path.to_string(),
            params,
            None,
            headers,
            timeout,
            no_retry_on,
            context.clone(),
        )?)
        .map_err(|err| {
            json_error_serialize(err, Some(context.unwrap_or(self.create_context(path, Method::DELETE))))
        })?)
    }
}

#[cfg(test)]
mod test {
    use crate::client_rest::RestClient;
    use crate::errors::http::NOT_FOUND;
    use crate::errors::rest::DataError;
    use crate::ClientBuilder;
    use serde::{Deserialize, Serialize};
    use simple_logger::SimpleLogger;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            let _ = SimpleLogger::new()
                .with_level(log::LevelFilter::Info)
                .init();
        });
    }
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Todo {
        id: usize,
        todo: String,
        completed: bool,
        #[serde(rename(deserialize = "userId"))]
        user_id: u64,
    }
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Foo {
        id: usize,
        foo: String,
    }

    #[test]
    fn test_get() {
        init_logger();
        let cli = RestClient::new("https://dummyjson.com", None).unwrap();
        let result = cli.get::<Todo>("/todos/1".into(), None, None, None, None, None);
        match result {
            Ok(todo) => assert_eq!(todo.user_id, 152),
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn test_get_payload_error() {
        init_logger();
        let cli = RestClient::new("https://dummyjson.com", None)
            .unwrap()
            .set_retry_number(2)
            .set_retry_delay(1);
        let result = cli.get::<Foo>("/todos/1".into(), None, None, None, None, None);
        match result {
            Ok(_) => panic!("No error raised!"),
            Err(err) => assert_eq!(err.kind, DataError),
        }
    }
    #[test]
    fn test_get_response_error() {
        init_logger();
        let cli = RestClient::new("https://dummyjson.com", None)
            .unwrap()
            .set_retry_number(2)
            .set_retry_delay(1);
        let result = cli.get::<Todo>("/todos/a".into(), None, None, None, Some(vec![NOT_FOUND]), None);
        match result {
            Ok(_) => panic!("No error raised!"),
            Err(err) => assert_eq!(err.kind, NOT_FOUND),
        }
    }
}
