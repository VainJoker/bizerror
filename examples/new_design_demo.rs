use bizerror::*;
use thiserror::Error as ThisError;

/// Example 1: Basic auto-assignment (default config)
#[derive(BizError, ThisError)]
pub enum SimpleError {
    #[error("Network connection failed")]
    NetworkError, // Auto code: 0

    #[error("Invalid input: {input}")]
    ValidationError { input: String }, // Auto code: 1

    #[error("Operation timeout")]
    TimeoutError, // Auto code: 2
}

/// Example 2: Custom configuration with mixed codes
#[derive(BizError, ThisError)]
#[bizconfig(code_type = "u32", auto_start = 1000, auto_increment = 10)]
pub enum ApiError {
    #[error("Authentication failed")]
    AuthError, // Auto code: 1000

    #[bizcode(5001)]
    #[error("Custom business rule violation")]
    BusinessRuleError, // Explicit code: 5001

    #[error("Database connection failed")]
    DatabaseError, // Auto code: 1010

    #[bizcode(9999)]
    #[error("Unknown error")]
    UnknownError, // Explicit code: 9999

    #[error("Service unavailable")]
    ServiceError, // Auto code: 1020
}

/// Example 4: String codes
#[derive(BizError, ThisError)]
#[bizconfig(code_type = "&'static str", auto_start = 0, auto_increment = 1)]
pub enum StringCodeError {
    #[error("Network timeout")]
    NetworkTimeout, // Auto code: "0"

    #[bizcode("AUTH_FAILED")]
    #[error("Authentication failed")]
    AuthFailed, // Explicit code: "AUTH_FAILED"

    #[error("Data validation failed")]
    ValidationFailed, // Auto code: "1"

    #[bizcode("DB_ERROR")]
    #[error("Database error")]
    DatabaseError, // Explicit code: "DB_ERROR"
}

/// Example 5: Duplicate codes (allowed)
#[derive(BizError, ThisError)]
pub enum DuplicateCodeError {
    #[bizcode(500)]
    #[error("Server error type A")]
    ServerErrorA,

    #[bizcode(500)]
    #[error("Server error type B")]
    ServerErrorB, // Same code as A - allowed

    #[error("Client error")]
    ClientError, // Auto code: 0
}

fn main() {
    println!("🎯 BizError New Design Demo");
    println!("===========================\n");

    // Example 1: Simple auto-assignment
    println!("📝 Example 1: Basic Auto-Assignment");
    println!("------------------------------------");
    let simple_errors = [
        SimpleError::NetworkError,
        SimpleError::ValidationError {
            input: "invalid@email".to_string(),
        },
        SimpleError::TimeoutError,
    ];

    for error in &simple_errors {
        println!(
            "  {} -> Code: {}, Name: {}",
            error.msg(),
            error.code(),
            error.name()
        );
    }
    println!();

    // Example 2: Mixed explicit and auto codes
    println!("📝 Example 2: Mixed Explicit and Auto Codes");
    println!("--------------------------------------------");
    let api_errors = [
        ApiError::AuthError,
        ApiError::BusinessRuleError,
        ApiError::DatabaseError,
        ApiError::UnknownError,
        ApiError::ServiceError,
    ];

    for error in &api_errors {
        println!(
            "  {} -> Code: {}, Name: {}",
            error.msg(),
            error.code(),
            error.name()
        );
    }
    println!();

    // Example 4: String codes
    println!("📝 Example 4: String Codes");
    println!("---------------------------");
    let string_errors = [
        StringCodeError::NetworkTimeout,
        StringCodeError::AuthFailed,
        StringCodeError::ValidationFailed,
        StringCodeError::DatabaseError,
    ];

    for error in &string_errors {
        println!(
            "  {} -> Code: \"{}\", Name: {}",
            error.msg(),
            error.code(),
            error.name()
        );
    }
    println!();

    // Example 5: Duplicate codes
    println!("📝 Example 5: Duplicate Codes (Allowed)");
    println!("----------------------------------------");
    let duplicate_errors = [
        DuplicateCodeError::ServerErrorA,
        DuplicateCodeError::ServerErrorB,
        DuplicateCodeError::ClientError,
    ];

    for error in &duplicate_errors {
        println!(
            "  {} -> Code: {}, Name: {}",
            error.msg(),
            error.code(),
            error.name()
        );
    }
    println!();

    // Example 6: Debug output
    println!("📝 Example 6: Debug Output");
    println!("--------------------------");
    let error = ApiError::BusinessRuleError;
    println!("  Debug: {error:?}");
    println!();

    // Example 7: Contextual errors
    println!("📝 Example 7: Contextual Errors");
    println!("--------------------------------");
    let contextual_error = SimpleError::NetworkError
        .with_context("Failed to connect to external API");

    println!("  Code: {}", contextual_error.code());
    println!("  Name: {}", contextual_error.name());
    println!("  Context: {}", contextual_error.context());
    println!("  Location: {}", contextual_error.location());
    println!();

    // Example 8: Result extension
    println!("📝 Example 8: Result Extension");
    println!("------------------------------");
    let result: Result<String, SimpleError> = Err(SimpleError::TimeoutError);
    let contextual_result: Result<String, ContextualError<SimpleError>> =
        result.with_biz_context("Processing user request");

    match contextual_result {
        Ok(_) => println!("  Success"),
        Err(e) => {
            println!("  Error: {}", e.msg());
            println!("  Context: {}", e.context());
        }
    }
}
