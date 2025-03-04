# cdumay_http_client

[![License: BSD-3-Clause](https://img.shields.io/badge/license-BSD--3--Clause-blue)](./LICENSE)
[![cdumay_http_client on crates.io](https://img.shields.io/crates/v/cdumay_http_client)](https://crates.io/crates/cdumay_http_client)
[![cdumay_http_client on docs.rs](https://docs.rs/cdumay_http_client/badge.svg)](https://docs.rs/cdumay_http_client)
[![Source Code Repository](https://img.shields.io/badge/Code-On%20GitHub-blue?logo=GitHub)](https://github.com/cdumay/cdumay_http_client)

A flexible and robust HTTP client library for Rust that provides both basic HTTP and REST client implementations.

## Features

- HTTP and REST client implementations
- Configurable timeout, headers, and SSL verification
- Authentication support
- Automatic retry mechanism
- Error handling with detailed context
- JSON serialization/deserialization for REST client
- Query parameters support
- Comprehensive logging

## Basic Usage

### HTTP Client

```rust
use cdumay_http_client::{ClientBuilder, HttpClient};
use cdumay_error::Result;

fn main() -> Result<()> {
    // Create a new HTTP client
    let client = HttpClient::new("https://dummyjson.com", None)?
        .set_timeout(30)  // Set timeout to 30 seconds
        .set_ssl_verify(true);

    // Make a GET request
    let response = client.get(
        "/users".to_string(),
        None,  // No query parameters
        None,  // No additional headers
        None,  // Use default timeout
        None,  // Use default retry behavior
        None,  // No context
    );

    println!("Response: {:?}", response);
    Ok(())
}
```

### REST Client

```rust
use cdumay_http_client::{ClientBuilder, RestClient};
use serde::{Deserialize, Serialize};
use cdumay_error::Result;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: i32,
    name: String,
}

fn create_user() -> Result<User> {
    // Create a new REST client
    let client = RestClient::new("https://dummyjson.com", None)?
        .set_timeout(30)
        .set_ssl_verify(true);

    // Make a GET request with automatic JSON deserialization
    let user: User = client.get(
        "/users/1".to_string(),
        None,
        None,
        None,
        None,
        None,
    )?;

    // Make a POST request with JSON payload
    let new_user = User {
        id: 0,
        name: "John Doe".to_string(),
    };

    let created: User = client.post(
        "/users".to_string(),
        None,
        Some(new_user),
        None,
        None,
        None,
        None,
    )?;
    Ok(created)
}

fn main() {
    println!("User creation result: {:?}", create_user());
}
```

## Authentication

The library supports custom authentication implementations through the `Authentication` trait:

```rust
use http::HeaderName;use cdumay_http_client::{HttpClient, ClientBuilder};
use cdumay_http_client::authentication::Authentication;
use reqwest::header::{HeaderMap, HeaderValue};

#[derive(Debug)]
struct BearerAuth {
    token: String,
}

impl Authentication for BearerAuth {
    fn username(&self) -> Option<String> { Some("token".into()) }
    fn password(&self) -> Option<String> { Some(self.token.clone()) }
    fn as_header(&self) -> Option<(HeaderName, HeaderValue)> { None }
}

// Using authentication with a client
let auth = BearerAuth { token: "your-token".to_string() };
let client = HttpClient::new("https://dummyjson.com", None).unwrap()
    .set_auth(auth);
```

## Error Handling

The library uses the `cdumay-error` crate for error handling, providing detailed context:

```rust
use cdumay_http_client::{HttpClient, ClientBuilder};
use cdumay_http_client::errors::http::{NotFound, Forbidden};
use cdumay_http_client::errors::http::{NOT_FOUND, FORBIDDEN};
use cdumay_error::Result;

// Specify error kinds to not retry on
let no_retry_on = vec![NOT_FOUND, FORBIDDEN];

let client = HttpClient::new("https://dummyjson.com", None).unwrap();

let result: Result<String> = client.get(
    "/users/1".to_string(),
    None,
    None,
    None,
    Some(no_retry_on),
    None,
);
```

## Retry Mechanism

Both clients support automatic retry with configurable attempts and delay:

```rust
use cdumay_http_client::{HttpClient, ClientBuilder};

let client = HttpClient::new("https://dummyjson.com", None).unwrap()
    .set_retry_number(3)    // Maximum 3 retry attempts
    .set_retry_delay(1);    // 1 second delay between retries
```
