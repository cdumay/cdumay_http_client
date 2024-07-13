use std::collections::{BTreeMap, HashMap};
use std::thread;
use std::time::Duration;

use cdumay_error::{Error, ErrorBuilder, ErrorKind, GenericErrors};
use chrono::{DateTime, Utc};
use reqwest::{Method, Url};
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::Value;

use crate::authentication::Authentication;
use crate::ClientError;
use crate::utils::{build_url, merge_headers};

pub struct CallContext {
    pub start: DateTime<Utc>,
    pub url: Url,
    pub method: Method,
}

pub trait ClientBuilder {
    fn new(url_root: &str) -> Result<Self, ClientError>
    where
        Self: Sized;
    fn set_timeout(self, timeout: u64) -> Self;
    fn set_headers(self, headers: HeaderMap) -> Self;
    fn set_auth<A: Authentication + 'static>(self, auth: A) -> Self;
    fn set_try_number(self, try_number: u64) -> Self;
    fn set_retry_delay(self, try_number: u64) -> Self;
}

pub trait BaseClient {
    type Output;

    // To implement
    fn url_root(&self) -> &Url;
    fn timeout(&self) -> &u64;
    fn headers(&self) -> &HeaderMap;
    fn auth(&self) -> Option<&Box<dyn Authentication>>;
    fn try_number(&self) -> u64;
    fn retry_delay(&self) -> u64;
    fn _parse_response(&self, response: Response, context: BTreeMap<String, Value>) -> Result<Self::Output, Error>;
    fn _request_wrapper(&self, req: RequestBuilder) -> Result<Response, Error> {
        Ok(req.send().map_err(|err| ErrorBuilder::from(ClientError::from(err)).build())?)
    }
    fn do_request(&self, method: Method, path: String, params: Option<HashMap<String, String>>, data: Option<String>, headers: Option<HeaderMap>, timeout: Option<u64>, no_retry_on: Option<Vec<ErrorKind>>) -> Result<Self::Output, Error> {
        let start = Utc::now();
        let url = build_url(self.url_root(), path, params).map_err(|err| ErrorBuilder::from(err).build())?;
        let context = {
            let mut out: BTreeMap<String, Value> = BTreeMap::new();
            out.insert("url".into(), Value::from(url.to_string()));
            out.insert("method".into(), Value::from(method.to_string()));
            out
        };
        let cli = Client::builder()
            .timeout(Duration::from_secs(timeout.unwrap_or(self.timeout().clone())))
            .default_headers(merge_headers(self.headers(), headers))
            .build().map_err(|err| ErrorBuilder::from(ClientError::from(err)).extra(context.clone().into()).build())?;
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
        for req_try in 1..=self.try_number() {
            info!("[{}] - {} (try: {})", method, url, req_try);
            match req.try_clone() {
                Some(req) => {
                    let resp = self._request_wrapper(req)?;
                    let end = { Utc::now() - start }.to_std().unwrap();
                    let human = humantime::format_duration(end).to_string();
                    let length = resp.content_length().unwrap_or(0);
                    match resp.status().is_success() {
                        true => {
                            info!("{} {} - {} - {} [{}]", &method, &url.as_str(), resp.status(), length, &human);
                            return Ok(self._parse_response(resp, context.clone().into())?);
                        }
                        false => {
                            error!("{} {} - {} - {} [{}]", &method, &url.as_str(), resp.status(), length, &human);
                            let mut err_context = context.clone();
                            err_context.insert("try".into(), Value::from(req_try));
                            let err = ErrorBuilder::from(ClientError::from(resp)).extra(err_context.into()).build();
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
                    return Err(
                        ErrorBuilder::from(GenericErrors::UNKNOWN_ERROR)
                            .message("Internal error, failed to clone request".into())
                            .extra(context.into()).build()
                    )
                }
            }
        }
        match last_error {
            Some(err) => {
                error!("Failed to perform request {} on {} after {} retries : {}", method, url,self.try_number() ,err);
                Err(err)
            }
            None => {
                error!("Unexpected error, failed to perform request {} on {} after {} retries", method, url,self.try_number());
                Err(ErrorBuilder::from(GenericErrors::UNKNOWN_ERROR)
                    .extra(context.into()).build()
                )
            }
        }
    }
    fn head(&self, path: String, params: Option<HashMap<String, String>>, headers: Option<HeaderMap>, timeout: Option<u64>, no_retry_on: Option<Vec<ErrorKind>>) -> Result<(), Error> {
        self.do_request(Method::HEAD, path, params, None, headers, timeout, no_retry_on)?;
        Ok(())
    }
    fn get(&self, path: String, params: Option<HashMap<String, String>>, headers: Option<HeaderMap>, timeout: Option<u64>, no_retry_on: Option<Vec<ErrorKind>>) -> Result<Self::Output, Error> {
        self.do_request(Method::GET, path, params, None, headers, timeout, no_retry_on)
    }
    fn post(&self, path: String, params: Option<HashMap<String, String>>, data: Option<String>, headers: Option<HeaderMap>, timeout: Option<u64>, no_retry_on: Option<Vec<ErrorKind>>) -> Result<Self::Output, Error> {
        self.do_request(Method::POST, path, params, data, headers, timeout, no_retry_on)
    }
    fn put(&self, path: String, params: Option<HashMap<String, String>>, data: Option<String>, headers: Option<HeaderMap>, timeout: Option<u64>, no_retry_on: Option<Vec<ErrorKind>>) -> Result<Self::Output, Error> {
        self.do_request(Method::PUT, path, params, data, headers, timeout, no_retry_on)
    }
    fn delete(&self, path: String, params: Option<HashMap<String, String>>, headers: Option<HeaderMap>, timeout: Option<u64>, no_retry_on: Option<Vec<ErrorKind>>) -> Result<Self::Output, Error> {
        self.do_request(Method::DELETE, path, params, None, headers, timeout, no_retry_on)
    }
}


#[derive(Debug)]
pub struct HttpClient {
    url_root: Url,
    timeout: u64,
    headers: HeaderMap,
    auth: Option<Box<dyn Authentication>>,
    try_number: u64,
    retry_delay: u64,
}


impl ClientBuilder for HttpClient {
    fn new(url_root: &str) -> Result<HttpClient, ClientError> {
        Ok(HttpClient {
            url_root: Url::parse(url_root.trim_end_matches("/")).map_err(|err| {
                ClientError::UrlError(err)
            })?,
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
            try_number: 10,
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
    fn set_try_number(mut self, try_number: u64) -> HttpClient {
        if try_number == 0 {
            panic!("Try number MUST be > 0 !");
        }
        self.try_number = try_number;
        self
    }
    fn set_retry_delay(mut self, retry_delay: u64) -> HttpClient {
        self.retry_delay = retry_delay;
        self
    }
}

impl BaseClient for HttpClient {
    type Output = String;
    fn url_root(&self) -> &Url { &self.url_root }
    fn timeout(&self) -> &u64 { &self.timeout }
    fn headers(&self) -> &HeaderMap { &self.headers }
    fn auth(&self) -> Option<&Box<dyn Authentication>> { self.auth.as_ref() }
    fn try_number(&self) -> u64 { self.try_number }
    fn retry_delay(&self) -> u64 { self.retry_delay }
    fn _parse_response(&self, response: Response, context: BTreeMap<String, Value>) -> Result<Self::Output, Error> {
        response
            .text()
            .map_err(
                |err| ErrorBuilder::from(ClientError::from(err)).extra(context.clone().into()).build()
            )
    }
}

#[cfg(test)]
mod test {
    use std::sync::Once;
    use simple_logger::SimpleLogger;

    use crate::{BaseClient, ClientBuilder, HttpClient, HttpStatusCodeErrors};

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            let _ = SimpleLogger::new().with_level(log::LevelFilter::Info).init();
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
            .set_try_number(2)
            .set_retry_delay(1);
        match cli.get("/sdq".into(), None, None, None, None) {
            Ok(_) => panic!("No error raised!"),
            Err(err) => assert_eq!(err.kind, HttpStatusCodeErrors::UNPROCESSABLE_ENTITY)
        };
    }
    #[test]
    fn test_err_no_retry() {
        init_logger();
        let cli = HttpClient::new("https://www.rust-lang.org")
            .unwrap()
            .set_try_number(2)
            .set_retry_delay(1);
        match cli.get("/sdq".into(), None, None, None, Some(vec![HttpStatusCodeErrors::UNPROCESSABLE_ENTITY])) {
            Ok(_) => panic!("No error raised!"),
            Err(err) => {
                assert_eq!(err.kind, HttpStatusCodeErrors::UNPROCESSABLE_ENTITY);
                match err.extra {
                    Some(data) => {
                        match data.get("try") {
                            Some(value) => {
                                assert_eq!(value.is_u64(), true);
                                assert_eq!(value.as_u64().unwrap(), 1);
                            }
                            None => panic!("No try in error.extra")
                        };
                    }
                    None => panic!("Not error extra found !")
                }
            }
        };
    }
}