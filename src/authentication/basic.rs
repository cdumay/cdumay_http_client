/*!
# Basic Authentication

This module provides HTTP Basic Authentication implementation for the HTTP client library.
Basic Authentication is a simple authentication scheme built into the HTTP protocol.
The client sends HTTP requests with the Authorization header that contains the word
Basic followed by a space and a base64-encoded string username:password.

## Security Note

Basic Authentication sends credentials as base64 encoded text that can be easily
decoded. Therefore, it should only be used over HTTPS/TLS to ensure the credentials
are encrypted during transmission.

## Examples

### Basic Usage

```rust
use cdumay_http_client::{ClientBuilder, HttpClient};
use cdumay_http_client::authentication::basic::BasicAuth;

// Create authentication with username and password
let auth = BasicAuth::new(
    "john.doe".to_string(),
    Some("secret123".to_string())
);

// Create client with basic authentication
let client = HttpClient::new("https://api.example.com").unwrap()
    .set_auth(auth);

// Make authenticated request
let response = client.get(
    "/protected-resource".to_string(),
    None,
    None,
    None,
    None,
);
```

### Username Only

You can also create Basic Authentication with just a username:

```rust
use cdumay_http_client::authentication::basic::BasicAuth;

let auth = BasicAuth::new(
    "anonymous".to_string(),
    None  // No password
);
```
*/

use base64::prelude::*;
use reqwest::header::{AUTHORIZATION, HeaderName, HeaderValue};
use crate::authentication::Authentication;

/// Basic Authentication implementation.
///
/// This struct implements the HTTP Basic Authentication scheme.
/// It can be created with a username and an optional password.
///
/// # Examples
///
/// ```rust
/// use cdumay_http_client::authentication::basic::BasicAuth;
///
/// // With password
/// let auth = BasicAuth::new(
///     "username".to_string(),
///     Some("password".to_string())
/// );
///
/// // Without password
/// let auth = BasicAuth::new(
///     "username".to_string(),
///     None
/// );
/// ```
#[derive(Debug)]
pub struct BasicAuth {
    username: String,
    password: Option<String>,
}

impl BasicAuth {
    /// Creates a new Basic Authentication instance.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for authentication
    /// * `password` - Optional password. If None, only the username will be used
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cdumay_http_client::authentication::basic::BasicAuth;
    ///
    /// let auth = BasicAuth::new(
    ///     "john.doe".to_string(),
    ///     Some("secret123".to_string())
    /// );
    /// ```
    pub fn new(username: String, password: Option<String>) -> BasicAuth {
        BasicAuth {
            username,
            password,
        }
    }
}

impl Authentication for BasicAuth {
    fn username(&self) -> Option<String> { Some(self.username.clone()) }
    fn password(&self) -> Option<String> { self.password.clone() }
    
    /// Generates the Basic Authentication header.
    ///
    /// This method creates the Authorization header with the Basic authentication
    /// scheme. The header value is created by:
    /// 1. Combining username and password (if any) with a colon
    /// 2. Base64 encoding the resulting string
    /// 3. Prepending "Basic " to the encoded string
    ///
    /// # Returns
    ///
    /// Returns `Some((HeaderName, HeaderValue))` containing the Authorization
    /// header name and the properly formatted Basic auth value.
    fn as_header(&self) -> Option<(HeaderName, HeaderValue)> {
        let auth = match self.password() {
            Some(password) => format!("{}:{}", self.username, password),
            None => format!("{}:", self.username)
        };
        let header_value = format!("Basic {}", BASE64_STANDARD.encode(&auth));
        Some((AUTHORIZATION, HeaderValue::from_str(&*header_value).unwrap()))
    }
}
