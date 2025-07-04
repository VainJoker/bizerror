use std::{
    error::Error as StdError,
    io,
};

use bizerror::*;
use thiserror::Error as ThisError;

// --- Test Error Enums ---

#[derive(BizError, ThisError)]
#[bizconfig(auto_start = 1000, auto_increment = 10)]
pub enum AppError {
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: u32 },

    #[bizcode(2001)]
    #[error("Invalid input: {field}")]
    InvalidInput { field: String },

    #[error("Database connection failed")]
    DatabaseError(#[from] io::Error),

    #[bizcode(3000)]
    #[error("Permission denied")]
    PermissionDenied,

    #[error("Unknown error")]
    Unknown,
}

#[derive(BizError, ThisError)]
#[bizconfig(code_type = "&'static str", auto_start = 0, auto_increment = 1)]
pub enum StringCodeError {
    #[error("Resource not found")]
    NotFound,

    #[bizcode("AUTH_FAILED")]
    #[error("Authentication failed")]
    AuthFailed,

    #[error("Service unavailable")]
    ServiceUnavailable,
}

#[derive(BizError, ThisError)]
#[bizconfig(code_type = "i32", auto_start = -100, auto_increment = -5)]
pub enum SignedCodeError {
    #[error("Negative operation")]
    NegativeOp,

    #[bizcode(50)]
    #[error("Positive override")]
    PositiveOverride,

    #[error("Another negative")]
    AnotherNegative,
}

// --- Custom Error for BizError Trait Test ---

#[derive(Debug, PartialEq, Eq)]
pub struct CustomBizError {
    code:    u32,
    name:    &'static str,
    message: String,
}

impl StdError for CustomBizError {}

impl std::fmt::Display for CustomBizError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CustomBizError: {}", self.message)
    }
}

impl BizError for CustomBizError {
    type CodeType = u32;

    fn code(&self) -> Self::CodeType {
        self.code
    }

    fn name(&self) -> &str {
        self.name
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    // --- BizError Trait Tests ---

    #[test]
    fn test_biz_error_basic_properties() {
        let err = AppError::UserNotFound { user_id: 123 };
        assert_eq!(err.code(), 1000);
        assert_eq!(err.name(), "UserNotFound");
        assert_eq!(err.to_string(), "User not found: 123");

        let err = AppError::InvalidInput {
            field: "username".to_string(),
        };
        assert_eq!(err.code(), 2001);
        assert_eq!(err.name(), "InvalidInput");
        assert_eq!(err.to_string(), "Invalid input: username");

        let err = AppError::Unknown;
        assert_eq!(err.code(), 1020); // 1000 + 2 * 10 (UserNotFound, DatabaseError, Unknown - skipping InvalidInput and PermissionDenied)
        assert_eq!(err.name(), "Unknown");
    }

    #[test]
    fn test_biz_error_from_io_error() {
        let io_err =
            io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err = AppError::from(io_err);
        assert_eq!(err.code(), 1010); // 1000 + 1 * 10 (UserNotFound, DatabaseError)
        assert_eq!(err.name(), "DatabaseError");

        // 检查错误信息
        assert!(err.to_string().contains("Database connection failed"));
    }

    #[test]
    fn test_string_code_error() {
        let err = StringCodeError::NotFound;
        assert_eq!(err.code(), "0");
        assert_eq!(err.name(), "NotFound");

        let err = StringCodeError::AuthFailed;
        assert_eq!(err.code(), "AUTH_FAILED");
        assert_eq!(err.name(), "AuthFailed");

        let err = StringCodeError::ServiceUnavailable;
        assert_eq!(err.code(), "1");
        assert_eq!(err.name(), "ServiceUnavailable");
    }

    #[test]
    fn test_signed_code_error() {
        let err = SignedCodeError::NegativeOp;
        assert_eq!(err.code(), -100);
        assert_eq!(err.name(), "NegativeOp");

        let err = SignedCodeError::PositiveOverride;
        assert_eq!(err.code(), 50);
        assert_eq!(err.name(), "PositiveOverride");

        let err = SignedCodeError::AnotherNegative;
        assert_eq!(err.code(), -105);
        assert_eq!(err.name(), "AnotherNegative");
    }

    #[test]
    fn test_custom_biz_error_trait_impl() {
        let custom_err = CustomBizError {
            code:    999,
            name:    "MyCustomError",
            message: "Something went wrong".to_string(),
        };
        assert_eq!(custom_err.code(), 999);
        assert_eq!(custom_err.name(), "MyCustomError");
        assert_eq!(
            custom_err.to_string(),
            "CustomBizError: Something went wrong"
        );
    }

    // --- ContextualError Struct Tests ---

    #[test]
    fn test_contextual_error_creation_and_accessors() {
        let err = AppError::UserNotFound { user_id: 456 };
        let contextual = err.with_context("Fetching user profile");

        assert_eq!(contextual.code(), 1000);
        assert_eq!(contextual.name(), "UserNotFound");
        assert_eq!(contextual.context(), "Fetching user profile");
        assert!(contextual.location().file().contains("bizerror_tests.rs"));
        assert!(contextual.location().line() > 0);
    }

    #[test]
    fn test_contextual_error_add_context() {
        let err = AppError::InvalidInput {
            field: "email".to_string(),
        };
        let contextual = err.with_context("Validating form data");
        let layered = contextual.add_context("Before database insert");

        assert_eq!(
            layered.context(),
            "Validating form data -> Before database insert"
        );
        assert_eq!(layered.code(), 2001);
    }

    #[test]
    fn test_contextual_error_into_inner() {
        let err = AppError::PermissionDenied;
        let contextual = err.with_context("Accessing restricted resource");
        let _inner_err = contextual.into_inner();
    }

    #[test]
    fn test_contextual_error_debug_display() {
        let io_err = io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken");
        let err = AppError::from(io_err);
        let contextual = err.with_context("Writing to socket");

        let debug_str = format!("{contextual:?}");

        assert!(debug_str.contains("ContextualError"));
        assert!(debug_str.contains("type: \"DatabaseError\""));
        assert!(debug_str.contains("code: 1010"));
        assert!(debug_str.contains("context: \"Writing to socket\""));
        assert!(
            debug_str.contains("location:") &&
                debug_str.contains("bizerror_tests.rs")
        );

        let display_str = format!("{contextual}");
        assert!(display_str.contains("Database connection failed"));
        assert!(display_str.contains("Context: Writing to socket"));
    }

    // --- Error Chain Navigation Tests ---

    fn create_complex_error_chain() -> ContextualError<AppError> {
        let io_err =
            io::Error::new(io::ErrorKind::NotFound, "config.toml not found");
        let db_err = AppError::DatabaseError(io_err);

        db_err
            .with_context("Loading application config")
            .add_context("Initializing services")
            .add_context("Application startup failed")
    }

    #[test]
    fn test_chain_depth() {
        let err = create_complex_error_chain();
        // ContextualError -> AppError::DatabaseError -> io::Error
        assert_eq!(err.chain_depth(), 3);
    }

    #[test]
    fn test_root_cause_message() {
        let err = create_complex_error_chain();
        assert_eq!(err.root_cause_message(), "config.toml not found");
    }

    #[test]
    fn test_error_chain_messages() {
        let err = create_complex_error_chain();
        let messages = err.error_chain_messages();
        assert_eq!(messages.len(), 3);
        assert!(messages[0].contains("Application startup failed"));
        assert!(messages[1].contains("Database connection failed"));
        assert!(messages[2].contains("config.toml not found"));
    }

    #[test]
    fn test_find_root() {
        let err = create_complex_error_chain();
        assert!(err.find_root::<io::Error>().is_some());
        assert!(err.find_root::<AppError>().is_some());
        assert!(err.find_root::<StringCodeError>().is_none());
    }

    #[test]
    fn test_contains_error_type() {
        let err = create_complex_error_chain();
        assert!(err.contains_error::<io::Error>());
        assert!(err.contains_error::<AppError>());
        assert!(!err.contains_error::<StringCodeError>());
    }

    #[test]
    fn test_chain_contains_code() {
        let err = create_complex_error_chain();
        assert!(err.chain_contains_code(1010)); // AppError::DatabaseError code
        assert!(!err.chain_contains_code(1000)); // AppError::UserNotFound code (not in this chain)
        assert!(!err.chain_contains_code(9999)); // Non-existent code
    }

    // --- ResultExt Trait Tests ---

    fn fallible_io_op(succeed: bool) -> Result<String, io::Error> {
        if succeed {
            Ok("data".to_string())
        } else {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken"))
        }
    }

    #[test]
    fn test_result_ext_with_context() {
        let result: Result<String, ContextualError<AppError>> =
            fallible_io_op(false).with_context("Reading from device");
        assert!(result.is_err());
        let err = result.expect_err("");
        assert_eq!(err.code(), 1010);
        assert_eq!(err.context(), "Reading from device");
    }

    #[test]
    fn test_result_ext_map_biz() {
        let result: Result<String, AppError> =
            fallible_io_op(false).map_biz::<AppError>();
        assert!(result.is_err());
        let err = result.expect_err("");
        assert_eq!(err.code(), 1010);
    }

    #[test]
    fn test_result_ext_with_context_if() {
        let result_true: Result<String, ContextualError<AppError>> =
            fallible_io_op(false).with_context_if(true, "Debug mode enabled");
        assert!(result_true.is_err());
        assert_eq!(result_true.expect_err("").context(), "Debug mode enabled");

        let result_false: Result<String, ContextualError<AppError>> =
            fallible_io_op(false).with_context_if(false, "Should not see this");
        assert!(result_false.is_err());
        assert_eq!(result_false.expect_err("").context(), "no context");

        let result_ok: Result<String, ContextualError<AppError>> =
            fallible_io_op(true).with_context_if(true, "Should not see this");
        assert!(result_ok.is_ok());
    }

    #[test]
    fn test_result_ext_and_then_biz() {
        let initial_result: Result<u32, io::Error> = Ok(10);
        let chained_result: Result<String, AppError> = initial_result
            .and_then_biz(|val| {
                if val > 5 {
                    Ok(format!("Value is {val}"))
                } else {
                    Err(AppError::InvalidInput {
                        field: "value".to_string(),
                    })
                }
            });
        assert_eq!(chained_result.expect(""), "Value is 10");

        let initial_err: Result<u32, io::Error> =
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "pipe broken"));
        let chained_err: Result<String, AppError> =
            initial_err.and_then_biz(|val| Ok(format!("Value is {val}")));
        assert!(chained_err.is_err());
        assert_eq!(chained_err.expect_err("").code(), 1010);
    }

    // --- OptionExt Trait Tests ---

    #[test]
    fn test_option_ext_ok_or_biz() {
        let some_val: Option<u32> = Some(100);
        let result = some_val.ok_or_biz(AppError::UserNotFound { user_id: 0 });
        assert_eq!(result.expect(""), 100);

        let none_val: Option<u32> = None;
        let result = none_val.ok_or_biz(AppError::UserNotFound { user_id: 0 });
        assert!(result.is_err());
        assert_eq!(result.expect_err("").code(), 1000);
    }

    // --- BizErrors Struct Tests ---

    #[test]
    fn test_biz_errors_collection() {
        let mut errors = BizErrors::new();
        assert!(errors.is_empty());
        assert_eq!(errors.len(), 0);

        errors.push_simple(AppError::UserNotFound { user_id: 1 });
        errors.push_with_context(
            AppError::InvalidInput {
                field: "name".to_string(),
            },
            "Validating user name",
        );
        errors.push(AppError::PermissionDenied.with_context("Admin access"));

        assert!(!errors.is_empty());
        assert_eq!(errors.len(), 3);

        assert_eq!(errors.first().expect("").code(), 1000);
        assert_eq!(errors.last().expect("").code(), 3000);

        let collected_codes = errors.error_codes();
        assert_eq!(collected_codes.len(), 3);
        assert!(collected_codes.contains(&1000));
        assert!(collected_codes.contains(&2001));
        assert!(collected_codes.contains(&3000));

        assert!(errors.contains_code(1000));
        assert!(!errors.contains_code(9999));

        let filtered_errors: Vec<_> =
            errors.filter(|e| e.code() == 2001).collect();
        assert_eq!(filtered_errors.len(), 1);
        assert_eq!(filtered_errors[0].name(), "InvalidInput");
    }

    #[test]
    fn test_biz_errors_collect_from_iterator() {
        let results: Vec<Result<u32, ContextualError<AppError>>> = vec![
            Ok(1),
            Err(AppError::UserNotFound { user_id: 1 }.with_context("Op 1")),
            Ok(2),
            Err(AppError::InvalidInput {
                field: "data".to_string(),
            }
            .with_context("Op 2")),
            Ok(3),
        ];

        let (successes, errors_opt) =
            BizErrors::collect_from(results.into_iter());
        assert_eq!(successes, vec![1, 2, 3]);
        assert!(errors_opt.is_some());
        let errors = errors_opt.expect("");
        assert_eq!(errors.len(), 2);
        assert!(errors.contains_code(1000));
        assert!(errors.contains_code(2001));
    }

    #[test]
    fn test_biz_errors_collect_errors_from_iterator() {
        let results: Vec<Result<(), ContextualError<AppError>>> = vec![
            Ok(()),
            Err(AppError::PermissionDenied.with_context("Check 1")),
            Ok(()),
            Err(AppError::Unknown.with_context("Check 2")),
        ];

        let errors_opt = BizErrors::collect_errors(results.into_iter());
        assert!(errors_opt.is_some());
        let errors = errors_opt.expect("");
        assert_eq!(errors.len(), 2);
        assert!(errors.contains_code(3000)); // PermissionDenied
        assert!(errors.contains_code(1020)); // Unknown
    }

    #[test]
    fn test_biz_errors_debug_display() {
        let mut errors = BizErrors::new();
        errors.push_simple(AppError::UserNotFound { user_id: 1 });
        errors.push_simple(AppError::InvalidInput {
            field: "name".to_string(),
        });

        let debug_str = format!("{errors:?}");
        assert!(debug_str.contains("BizErrors"));
        assert!(debug_str.contains("count: 2"));
        assert!(debug_str.contains("codes: [1000, 2001]"));

        let display_str = format!("{errors}");
        assert!(display_str.contains("Multiple errors occurred (2 total):"));
        assert!(display_str.contains("1. User not found: 1"));
        assert!(display_str.contains("2. Invalid input: name"));
    }

    #[test]
    fn test_biz_errors_into_iterator() {
        let mut errors = BizErrors::new();
        errors.push_simple(AppError::UserNotFound { user_id: 1 });
        errors.push_simple(AppError::PermissionDenied);

        let mut count = 0;
        for err in errors {
            count += 1;
            assert!(err.code() == 1000 || err.code() == 3000);
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_biz_errors_from_iterator() {
        let errors: BizErrors<AppError> = vec![
            AppError::UserNotFound { user_id: 1 },
            AppError::PermissionDenied,
        ]
        .into_iter()
        .collect();

        assert_eq!(errors.len(), 2);
        assert!(errors.contains_code(1000));
        assert!(errors.contains_code(3000));

        let contextual_errors: BizErrors<AppError> = vec![
            AppError::UserNotFound { user_id: 1 }.with_context("Ctx 1"),
            AppError::PermissionDenied.with_context("Ctx 2"),
        ]
        .into_iter()
        .collect();

        assert_eq!(contextual_errors.len(), 2);
        assert!(contextual_errors.contains_code(1000));
        assert!(contextual_errors.contains_code(3000));
    }
}
