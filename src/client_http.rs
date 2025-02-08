use cdumay_context::Context;
use cdumay_error::{Error, ErrorKind, Result};
use chrono::Utc;
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Method, Url};
use serde_value::Value;
use std::collections::{BTreeMap, HashMap};
use std::thread;
use std::time::Duration;

use crate::authentication::Authentication;
use crate::errors::client::{ClientBuilderError, InvalidHeaderValue, InvalidUrl};
use crate::errors::{http_error_serialize, http_resp_serialise};
use crate::utils::{build_url, merge_headers};

pub trait ClientBuilder {
    fn new(url_root: &str) -> Result<Self>
    where
        Self: Sized;
    fn set_timeout(self, timeout: u64) -> Self;
    fn set_headers(self, headers: HeaderMap) -> Self;
    fn set_auth<A: Authentication + 'static>(self, auth: A) -> Self;
    fn set_ssl_verify(self, ssl_verify: bool) -> Self;
    fn set_retry_number(self, try_number: u64) -> Self;
    fn set_retry_delay(self, try_number: u64) -> Self;
}

pub trait BaseClient {
    // To implement
    fn url_root(&self) -> &Url;
    fn timeout(&self) -> &u64;
    fn headers(&self) -> &HeaderMap;
    fn auth(&self) -> Option<&Box<dyn Authentication>>;
    fn ssl_verify(&self) -> bool;
    fn retry_number(&self) -> u64;
    fn retry_delay(&self) -> u64;
    fn _request_wrapper(&self, req: RequestBuilder) -> Result<Response> {
        Ok(req.send().map_err(|err| http_error_serialize(&err, None))?)
    }
    fn do_request(
        &self,
        method: Method,
        path: String,
        params: Option<HashMap<String, String>>,
        data: Option<String>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
    ) -> Result<String> {
        let start = Utc::now();
        let url = build_url(self.url_root(), path, params)?;
        let mut context = Context::default();
        context.insert("url".into(), Value::String(url.to_string()));
        context.insert("method".into(), Value::String(method.to_string()));
        let cli = Client::builder()
            .timeout(Duration::from_secs(
                timeout.unwrap_or(self.timeout().clone()),
            ))
            .default_headers(merge_headers(self.headers(), headers))
            .build()
            .map_err(|err| http_error_serialize(&err, Some(context.clone().into())))?;

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
        let mut last_error: Option<Error> = None;
        for req_try in 1..=self.retry_number() {
            info!("[{}] - {} (try: {})", method, url, req_try);
            match req.try_clone() {
                Some(req) => {
                    let resp = self._request_wrapper(req)?;
                    let end = { Utc::now() - start }.to_std().unwrap();
                    let human = humantime::format_duration(end).to_string();
                    let length = resp.content_length().unwrap_or(0);
                    match resp.status().is_success() {
                        true => {
                            info!(
                                "{} {} - {} - {} [{}]",
                                &method,
                                &url.as_str(),
                                resp.status(),
                                length,
                                &human
                            );
                            return Ok(resp.text().map_err(|err| {
                                http_error_serialize(&err, Some(context.into()))
                            })?);
                        }
                        false => {
                            error!(
                                "{} {} - {} - {} [{}]",
                                &method,
                                &url.as_str(),
                                resp.status(),
                                length,
                                &human
                            );
                            let mut err_context = context.clone();
                            err_context.insert("try".into(), Value::U64(req_try));
                            let err = http_resp_serialise(resp, Some(err_context));
                            if let Some(kinds) = &no_retry_on {
                                if kinds.contains(&err.kind) {
                                    return Err(err);
                                }
                            }
                            last_error = Some(err);
                        }
                    };
                    thread::sleep(Duration::from_secs(self.retry_delay()));
                }
                None => {
                    return Err(ClientBuilderError::new()
                        .set_message("Internal error, failed to clone request".into())
                        .set_details(context.into())
                        .into())
                }
            }
        }
        match last_error {
            Some(err) => {
                error!(
                    "Failed to perform request {} on {} after {} retries : {}",
                    method,
                    url,
                    self.retry_number(),
                    err
                );
                Err(err)
            }
            None => {
                error!(
                    "Unexpected error, failed to perform request {} on {} after {} retries",
                    method,
                    url,
                    self.retry_number()
                );
                Err(ClientBuilderError::new()
                    .set_message("Internal error, failed to clone request".into())
                    .set_details(context.into())
                    .into())
            }
        }
    }
    fn head(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
    ) -> Result<()> {
        self.do_request(
            Method::HEAD,
            path,
            params,
            None,
            headers,
            timeout,
            no_retry_on,
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct HttpClient {
    url_root: Url,
    timeout: u64,
    headers: HeaderMap,
    auth: Option<Box<dyn Authentication>>,
    ssl_verify: bool,
    retry_number: u64,
    retry_delay: u64,
}

impl ClientBuilder for HttpClient {
    fn new(url_root: &str) -> Result<HttpClient> {
        Ok(HttpClient {
            url_root: Url::parse(url_root.trim_end_matches("/")).map_err(|err| {
                InvalidUrl::new()
                    .set_message(format!("Failed to parse URL: {:?}", err))
                    .set_details({
                        let mut context = BTreeMap::new();
                        context.insert("url".to_string(), Value::String(url_root.to_string()));
                        context
                    })
            })?,
            timeout: 10,
            headers: {
                let mut headers = HeaderMap::new();
                headers.append(
                    USER_AGENT,
                    HeaderValue::from_str(&format!(
                        "{}/{}",
                        env!("CARGO_PKG_NAME"),
                        env!("CARGO_PKG_VERSION")
                    ))
                    .map_err(|err| InvalidHeaderValue::new().set_message(err.to_string()))?,
                );
                headers
            },
            auth: None,
            ssl_verify: true,
            retry_number: 10,
            retry_delay: 30,
        })
    }
    fn set_timeout(mut self, timeout: u64) -> HttpClient {
        self.timeout = timeout;
        self
    }
    fn set_headers(mut self, headers: HeaderMap) -> HttpClient {
        self.headers.extend(headers);
        self
    }
    fn set_auth<A: Authentication + 'static>(mut self, auth: A) -> HttpClient {
        self.auth = Some(Box::new(auth));
        self
    }
    fn set_ssl_verify(mut self, ssl_verify: bool) -> HttpClient {
        self.ssl_verify = ssl_verify;
        self
    }
    fn set_retry_number(mut self, retry_number: u64) -> HttpClient {
        if retry_number == 0 {
            panic!("Try number MUST be > 0 !");
        }
        self.retry_number = retry_number;
        self
    }
    fn set_retry_delay(mut self, retry_delay: u64) -> HttpClient {
        self.retry_delay = retry_delay;
        self
    }
}

impl BaseClient for HttpClient {
    fn url_root(&self) -> &Url {
        &self.url_root
    }
    fn timeout(&self) -> &u64 {
        &self.timeout
    }
    fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    fn auth(&self) -> Option<&Box<dyn Authentication>> {
        self.auth.as_ref()
    }
    fn ssl_verify(&self) -> bool {
        self.ssl_verify
    }
    fn retry_number(&self) -> u64 {
        self.retry_number
    }
    fn retry_delay(&self) -> u64 {
        self.retry_delay
    }
}

impl HttpClient {
    pub fn get(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
    ) -> Result<String> {
        self.do_request(
            Method::GET,
            path,
            params,
            None,
            headers,
            timeout,
            no_retry_on,
        )
    }
    pub fn post(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        data: Option<String>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
    ) -> Result<String> {
        self.do_request(
            Method::POST,
            path,
            params,
            data,
            headers,
            timeout,
            no_retry_on,
        )
    }
    pub fn put(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        data: Option<String>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
    ) -> Result<String> {
        self.do_request(
            Method::PUT,
            path,
            params,
            data,
            headers,
            timeout,
            no_retry_on,
        )
    }
    pub fn delete(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
    ) -> Result<String> {
        self.do_request(
            Method::DELETE,
            path,
            params,
            None,
            headers,
            timeout,
            no_retry_on,
        )
    }
}

#[cfg(test)]
mod test {
    use std::sync::Once;

    use simple_logger::SimpleLogger;

    use crate::errors::http::UNPROCESSABLE_ENTITY;
    use crate::{ClientBuilder, HttpClient};

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            let _ = SimpleLogger::new()
                .with_level(log::LevelFilter::Info)
                .init();
        });
    }

    #[test]
    fn test_no_auth() {
        init_logger();
        let cli = HttpClient::new("https://www.rust-lang.org").unwrap();
        let result = cli.get("/learn/get-started".into(), None, None, None, None);
        assert_eq!(result.unwrap().starts_with("<!doctype html>"), true);
    }

    #[test]
    fn test_err() {
        init_logger();
        let cli = HttpClient::new("https://www.rust-lang.org")
            .unwrap()
            .set_retry_number(2)
            .set_retry_delay(1);
        match cli.get("/sdq".into(), None, None, None, None) {
            Ok(_) => panic!("No error raised!"),
            Err(err) => assert_eq!(err.kind, UNPROCESSABLE_ENTITY),
        };
    }
    #[test]
    fn test_err_no_retry() {
        init_logger();
        let cli = HttpClient::new("https://www.rust-lang.org")
            .unwrap()
            .set_retry_number(2)
            .set_retry_delay(1);
        match cli.get(
            "/sdq".into(),
            None,
            None,
            None,
            Some(vec![UNPROCESSABLE_ENTITY]),
        ) {
            Ok(_) => panic!("No error raised!"),
            Err(err) => {
                assert_eq!(err.kind, UNPROCESSABLE_ENTITY);
                match err.details {
                    Some(data) => {
                        match data.get("try") {
                            Some(value) => {
                                assert_eq!(serde_value::Value::U64(1), value.clone());
                            }
                            None => panic!("No try in error.extra"),
                        };
                    }
                    None => panic!("Not error extra found !"),
                }
            }
        };
    }
}
