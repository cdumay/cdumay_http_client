/*!
# HTTP Client Utilities

This module provides utility functions for HTTP client operations, including URL building
and header manipulation. These utilities are used internally by the HTTP client but
can also be used directly when needed.

## Features

- URL construction with path and query parameters
- HTTP header merging
- Error handling for URL operations

## Examples

### URL Building

```rust
use cdumay_http_client::build_url;
use reqwest::Url;
use std::collections::HashMap;

// Create base URL
let root = Url::parse("https://api.example.com").unwrap();

// Add path and query parameters
let mut params = HashMap::new();
params.insert("page".to_string(), "1".to_string());
params.insert("limit".to_string(), "10".to_string());

let url = build_url(
    &root,
    "/users/search".to_string(),
    Some(params)
).unwrap();

assert_eq!(
    url.as_str(),
    "https://api.example.com/users/search?limit=10&page=1"
);
```

### Header Merging

```rust
use cdumay_http_client::merge_headers;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION};

// Create base headers
let mut base_headers = HeaderMap::new();
base_headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

// Create additional headers
let mut additional_headers = HeaderMap::new();
additional_headers.insert(
    AUTHORIZATION,
    HeaderValue::from_static("Bearer token123")
);

// Merge headers
let merged = merge_headers(&base_headers, Some(additional_headers));

assert_eq!(
    merged.get(ACCEPT).unwrap(),
    "application/json"
);
assert_eq!(
    merged.get(AUTHORIZATION).unwrap(),
    "Bearer token123"
);
```
*/

use crate::errors::client::InvalidUrl;
use cdumay_error::Result;
use reqwest::header::HeaderMap;
use reqwest::Url;
use std::collections::HashMap;

/// Merges two sets of HTTP headers.
///
/// This function takes a base set of headers and optionally additional headers,
/// combining them into a single HeaderMap. If a header exists in both maps,
/// the value from the additional headers will override the base value.
///
/// # Arguments
///
/// * `h1` - Base headers that will be used as the foundation
/// * `h2` - Optional additional headers that will override any duplicate headers from h1
///
/// # Returns
///
/// Returns a new `HeaderMap` containing all headers from both maps, with values
/// from `h2` taking precedence over values from `h1` for any duplicate headers.
///
/// # Examples
///
/// ```rust
/// use cdumay_http_client::merge_headers;
/// use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
///
/// let mut base = HeaderMap::new();
/// base.insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
///
/// let mut additional = HeaderMap::new();
/// additional.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
///
/// let merged = merge_headers(&base, Some(additional));
/// assert_eq!(
///     merged.get(CONTENT_TYPE).unwrap(),
///     "application/json"
/// );
/// ```
pub fn merge_headers(h1: &HeaderMap, h2: Option<HeaderMap>) -> HeaderMap {
    let mut headers = h1.clone();
    if let Some(additional_headers) = h2 {
        headers.extend(additional_headers);
    }
    headers
}

/// Builds a complete URL from a root URL, path, and optional query parameters.
///
/// This function constructs a URL by:
/// 1. Starting with the root URL
/// 2. Appending the path components
/// 3. Adding query parameters if provided
///
/// # Arguments
///
/// * `root` - Base URL to build upon
/// * `path` - Path to append to the root URL (can include multiple segments separated by '/')
/// * `params` - Optional query parameters to add to the URL
///
/// # Returns
///
/// Returns a `Result<Url>` which is:
/// - `Ok(Url)` containing the constructed URL if successful
/// - `Err` with an `InvalidUrl` error if URL construction fails
///
/// # Examples
///
/// ```rust
/// use cdumay_http_client::build_url;
/// use reqwest::Url;
/// use std::collections::HashMap;
///
/// // Basic URL with path
/// let root = Url::parse("https://api.example.com").unwrap();
/// let url = build_url(
///     &root,
///     "/users".to_string(),
///     None
/// ).unwrap();
/// assert_eq!(url.as_str(), "https://api.example.com/users");
///
/// // URL with path and query parameters
/// let mut params = HashMap::new();
/// params.insert("search".to_string(), "john".to_string());
/// params.insert("sort".to_string(), "name".to_string());
///
/// let url = build_url(
///     &root,
///     "/users/search".to_string(),
///     Some(params)
/// ).unwrap();
/// assert_eq!(
///     url.as_str(),
///     "https://api.example.com/users/search?search=john&sort=name"
/// );
/// ```
pub fn build_url(root: &Url, path: String, params: Option<HashMap<String, String>>) -> Result<Url> {
    let mut url = root.clone();
    let spath: Vec<&str> = path.split("/").filter(|part| part.len() != 0).collect();
    url.path_segments_mut()
        .map_err(|_| InvalidUrl::new().set_message("Cannot build url".to_string()))?
        .extend(&spath);

    if let Some(data) = params {
        let mut sorted_entries: Vec<_> = data.iter().collect();
        sorted_entries.sort_by_key(|&(k, _)| k);
        for (key, value) in sorted_entries {
            url.query_pairs_mut().append_pair(key, value);
        }
    }
    Ok(url)
}
