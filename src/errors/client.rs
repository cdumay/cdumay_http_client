use cdumay_error::{define_errors, define_kinds, AsError};

define_kinds! {
    UNKNOWN_ERROR = ("Err-79483", 500, "Unexpected error"),
    INVALID_URL = ("Err-27071", 400, "Invalid url"),
    GENERIC_HTTP_CLIENT_ERROR = ("Err-05192", 500, "Generic HTTP client error"),
    CONTENT_ERROR = ("Err-45973", 400, "The error is related to the request or response body"),
    NETWORK_CONNECTION = ("Err-64752", 500, "The error is related to connect"),
    REQUEST_ERROR = ("Err-37984", 500, "The error is related to the request"),
}

define_errors! {
    InvalidUrl = INVALID_URL,
    ClientBuilderError = GENERIC_HTTP_CLIENT_ERROR,
    InvalidContent = CONTENT_ERROR,
    NetworkError = NETWORK_CONNECTION,
    RequestError = REQUEST_ERROR,
    UnexpectedError = UNKNOWN_ERROR,
    InvalidHeaderValue = CONTENT_ERROR
}
