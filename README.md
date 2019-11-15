# cdumay_http_client

[![Build Status](https://travis-ci.org/cdumay/rust-cdumay_http_client.svg?branch=master)](https://travis-ci.org/cdumay/rust-cdumay_http_client)
[![Latest version](https://img.shields.io/crates/v/cdumay_http_client.svg)](https://crates.io/crates/cdumay_http_client)
[![Documentation](https://docs.rs/cdumay_http_client/badge.svg)](https://docs.rs/cdumay_http_client)
![License](https://img.shields.io/crates/l/cdumay_http_client.svg)

cdumay_http_client is a basic library used to standardize result and serialize them using [serde](https://docs.serde.rs/serde/).

## Quickstart

_Cargo.toml_:
```toml
[dependencies]
cdumay_error = "0.1"
cdumay_result = "1.0"
```

_main.rs_:

```rust
extern crate cdumay_error;
extern crate cdumay_http_client;

use cdumay_error::ErrorRepr;
use cdumay_http_client::authentication::NoAuth;
use cdumay_http_client::{ClientBuilder, HttpClient};

fn main() {
    let cli = HttpClient::<NoAuth>::new("https://www.rust-lang.org").unwrap();
    let result = cli.get("/learn/get-started".into(), None, None, None);

    match result {
        Ok(data) => println!("{}", data),
        Err(err) => println!("{}", serde_json::to_string_pretty(&ErrorRepr::from(err)).unwrap()),
    }
```
_Output_:
```html
<!doctype html>
<html lang="en-US">
  <head>
    <meta charset="utf-8">
    <title>
[...]
```
## Errors

Errors can be displayed using [cdumay_error](https://docs.serde.rs/cdumay_error/):

```json
{
  "code": 500,
  "message": "error trying to connect",
  "msgid": "Err-05192"
}
```

## Project Links

- Issues: https://github.com/cdumay/rust-cdumay_http_client/issues
- Documentation: https://docs.rs/cdumay_http_client

License: MIT
