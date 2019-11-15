use reqwest::header::{HeaderName, HeaderValue};

pub mod basic;

pub trait Authentication {
    fn username(&self) -> Option<String>;
    fn password(&self) -> Option<String>;
    fn as_header(&self) -> Option<(HeaderName, HeaderValue)>;
}


#[derive(Debug)]
pub struct NoAuth;


impl Authentication for NoAuth {
    fn username(&self) -> Option<String> { None }
    fn password(&self) -> Option<String> { None }
    fn as_header(&self) -> Option<(HeaderName, HeaderValue)> { None }
}
