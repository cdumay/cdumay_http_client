use reqwest::{Method, Url};
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::clients::{Authentication, BaseClient, ClientBuilder};

#[derive(Debug)]
pub struct RestClient<A: Authentication> {
    url_root: Url,
    timeout: u64,
    headers: HeaderMap,
    auth: Option<A>,
}


impl<A: Authentication> ClientBuilder<A> for RestClient<A> {
    fn new(url_root: &str) -> Result<RestClient<A>> {
        Ok(RestClient {
            url_root: Url::parse(url_root.trim_end_matches("/"))?,
            timeout: 10,
            headers: {
                let mut headers = HeaderMap::new();
                headers.append(
                    USER_AGENT,
                    HeaderValue::from_str(&format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))).unwrap(),
                );
                headers.append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                headers.append(ACCEPT, HeaderValue::from_static("application/json"));
                headers
            },
            auth: None,
        })
    }
    fn set_timeout(mut self, timeout: u64) -> RestClient<A> {
        self.timeout = timeout;
        self
    }
    fn set_headers(mut self, headers: HeaderMap) -> RestClient<A> {
        self.headers.extend(headers);
        self
    }
    fn set_auth(mut self, auth: A) -> RestClient<A> {
        self.auth = Some(auth);
        self
    }
}

impl<A: Authentication> BaseClient<A> for RestClient<A> {
    fn url_root(&self) -> &Url { &self.url_root }
    fn timeout(&self) -> &u64 { &self.timeout }
    fn headers(&self) -> &HeaderMap { &self.headers }
    fn auth(&self) -> &Option<A> { &self.auth }
}

impl<A: Authentication> RestClient<A> {
    pub fn get<R>(&self, path: String, params: Option<HashMap<String, String>>, headers: Option<HeaderMap>, timeout: Option<u64>) -> Result<R>
        where for<'a> R: Deserialize<'a>
    {
        Ok(serde_json::from_str(&self.do_request(Method::GET, path, params, None, headers, timeout)?)?)
    }
    pub fn post<D, R>(&self, path: String, params: Option<HashMap<String, String>>, data: Option<D>, headers: Option<HeaderMap>, timeout: Option<u64>) -> Result<R>
        where for<'a> R: Deserialize<'a>, D: Serialize
    {
        let jdata = match data {
            Some(txt) => Some(serde_json::to_string(&txt)?),
            None => None
        };
        Ok(serde_json::from_str(&self.do_request(Method::POST, path, params, jdata, headers, timeout)?)?)
    }
    pub fn put<D, R>(&self, path: String, params: Option<HashMap<String, String>>, data: Option<D>, headers: Option<HeaderMap>, timeout: Option<u64>) -> Result<R>
        where for<'a> R: Deserialize<'a>, D: Serialize
    {
        let jdata = match data {
            Some(txt) => Some(serde_json::to_string(&txt)?),
            None => None
        };
        Ok(serde_json::from_str(&self.do_request(Method::PUT, path, params, jdata, headers, timeout)?)?)
    }
    pub fn delete<R>(&self, path: String, params: Option<HashMap<String, String>>, headers: Option<HeaderMap>, timeout: Option<u64>) -> Result<R>
        where for<'a> R: Deserialize<'a>
    {
        Ok(serde_json::from_str(&self.do_request(Method::DELETE, path, params, None, headers, timeout)?)?)
    }
}
