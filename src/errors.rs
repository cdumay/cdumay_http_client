use std::collections::BTreeMap;

use cdumay_error::{ErrorBuilder, ErrorKind, GenericErrors};
use serde_json::value::Value;

#[derive(Debug)]
pub struct ResponseErrorWithContext {
    pub status: reqwest::StatusCode,
    pub context: Option<BTreeMap<String, Value>>,
}


#[derive(Debug)]
pub enum ClientError {
    InternalError(String),
    ParseError(String),
    GenericHttpError(reqwest::Error),
    UrlError(url::ParseError),
    ResponseError(ResponseErrorWithContext),
    InvalidHeaderName(reqwest::header::InvalidHeaderName),
    InvalidHeaderValue(reqwest::header::InvalidHeaderValue),
}

impl From<ClientError> for ErrorBuilder {
    fn from(value: ClientError) -> Self {
        match value {
            ClientError::InternalError(data) => ErrorBuilder::from(GenericErrors::UNKNOWN_ERROR).message(data),
            ClientError::ParseError(data) => ErrorBuilder::from(GenericErrors::DESERIALIZATION_ERROR).message(data),
            ClientError::GenericHttpError(err) => ErrorBuilder::from(HttpStatusCodeErrors::GENERIC_HTTP_CLIENT_ERROR).message(err.to_string()),
            ClientError::UrlError(err) => ErrorBuilder::from(HttpStatusCodeErrors::INVALID_URL).message(err.to_string()),
            ClientError::ResponseError(err) => {
                match err.context {
                    Some(context) => ErrorBuilder::from(HttpStatusCodeErrors::from_status(err.status)).extra(context.into()),
                    None => ErrorBuilder::from(HttpStatusCodeErrors::from_status(err.status))
                }
            }
            ClientError::InvalidHeaderName(err) => ErrorBuilder::from(GenericErrors::VALIDATION_ERROR).message(err.to_string()),
            ClientError::InvalidHeaderValue(err) => ErrorBuilder::from(GenericErrors::VALIDATION_ERROR).message(err.to_string()),
        }
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> ClientError {
        ClientError::GenericHttpError(err)
    }
}

impl From<reqwest::header::InvalidHeaderName> for ClientError {
    fn from(err: reqwest::header::InvalidHeaderName) -> ClientError {
        ClientError::InvalidHeaderName(err)
    }
}

impl From<reqwest::header::InvalidHeaderValue> for ClientError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> ClientError {
        ClientError::InvalidHeaderValue(err)
    }
}

impl From<reqwest::blocking::Response> for ClientError {
    fn from(resp: reqwest::blocking::Response) -> ClientError {
        ClientError::ResponseError(ResponseErrorWithContext {
            status: resp.status(),
            context: resp.json::<BTreeMap<String, Value>>().ok(),
        })
    }
}


pub struct HttpStatusCodeErrors;

impl HttpStatusCodeErrors {
    pub const INVALID_URL: ErrorKind = ErrorKind("Err-27071", 400, "Invalid url");
    pub const GENERIC_HTTP_CLIENT_ERROR: ErrorKind = ErrorKind("Err-05192", 500, "Generic HTTP client error");
    pub const MULTIPLE_CHOICES: ErrorKind = ErrorKind("Err-11298", 300, "Multiple Choices");
    pub const MOVED_PERMANENTLY: ErrorKind = ErrorKind("Err-23108", 301, "Moved Permanently");
    pub const FOUND: ErrorKind = ErrorKind("Err-07132", 302, "Found");
    pub const SEE_OTHER: ErrorKind = ErrorKind("Err-16746", 303, "See Other");
    pub const NOT_MODIFIED: ErrorKind = ErrorKind("Err-21556", 304, "Not Modified");
    pub const USE_PROXY: ErrorKind = ErrorKind("Err-31839", 305, "Use Proxy");
    pub const TEMPORARY_REDIRECT: ErrorKind = ErrorKind("Err-25446", 307, "Temporary Redirect");
    pub const PERMANENT_REDIRECT: ErrorKind = ErrorKind("Err-12280", 308, "Permanent Redirect");
    pub const BAD_REQUEST: ErrorKind = ErrorKind("Err-26760", 400, "Bad Request");
    pub const UNAUTHORIZED: ErrorKind = ErrorKind("Err-08059", 401, "Unauthorized");
    pub const PAYMENT_REQUIRED: ErrorKind = ErrorKind("Err-18076", 402, "Payment Required");
    pub const FORBIDDEN: ErrorKind = ErrorKind("Err-23134", 403, "Forbidden");
    pub const NOT_FOUND: ErrorKind = ErrorKind("Err-18430", 404, "Not Found");
    pub const METHOD_NOT_ALLOWED: ErrorKind = ErrorKind("Err-23585", 405, "Method Not Allowed");
    pub const NOT_ACCEPTABLE: ErrorKind = ErrorKind("Err-04289", 406, "Not Acceptable");
    pub const PROXY_AUTHENTICATION_REQUIRED: ErrorKind = ErrorKind("Err-17336", 407, "Proxy Authentication Required");
    pub const REQUEST_TIMEOUT: ErrorKind = ErrorKind("Err-00565", 408, "Request Timeout");
    pub const CONFLICT: ErrorKind = ErrorKind("Err-08442", 409, "Conflict");
    pub const GONE: ErrorKind = ErrorKind("Err-19916", 410, "Gone");
    pub const LENGTH_REQUIRED: ErrorKind = ErrorKind("Err-09400", 411, "Length Required");
    pub const PRECONDITION_FAILED: ErrorKind = ErrorKind("Err-22509", 412, "Precondition Failed");
    pub const PAYLOAD_TOO_LARGE: ErrorKind = ErrorKind("Err-10591", 413, "Payload Too Large");
    pub const URI_TOO_LONG: ErrorKind = ErrorKind("Err-01377", 414, "URI Too Long");
    pub const UNSUPPORTED_MEDIA_TYPE: ErrorKind = ErrorKind("Err-12512", 415, "Unsupported Media Type");
    pub const RANGE_NOT_SATISFIABLE: ErrorKind = ErrorKind("Err-21696", 416, "Range Not Satisfiable");
    pub const EXPECTATION_FAILED: ErrorKind = ErrorKind("Err-16872", 417, "Expectation Failed");
    pub const IM_A_TEAPOT: ErrorKind = ErrorKind("Err-23719", 418, "I'm a teapot");
    pub const MISDIRECTED_REQUEST: ErrorKind = ErrorKind("Err-26981", 421, "Misdirected Request");
    pub const UNPROCESSABLE_ENTITY: ErrorKind = ErrorKind("Err-12568", 422, "Unprocessable Entity");
    pub const LOCKED: ErrorKind = ErrorKind("Err-32695", 423, "Locked");
    pub const FAILED_DEPENDENCY: ErrorKind = ErrorKind("Err-19693", 424, "Failed Dependency");
    pub const UPGRADE_REQUIRED: ErrorKind = ErrorKind("Err-22991", 426, "Upgrade Required");
    pub const PRECONDITION_REQUIRED: ErrorKind = ErrorKind("Err-02452", 428, "Precondition Required");
    pub const TOO_MANY_REQUESTS: ErrorKind = ErrorKind("Err-12176", 429, "Too Many Requests");
    pub const REQUEST_HEADER_FIELDS_TOO_LARGE: ErrorKind = ErrorKind("Err-07756", 431, "Request Header Fields Too Large");
    pub const UNAVAILABLE_FOR_LEGAL_REASONS: ErrorKind = ErrorKind("Err-12136", 451, "Unavailable For Legal Reasons");
    pub const INTERNAL_SERVER_ERROR: ErrorKind = ErrorKind("Err-09069", 500, "Internal Server Error");
    pub const NOT_IMPLEMENTED: ErrorKind = ErrorKind("Err-03394", 501, "Not Implemented");
    pub const BAD_GATEWAY: ErrorKind = ErrorKind("Err-19734", 502, "Bad Gateway");
    pub const SERVICE_UNAVAILABLE: ErrorKind = ErrorKind("Err-18979", 503, "Service Unavailable");
    pub const GATEWAY_TIMEOUT: ErrorKind = ErrorKind("Err-17595", 504, "Gateway Timeout");
    pub const HTTP_VERSION_NOT_SUPPORTED: ErrorKind = ErrorKind("Err-01625", 505, "HTTP Version Not Supported");
    pub const VARIANT_ALSO_NEGOTIATES: ErrorKind = ErrorKind("Err-28382", 506, "Variant Also Negotiates");
    pub const INSUFFICIENT_STORAGE: ErrorKind = ErrorKind("Err-32132", 507, "Insufficient Storage");
    pub const LOOP_DETECTED: ErrorKind = ErrorKind("Err-30770", 508, "Loop Detected");
    pub const NOT_EXTENDED: ErrorKind = ErrorKind("Err-19347", 510, "Not Extended");
    pub const NETWORK_AUTHENTICATION_REQUIRED: ErrorKind = ErrorKind("Err-31948", 511, "Network Authentication Required");
}

impl HttpStatusCodeErrors {
    pub fn from_status(status: http::StatusCode) -> ErrorKind {
        match status {
            http::StatusCode::MOVED_PERMANENTLY => HttpStatusCodeErrors::MOVED_PERMANENTLY,
            http::StatusCode::FOUND => HttpStatusCodeErrors::FOUND,
            http::StatusCode::SEE_OTHER => HttpStatusCodeErrors::SEE_OTHER,
            http::StatusCode::NOT_MODIFIED => HttpStatusCodeErrors::NOT_MODIFIED,
            http::StatusCode::USE_PROXY => HttpStatusCodeErrors::USE_PROXY,
            http::StatusCode::TEMPORARY_REDIRECT => HttpStatusCodeErrors::TEMPORARY_REDIRECT,
            http::StatusCode::PERMANENT_REDIRECT => HttpStatusCodeErrors::PERMANENT_REDIRECT,
            http::StatusCode::BAD_REQUEST => HttpStatusCodeErrors::BAD_REQUEST,
            http::StatusCode::UNAUTHORIZED => HttpStatusCodeErrors::UNAUTHORIZED,
            http::StatusCode::PAYMENT_REQUIRED => HttpStatusCodeErrors::PAYMENT_REQUIRED,
            http::StatusCode::FORBIDDEN => HttpStatusCodeErrors::FORBIDDEN,
            http::StatusCode::NOT_FOUND => HttpStatusCodeErrors::NOT_FOUND,
            http::StatusCode::METHOD_NOT_ALLOWED => HttpStatusCodeErrors::METHOD_NOT_ALLOWED,
            http::StatusCode::NOT_ACCEPTABLE => HttpStatusCodeErrors::NOT_ACCEPTABLE,
            http::StatusCode::PROXY_AUTHENTICATION_REQUIRED => HttpStatusCodeErrors::PROXY_AUTHENTICATION_REQUIRED,
            http::StatusCode::REQUEST_TIMEOUT => HttpStatusCodeErrors::REQUEST_TIMEOUT,
            http::StatusCode::CONFLICT => HttpStatusCodeErrors::CONFLICT,
            http::StatusCode::GONE => HttpStatusCodeErrors::GONE,
            http::StatusCode::LENGTH_REQUIRED => HttpStatusCodeErrors::LENGTH_REQUIRED,
            http::StatusCode::PRECONDITION_FAILED => HttpStatusCodeErrors::PRECONDITION_FAILED,
            http::StatusCode::PAYLOAD_TOO_LARGE => HttpStatusCodeErrors::PAYLOAD_TOO_LARGE,
            http::StatusCode::URI_TOO_LONG => HttpStatusCodeErrors::URI_TOO_LONG,
            http::StatusCode::UNSUPPORTED_MEDIA_TYPE => HttpStatusCodeErrors::UNSUPPORTED_MEDIA_TYPE,
            http::StatusCode::RANGE_NOT_SATISFIABLE => HttpStatusCodeErrors::RANGE_NOT_SATISFIABLE,
            http::StatusCode::EXPECTATION_FAILED => HttpStatusCodeErrors::EXPECTATION_FAILED,
            http::StatusCode::IM_A_TEAPOT => HttpStatusCodeErrors::IM_A_TEAPOT,
            http::StatusCode::MISDIRECTED_REQUEST => HttpStatusCodeErrors::MISDIRECTED_REQUEST,
            http::StatusCode::UNPROCESSABLE_ENTITY => HttpStatusCodeErrors::UNPROCESSABLE_ENTITY,
            http::StatusCode::LOCKED => HttpStatusCodeErrors::LOCKED,
            http::StatusCode::FAILED_DEPENDENCY => HttpStatusCodeErrors::FAILED_DEPENDENCY,
            http::StatusCode::UPGRADE_REQUIRED => HttpStatusCodeErrors::UPGRADE_REQUIRED,
            http::StatusCode::PRECONDITION_REQUIRED => HttpStatusCodeErrors::PRECONDITION_REQUIRED,
            http::StatusCode::TOO_MANY_REQUESTS => HttpStatusCodeErrors::TOO_MANY_REQUESTS,
            http::StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE => HttpStatusCodeErrors::REQUEST_HEADER_FIELDS_TOO_LARGE,
            http::StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS => HttpStatusCodeErrors::UNAVAILABLE_FOR_LEGAL_REASONS,
            http::StatusCode::INTERNAL_SERVER_ERROR => HttpStatusCodeErrors::INTERNAL_SERVER_ERROR,
            http::StatusCode::NOT_IMPLEMENTED => HttpStatusCodeErrors::NOT_IMPLEMENTED,
            http::StatusCode::BAD_GATEWAY => HttpStatusCodeErrors::BAD_GATEWAY,
            http::StatusCode::SERVICE_UNAVAILABLE => HttpStatusCodeErrors::SERVICE_UNAVAILABLE,
            http::StatusCode::GATEWAY_TIMEOUT => HttpStatusCodeErrors::GATEWAY_TIMEOUT,
            http::StatusCode::HTTP_VERSION_NOT_SUPPORTED => HttpStatusCodeErrors::HTTP_VERSION_NOT_SUPPORTED,
            http::StatusCode::VARIANT_ALSO_NEGOTIATES => HttpStatusCodeErrors::VARIANT_ALSO_NEGOTIATES,
            http::StatusCode::INSUFFICIENT_STORAGE => HttpStatusCodeErrors::INSUFFICIENT_STORAGE,
            http::StatusCode::LOOP_DETECTED => HttpStatusCodeErrors::LOOP_DETECTED,
            http::StatusCode::NOT_EXTENDED => HttpStatusCodeErrors::NOT_EXTENDED,
            http::StatusCode::NETWORK_AUTHENTICATION_REQUIRED => HttpStatusCodeErrors::NETWORK_AUTHENTICATION_REQUIRED,
            _ => panic!("{} is not an error!", status),
        }
    }
}
