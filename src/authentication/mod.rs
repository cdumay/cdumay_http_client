/*!
# Authentication Module

This module provides authentication mechanisms for the HTTP client library. It includes a trait for implementing
different authentication methods and built-in implementations for common authentication schemes.

## Features

- Flexible authentication trait system
- Built-in Basic Authentication support
- No Authentication option for public endpoints
- Easy to extend with custom authentication methods

## Usage Examples

### No Authentication

For public endpoints that don't require authentication:

```rust
use cdumay_http_client::{ClientBuilder, HttpClient};
use cdumay_http_client::authentication::NoAuth;

let client = HttpClient::new("https://api.example.com", None).unwrap()
    .set_auth(NoAuth);
```

### Basic Authentication

For endpoints that require HTTP Basic Authentication:

```rust
use cdumay_http_client::{ClientBuilder, HttpClient};
use cdumay_http_client::authentication::basic::BasicAuth;

// With username and password
let auth = BasicAuth::new(
    "username".to_string(),
    Some("password".to_string())
);

let client = HttpClient::new("https://api.example.com", None).unwrap()
    .set_auth(auth);

// With username only
let auth = BasicAuth::new(
    "username".to_string(),
    None
);
```

### Custom Authentication

Implement the `Authentication` trait for custom authentication methods:

```rust
use cdumay_http_client::{ClientBuilder, HttpClient};
use cdumay_http_client::authentication::Authentication;
use reqwest::header::{HeaderName, HeaderValue, AUTHORIZATION};

#[derive(Debug)]
struct BearerAuth {
    token: String,
}

impl Authentication for BearerAuth {
    fn username(&self) -> Option<String> { None }
    fn password(&self) -> Option<String> { None }
    fn as_header(&self) -> Option<(HeaderName, HeaderValue)> {
        let value = format!("Bearer {}", self.token);
        Some((
            AUTHORIZATION,
            HeaderValue::from_str(&value).unwrap()
        ))
    }
}

// Using custom authentication
let auth = BearerAuth {
    token: "your-token".to_string()
};

let client = HttpClient::new("https://api.example.com", None).unwrap()
    .set_auth(auth);
```
*/

use std::fmt::Debug;
use reqwest::header::{HeaderName, HeaderValue};

pub mod basic;

/// Trait for implementing authentication methods.
///
/// This trait should be implemented by any struct that provides authentication
/// functionality. It requires implementing methods to get the username,
/// password (if applicable), and to generate the appropriate authentication header.
///
/// # Examples
///
/// ```rust
/// use cdumay_http_client::authentication::Authentication;
/// use reqwest::header::{HeaderName, HeaderValue, AUTHORIZATION};
///
/// #[derive(Debug)]
/// struct ApiKeyAuth {
///     api_key: String,
/// }
///
/// impl Authentication for ApiKeyAuth {
///     fn username(&self) -> Option<String> { None }
///     fn password(&self) -> Option<String> { None }
///     fn as_header(&self) -> Option<(HeaderName, HeaderValue)> {
///         Some((
///             AUTHORIZATION,
///             HeaderValue::from_str(&format!("ApiKey {}", self.api_key)).unwrap()
///         ))
///     }
/// }
/// ```
pub trait Authentication: Debug {
    /// Returns the username if the authentication method uses one.
    fn username(&self) -> Option<String>;
    
    /// Returns the password if the authentication method uses one.
    fn password(&self) -> Option<String>;
    
    /// Returns the authentication header name and value.
    ///
    /// This method should return `None` if no authentication header
    /// should be added to the request, or `Some((name, value))` with
    /// the appropriate header name and value for authentication.
    fn as_header(&self) -> Option<(HeaderName, HeaderValue)>;
}

/// A type that represents no authentication.
///
/// This is useful for endpoints that don't require authentication
/// or when you want to explicitly indicate that no authentication
/// should be used.
///
/// # Examples
///
/// ```rust
/// use cdumay_http_client::{ClientBuilder, HttpClient};
/// use cdumay_http_client::authentication::NoAuth;
///
/// let client = HttpClient::new("https://api.example.com", None).unwrap()
///     .set_auth(NoAuth);
/// ```
#[derive(Debug)]
pub struct NoAuth;

impl Authentication for NoAuth {
    fn username(&self) -> Option<String> { None }
    fn password(&self) -> Option<String> { None }
    fn as_header(&self) -> Option<(HeaderName, HeaderValue)> { None }
}
