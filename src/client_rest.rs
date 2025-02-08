use crate::authentication::Authentication;
use crate::errors::client::{InvalidHeaderValue, InvalidUrl};
use crate::errors::rest::json_error_serialize;
use crate::{BaseClient, ClientBuilder};
use cdumay_context::Context;
use cdumay_error::{ErrorKind, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, USER_AGENT};
use reqwest::{Method, Url};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;

#[derive(Debug)]
pub struct RestClient {
    url_root: Url,
    timeout: u64,
    headers: HeaderMap,
    auth: Option<Box<dyn Authentication>>,
    ssl_verify: bool,
    retry_number: u64,
    retry_delay: u64,
}

impl ClientBuilder for RestClient {
    fn new(url_root: &str) -> Result<RestClient> {
        Ok(RestClient {
            url_root: Url::parse(url_root.trim_end_matches("/")).map_err(|err| {
                InvalidUrl::new()
                    .set_message(format!("Failed to parse URL: {:?}", err))
                    .set_details({
                        let mut context = BTreeMap::new();
                        context.insert(
                            "url".to_string(),
                            serde_value::Value::String(url_root.to_string()),
                        );
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
                headers.append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                headers.append(ACCEPT, HeaderValue::from_static("application/json"));
                headers
            },
            auth: None,
            ssl_verify: true,
            retry_number: 10,
            retry_delay: 30,
        })
    }
    fn set_timeout(mut self, timeout: u64) -> RestClient {
        self.timeout = timeout;
        self
    }
    fn set_headers(mut self, headers: HeaderMap) -> RestClient {
        self.headers.extend(headers);
        self
    }
    fn set_auth<A: Authentication + 'static>(mut self, auth: A) -> RestClient {
        self.auth = Some(Box::new(auth));
        self
    }
    fn set_ssl_verify(mut self, ssl_verify: bool) -> RestClient {
        self.ssl_verify = ssl_verify;
        self
    }
    fn set_retry_number(mut self, try_number: u64) -> RestClient {
        if try_number == 0 {
            panic!("Try number MUST be > 0 !");
        }
        self.retry_number = try_number;
        self
    }
    fn set_retry_delay(mut self, retry_delay: u64) -> RestClient {
        self.retry_delay = retry_delay;
        self
    }
}

impl BaseClient for RestClient {
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

impl RestClient {
    fn create_context(&self, path: String, method: Method) -> Context {
        let mut context = Context::default();
        context.insert(
            "server".into(),
            serde_value::Value::String(self.url_root.to_string()),
        );
        context.insert("path".into(), serde_value::Value::String(path));
        context.insert(
            "method".into(),
            serde_value::Value::String(method.to_string()),
        );
        context
    }
    pub fn get<R>(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
    ) -> Result<R>
    where
        for<'a> R: Deserialize<'a>,
    {
        Ok(serde_json::from_str(&self.do_request(
            Method::GET,
            path.to_string(),
            params,
            None,
            headers,
            timeout,
            no_retry_on,
        )?)
        .map_err(|err| json_error_serialize(err, Some(self.create_context(path, Method::GET))))?)
    }
    pub fn post<D, R>(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        data: Option<D>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
    ) -> Result<R>
    where
        for<'a> R: Deserialize<'a>,
        D: Serialize + Debug,
    {
        let payload = match data {
            Some(txt) => Some(serde_json::to_string(&txt).map_err(|err| {
                json_error_serialize(err, Some(self.create_context(path.clone(), Method::POST)))
            })?),
            None => None,
        };
        Ok(serde_json::from_str(&self.do_request(
            Method::POST,
            path.to_string(),
            params,
            payload,
            headers,
            timeout,
            no_retry_on,
        )?)
        .map_err(|err| json_error_serialize(err, Some(self.create_context(path, Method::POST))))?)
    }
    pub fn put<D, R>(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        data: Option<D>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
    ) -> Result<R>
    where
        for<'a> R: Deserialize<'a>,
        D: Serialize + Debug,
    {
        let payload = match data {
            Some(txt) => Some(serde_json::to_string(&txt).map_err(|err| {
                json_error_serialize(err, Some(self.create_context(path.clone(), Method::PUT)))
            })?),
            None => None,
        };
        Ok(serde_json::from_str(&self.do_request(
            Method::PUT,
            path.to_string(),
            params,
            payload,
            headers,
            timeout,
            no_retry_on,
        )?)
        .map_err(|err| json_error_serialize(err, Some(self.create_context(path, Method::PUT))))?)
    }
    pub fn delete<R>(
        &self,
        path: String,
        params: Option<HashMap<String, String>>,
        headers: Option<HeaderMap>,
        timeout: Option<u64>,
        no_retry_on: Option<Vec<ErrorKind>>,
    ) -> Result<R>
    where
        for<'a> R: Deserialize<'a>,
    {
        Ok(serde_json::from_str(&self.do_request(
            Method::DELETE,
            path.to_string(),
            params,
            None,
            headers,
            timeout,
            no_retry_on,
        )?)
        .map_err(|err| {
            json_error_serialize(err, Some(self.create_context(path, Method::DELETE)))
        })?)
    }
}

#[cfg(test)]
mod test {
    use crate::client_rest::RestClient;
    use crate::errors::http::NOT_FOUND;
    use crate::errors::rest::DataError;
    use crate::ClientBuilder;
    use serde::{Deserialize, Serialize};
    use simple_logger::SimpleLogger;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            let _ = SimpleLogger::new()
                .with_level(log::LevelFilter::Info)
                .init();
        });
    }
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Todo {
        id: usize,
        todo: String,
        completed: bool,
        #[serde(rename(deserialize = "userId"))]
        user_id: u64,
    }
    #[derive(Serialize, Deserialize, Clone, Debug)]
    struct Foo {
        id: usize,
        foo: String,
    }

    #[test]
    fn test_get() {
        init_logger();
        let cli = RestClient::new("https://dummyjson.com").unwrap();
        let result = cli.get::<Todo>("/todos/1".into(), None, None, None, None);
        match result {
            Ok(todo) => assert_eq!(todo.user_id, 152),
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn test_get_payload_error() {
        init_logger();
        let cli = RestClient::new("https://dummyjson.com")
            .unwrap()
            .set_retry_number(2)
            .set_retry_delay(1);
        let result = cli.get::<Foo>("/todos/1".into(), None, None, None, None);
        match result {
            Ok(_) => panic!("No error raised!"),
            Err(err) => assert_eq!(err.kind, DataError),
        }
    }
    #[test]
    fn test_get_response_error() {
        init_logger();
        let cli = RestClient::new("https://dummyjson.com")
            .unwrap()
            .set_retry_number(2)
            .set_retry_delay(1);
        let result = cli.get::<Todo>("/todos/a".into(), None, None, None, Some(vec![NOT_FOUND]));
        match result {
            Ok(_) => panic!("No error raised!"),
            Err(err) => assert_eq!(err.kind, NOT_FOUND),
        }
    }
}
