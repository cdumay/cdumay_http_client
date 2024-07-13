# cdumay_http_client ![License: BSD-3-Clause](https://img.shields.io/badge/license-BSD--3--Clause-blue) [![cdumay_http_client on crates.io](https://img.shields.io/crates/v/cdumay_http_client)](https://crates.io/crates/cdumay_http_client) [![cdumay_http_client on docs.rs](https://docs.rs/cdumay_http_client/badge.svg)](https://docs.rs/cdumay_http_client) [![Source Code Repository](https://img.shields.io/badge/Code-On%20GitHub-blue?logo=GitHub)](https://github.com/cdumay/rust-cdumay_http_client)

cdumay_http_client is a basic library used to standardize result and serialize them using [serde][__link0].

### Quickstart

*Cargo.toml*:

```toml
[dependencies]
cdumay_error = "0.3"
cdumay_result = "0.3"
cdumay_http_client = "0.3"
```

*main.rs*:

```rust
extern crate cdumay_error;
extern crate cdumay_http_client;
extern crate serde_json;

use cdumay_error::JsonError;
use cdumay_http_client::authentication::NoAuth;
use cdumay_http_client::{ClientBuilder, HttpClient};

fn main() {
    use cdumay_http_client::BaseClient;
let cli = HttpClient::new("https://www.rust-lang.org").unwrap();
    let result = cli.get("/learn/get-started".into(), None, None, None, None);

    match result {
        Ok(data) => println!("{}", data),
        Err(err) => println!("{}", serde_json::to_string_pretty(&JsonError::from(err)).unwrap()),
    }
}
```

*Output*:

```html
<!doctype html>
<html lang="en-US">
  <head>
    <meta charset="utf-8">
    <title>
[...]
```

### Errors

Errors can be displayed using [cdumay_error][__link1]:

```json
{
  "code": 500,
  "message": "error trying to connect",
  "msgid": "Err-05192"
}
```


 [__link0]: https://docs.serde.rs/serde/
 [__link1]: https://docs.serde.rs/cdumay_error/
