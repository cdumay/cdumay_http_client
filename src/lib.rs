//! cdumay_http_client is a basic library used to standardize result and serialize them using [serde](https://docs.serde.rs/serde/).
//!
//! ## Quickstart
//!
//! _Cargo.toml_:
//! ```toml
//! [dependencies]
//! cdumay_error = "0.3"
//! cdumay_result = "0.3"
//! cdumay_http_client = "0.3"
//! ```
//!
//! _main.rs_:
//!
//! ```rust
//! extern crate cdumay_error;
//! extern crate cdumay_http_client;
//! extern crate serde_json;
//!
//! use cdumay_error::JsonError;
//! use cdumay_http_client::authentication::NoAuth;
//! use cdumay_http_client::{ClientBuilder, HttpClient};
//!
//! fn main() {
//!     use cdumay_http_client::BaseClient;
//! let cli = HttpClient::new("https://www.rust-lang.org").unwrap();
//!     let result = cli.get("/learn/get-started".into(), None, None, None, None);
//!
//!     match result {
//!         Ok(data) => println!("{}", data),
//!         Err(err) => println!("{}", serde_json::to_string_pretty(&JsonError::from(err)).unwrap()),
//!     }
//! }
//! ```
//! _Output_:
//! ```html
//! <!doctype html>
//! <html lang="en-US">
//!   <head>
//!     <meta charset="utf-8">
//!     <title>
//! [...]
//! ```
//! ## Errors
//!
//! Errors can be displayed using [cdumay_error](https://docs.serde.rs/cdumay_error/):
//!
//! ```json
//! {
//!   "code": 500,
//!   "message": "error trying to connect",
//!   "msgid": "Err-05192"
//! }
//! ```
extern crate cdumay_error;
extern crate cdumay_result;
extern crate chrono;
extern crate http;
extern crate humantime;
#[macro_use]
extern crate log;
extern crate reqwest;
extern crate serde;
extern crate base64;
extern crate serde_json;

pub use client::{BaseClient, CallContext, ClientBuilder, HttpClient};
pub use errors::{ClientError, HttpStatusCodeErrors, ResponseErrorWithContext};

mod utils;
mod client;
pub mod authentication;
mod errors;