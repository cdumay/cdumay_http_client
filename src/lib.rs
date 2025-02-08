#[macro_use]
extern crate log;

pub use client_http::{BaseClient, ClientBuilder, HttpClient};
pub use client_rest::RestClient;

pub mod authentication;
mod client_http;
mod client_rest;
pub mod errors;
mod utils;
