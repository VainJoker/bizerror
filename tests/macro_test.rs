use std::error::Error as StdError;

use bizerror::*;
use thiserror::Error as ThisError;

/// Test enum with bizcode attributes
#[derive(BizError, ThisError)]
pub enum HttpRequestError {
    #[bizcode(8001)]
    #[error("Failed to build HTTP request: {0}")]
    RequestBuild(#[from] std::io::Error),

    #[bizcode(8002)]
    #[error("HTTP request failed with status {status}: {body}")]
    RequestFailed { status: u16, body: String },

    #[bizcode(8003)]
    #[error("Failed to parse response body: {0}")]
    ResponseParse(String),

    #[bizcode(8004)]
    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    #[bizcode(8005)]
    #[error("Serialization error: {0}")]
    Serialization(#[from] std::string::FromUtf8Error),

    #[bizcode(8006)]
    #[error("Request timeout")]
    Timeout,
}

/// Test enum with different variant types
#[derive(BizError, ThisError)]
pub enum CustomError {
    #[bizcode(1001)]
    #[error("Unit variant error")]
    UnitError,

    #[bizcode(1002)]
    #[error("Tuple variant error: {0}")]
    TupleError(String),

    #[bizcode(1003)]
    #[error("Struct variant error: {field}")]
    StructError { field: String },
}

// Another test enum to check for conflicts
#[derive(BizError, ThisError)]
pub enum DatabaseError {
    #[bizcode(9001)]
    #[error("Connection failed: {0}")]
    Connection(String),

    #[bizcode(9002)]
    #[error("Query failed: {query}")]
    QueryFailed { query: String },

    #[bizcode(9003)]
    #[error("Transaction rolled back")]
    TransactionRollback,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_error_codes() {
        let error = HttpRequestError::Timeout;
        assert_eq!(error.code(), 8006);
        assert_eq!(error.name(), "Timeout");
        assert_eq!(error.msg(), "Request timeout");
    }

    #[test]
    fn test_error_with_data() {
        let error = HttpRequestError::RequestFailed {
            status: 404,
            body:   "Not Found".to_string(),
        };
        assert_eq!(error.code(), 8002);
        assert_eq!(error.name(), "RequestFailed");
        assert!(error.msg().contains("404"));
        assert!(error.msg().contains("Not Found"));
    }

    #[test]
    fn test_error_with_source() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = HttpRequestError::RequestBuild(io_error);
        assert_eq!(error.code(), 8001);
        assert_eq!(error.name(), "RequestBuild");
        assert!(StdError::source(&error).is_some());
    }

    #[test]
    fn test_contextual_error() {
        let error = HttpRequestError::Timeout;
        let contextual = error.with_context("API call to user service");

        assert_eq!(contextual.code(), 8006);
        assert_eq!(contextual.name(), "Timeout");
        assert_eq!(contextual.context(), "API call to user service");
        assert!(contextual.location().file().contains("macro_test.rs"));
    }

    #[test]
    fn test_result_ext() {
        fn make_request() -> Result<String, std::io::Error> {
            Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"))
        }

        let result: Result<String, ContextualError<HttpRequestError>> =
            make_request().with_biz_context("Calling external API");

        assert!(result.is_err());
        let error = result.expect_err("should be an error");
        assert_eq!(error.code(), 8001); // RequestBuild from io::Error
        assert_eq!(error.context(), "Calling external API");
    }

    #[test]
    fn test_multiple_enums_no_conflict() {
        let http_error = HttpRequestError::Timeout;
        let db_error = DatabaseError::TransactionRollback;

        assert_eq!(http_error.code(), 8006);
        assert_eq!(db_error.code(), 9003);
        assert_eq!(http_error.name(), "Timeout");
        assert_eq!(db_error.name(), "TransactionRollback");
    }

    #[test]
    fn test_custom_debug_format() {
        let error = HttpRequestError::RequestFailed {
            status: 404,
            body:   "Not Found".to_string(),
        };

        let debug_output = format!("{error:?}");
        assert!(debug_output.contains("HttpRequestError"));
        assert!(debug_output.contains("variant"));
        assert!(debug_output.contains("RequestFailed"));
        assert!(debug_output.contains("code"));
        assert!(debug_output.contains("8002"));
        assert!(debug_output.contains("message"));
    }

    #[test]
    fn test_debug_with_source() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = HttpRequestError::RequestBuild(io_error);

        let debug_output = format!("{error:?}");
        assert!(debug_output.contains("HttpRequestError"));
        assert!(debug_output.contains("RequestBuild"));
        assert!(debug_output.contains("8001"));
        assert!(debug_output.contains("source"));
    }

    #[test]
    fn test_error_chain() {
        let io_error = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "config.json not found",
        );
        let http_error = HttpRequestError::from(io_error);
        let contextual = http_error.with_context("Loading application config");

        // Test the error chain
        assert_eq!(contextual.code(), 8001);
        assert_eq!(contextual.name(), "RequestBuild");
        assert!(StdError::source(&contextual).is_some());

        // Test display
        let display_str = format!("{contextual}");
        assert!(display_str.contains("Context: Loading application config"));
    }

    #[test]
    fn test_different_variant_types() {
        // Unit variant
        let unit_error = CustomError::UnitError;
        assert_eq!(unit_error.code(), 1001);
        assert_eq!(unit_error.name(), "UnitError");
        assert_eq!(unit_error.msg(), "Unit variant error");

        // Tuple variant
        let tuple_error = CustomError::TupleError("test".to_string());
        assert_eq!(tuple_error.code(), 1002);
        assert_eq!(tuple_error.name(), "TupleError");
        assert_eq!(tuple_error.msg(), "Tuple variant error: test");

        // Struct variant
        let struct_error = CustomError::StructError {
            field: "value".to_string(),
        };
        assert_eq!(struct_error.code(), 1003);
        assert_eq!(struct_error.name(), "StructError");
        assert_eq!(struct_error.msg(), "Struct variant error: value");
    }

    #[test]
    fn test_all_http_error_variants() {
        // Test RequestBuild
        let io_error =
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let request_build = HttpRequestError::from(io_error);
        assert_eq!(request_build.code(), 8001);
        assert_eq!(request_build.name(), "RequestBuild");
        assert!(request_build.msg().contains("Failed to build HTTP request"));

        // Test RequestFailed
        let request_failed = HttpRequestError::RequestFailed {
            status: 404,
            body:   "Not Found".to_string(),
        };
        assert_eq!(request_failed.code(), 8002);
        assert_eq!(request_failed.name(), "RequestFailed");
        assert!(request_failed.msg().contains("404"));
        assert!(request_failed.msg().contains("Not Found"));

        // Test ResponseParse
        let response_parse =
            HttpRequestError::ResponseParse("invalid json".to_string());
        assert_eq!(response_parse.code(), 8003);
        assert_eq!(response_parse.name(), "ResponseParse");
        assert!(response_parse.msg().contains("invalid json"));

        // Test InvalidUrl
        let invalid_url = HttpRequestError::InvalidUrl {
            url: "not-a-url".to_string(),
        };
        assert_eq!(invalid_url.code(), 8004);
        assert_eq!(invalid_url.name(), "InvalidUrl");
        assert!(invalid_url.msg().contains("not-a-url"));

        // Test Timeout
        let timeout = HttpRequestError::Timeout;
        assert_eq!(timeout.code(), 8006);
        assert_eq!(timeout.name(), "Timeout");
        assert_eq!(timeout.msg(), "Request timeout");
    }

    #[test]
    fn test_error_trait_implementations() {
        let error = HttpRequestError::Timeout;

        // Test that it implements Error trait
        let error_ref: &dyn std::error::Error = &error;
        assert!(error_ref.source().is_none());

        // Test Display implementation (from thiserror)
        let display_str = format!("{error}");
        assert_eq!(display_str, "Request timeout");

        // Test Debug implementation
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("Timeout"));
    }

    #[test]
    fn test_from_conversion() {
        let io_error = std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "access denied",
        );
        let http_error = HttpRequestError::from(io_error);

        assert_eq!(http_error.code(), 8001);
        assert_eq!(http_error.name(), "RequestBuild");
        assert!(http_error.msg().contains("access denied"));
    }
}
