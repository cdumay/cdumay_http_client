use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use reqwest::{Client, Method, Url};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

use crate::authentication::Authentication;
use crate::ClientError;
use crate::utils::{build_url, merge_headers};

pub struct CallContext {
    pub start: DateTime<Utc>,
    pub url: Url,
    pub method: Method,
}

pub trait ClientBuilder<A: Authentication> {
    fn new(url_root: &str) -> Result<Self, ClientError> where Self: std::marker::Sized;
    fn set_timeout(self, timeout: u64) -> Self;
    fn set_headers(self, headers: HeaderMap) -> Self;
    fn set_auth(self, auth: A) -> Self;
}

pub trait BaseClient<A: Authentication> {
    // To implement
    fn url_root(&self) -> &Url;
    fn timeout(&self) -> &u64;
    fn headers(&self) -> &HeaderMap;
    fn auth(&self) -> &Option<A>;

    fn do_request(&self, method: Method, path: String, params: Option<HashMap<String, String>>, data: Option<String>, headers: Option<HeaderMap>, timeout: Option<u64>) -> Result<String, ClientError> {
        let start = chrono::Utc::now();
        let url = build_url(self.url_root(), path, params)?;
        let cli = Client::builder()
            .timeout(Duration::from_secs(timeout.unwrap_or(self.timeout().clone())))
            .default_headers(merge_headers(self.headers(), headers))
            .build()?;
        debug!("{} {}", &method, &url.as_str());
        let mut req = cli.request(method.clone(), url.clone());
        if let Some(auth) = self.auth() {
            if let Some((name, value)) = auth.as_header() {
                req = req.header(name, value);
            }
        }
        if let Some(txt) = data {
            req = req.body::<String>(txt);
        }
        let mut resp = req.send()?;
        let end = { chrono::Utc::now() - start }.to_std().unwrap();
        let human = humantime::format_duration(end).to_string();
        let lenght = resp.content_length().unwrap_or(0);
        match resp.status().is_success() {
            true => {
                info!("{} {} - {} - {} [{}]", &method, &url.as_str(), resp.status(), lenght, &human);
                Ok(resp.text()?)
            }
            false => {
                error!("{} {} - {} - {} [{}]", &method, &url.as_str(), resp.status(), lenght, &human);
                Err(ClientError::from(&mut resp))
            }
        }
    }
    fn head(&self, path: String, params: Option<HashMap<String, String>>, headers: Option<HeaderMap>, timeout: Option<u64>) -> Result<(), ClientError> {
        self.do_request(Method::HEAD, path, params, None, headers, timeout)?;
        Ok(())
    }
}


#[derive(Debug)]
pub struct HttpClient<A: Authentication> {
    url_root: Url,
    timeout: u64,
    headers: HeaderMap,
    auth: Option<A>,
}


impl<A: Authentication> ClientBuilder<A> for HttpClient<A> {
    fn new(url_root: &str) -> Result<HttpClient<A>, ClientError> {
        Ok(HttpClient {
            url_root: Url::parse(url_root.trim_end_matches("/"))?,
            timeout: 10,
            headers: {
                let mut headers = HeaderMap::new();
                headers.append(
                    USER_AGENT,
                    HeaderValue::from_str(&format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")))?,
                );
                headers
            },
            auth: None,
        })
    }
    fn set_timeout(mut self, timeout: u64) -> HttpClient<A> {
        self.timeout = timeout;
        self
    }
    fn set_headers(mut self, headers: HeaderMap) -> HttpClient<A> {
        self.headers.extend(headers);
        self
    }
    fn set_auth(mut self, auth: A) -> HttpClient<A> {
        self.auth = Some(auth);
        self
    }
}

impl<A: Authentication> BaseClient<A> for HttpClient<A> {
    fn url_root(&self) -> &Url { &self.url_root }
    fn timeout(&self) -> &u64 { &self.timeout }
    fn headers(&self) -> &HeaderMap { &self.headers }
    fn auth(&self) -> &Option<A> { &self.auth }
}

impl<A: Authentication> HttpClient<A> {
    pub fn get(&self, path: String, params: Option<HashMap<String, String>>, headers: Option<HeaderMap>, timeout: Option<u64>) -> Result<String, ClientError> {
        self.do_request(Method::GET, path, params, None, headers, timeout)
    }
    pub fn post(&self, path: String, params: Option<HashMap<String, String>>, data: Option<String>, headers: Option<HeaderMap>, timeout: Option<u64>) -> Result<String, ClientError> {
        self.do_request(Method::POST, path, params, data, headers, timeout)
    }
    pub fn put(&self, path: String, params: Option<HashMap<String, String>>, data: Option<String>, headers: Option<HeaderMap>, timeout: Option<u64>) -> Result<String, ClientError> {
        self.do_request(Method::PUT, path, params, data, headers, timeout)
    }
    pub fn delete(&self, path: String, params: Option<HashMap<String, String>>, headers: Option<HeaderMap>, timeout: Option<u64>) -> Result<String, ClientError> {
        self.do_request(Method::DELETE, path, params, None, headers, timeout)
    }
}

