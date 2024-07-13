use base64::prelude::*;
use reqwest::header::{AUTHORIZATION, HeaderName, HeaderValue};

use crate::authentication::Authentication;

#[derive(Debug)]
pub struct BasicAuth {
    username: String,
    password: Option<String>,
}


impl BasicAuth {
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
    fn as_header(&self) -> Option<(HeaderName, HeaderValue)> {
        let auth = match self.password() {
            Some(password) => format!("{}:{}", self.username, password),
            None => format!("{}:", self.username)
        };
        let header_value = format!("Basic {}", BASE64_STANDARD.encode(&auth));
        Some((AUTHORIZATION, HeaderValue::from_str(&*header_value).unwrap()))
    }
}
