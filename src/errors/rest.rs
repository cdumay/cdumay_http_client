use cdumay_context::Context;
use cdumay_error::{define_errors, define_kinds, AsError, Error};
use serde_json::error::Category;

define_kinds! {
    IOError = ("JSON-11553", 500, "IO Error"),
    SyntaxError = ("JSON-57633", 400, "Syntax Error"),
    DataError = ("JSON-15852", 400, "Invalid JSON data"),
    EOF = ("JSON-15853", 500, "Reached the end of the input data"),
}

define_errors! {
    JsonIOError = IOError,
    JsonSyntaxError = SyntaxError,
    JsonDataError = DataError,
    JsonEOF = EOF,
}

pub fn json_error_serialize(err: serde_json::Error, context: Option<Context>) -> Error {
    let context = context.unwrap_or_default();
    match err.classify() {
        Category::Io => JsonIOError::new()
            .set_message(err.to_string())
            .set_details(context.into())
            .into(),
        Category::Syntax => JsonSyntaxError::new()
            .set_message(err.to_string())
            .set_details(context.into())
            .into(),
        Category::Data => JsonDataError::new()
            .set_message(err.to_string())
            .set_details(context.into())
            .into(),
        Category::Eof => JsonEOF::new()
            .set_message(err.to_string())
            .set_details(context.into())
            .into(),
    }
}
