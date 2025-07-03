use core::panic::Location;
use std::error::Error;

use thiserror::Error as ThisError;

/// Core business error trait
pub trait BizError: Error + Send + Sync + 'static {
    /// Get the business error code
    fn code(&self) -> u16;

    /// Get the error type name
    fn name(&self) -> &str;

    /// Get the business error message (optional, defaults to Display)
    fn msg(&self) -> Option<&str> {
        None
    }
}

/// Contextual error wrapper (only used when detailed context is needed)
#[derive(Debug)]
pub struct ContextualError<E: BizError> {
    error:    E,
    context:  String,
    location: &'static Location<'static>,
}

impl<E: BizError> ContextualError<E> {
    #[track_caller]
    pub fn new(error: E, context: impl Into<String>) -> Self {
        Self {
            error,
            context: context.into(),
            location: Location::caller(),
        }
    }

    /// Get the original error
    pub const fn inner(&self) -> &E {
        &self.error
    }

    /// Get the context
    pub fn context(&self) -> &str {
        &self.context
    }

    /// Get the location
    pub const fn location(&self) -> &'static Location<'static> {
        self.location
    }
}

impl<E: BizError> std::fmt::Display for ContextualError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\nContext: {}", self.error, self.context)
    }
}

impl<E: BizError> Error for ContextualError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

impl<E: BizError> BizError for ContextualError<E> {
    fn code(&self) -> u16 {
        self.error.code()
    }

    fn name(&self) -> &str {
        self.error.name()
    }

    fn msg(&self) -> Option<&str> {
        self.error.msg()
    }
}

/// Result extension trait
pub trait ResultExt<T, E> {
    /// Add contextual information
    fn with_biz_context<B>(
        self,
        context: impl Into<String>,
    ) -> Result<T, ContextualError<B>>
    where
        B: BizError + From<E>;
}

impl<T, E: Error + 'static> ResultExt<T, E> for Result<T, E> {
    #[track_caller]
    fn with_biz_context<B>(
        self,
        context: impl Into<String>,
    ) -> Result<T, ContextualError<B>>
    where
        B: BizError + From<E>,
    {
        self.map_err(|e| ContextualError::new(B::from(e), context))
    }
}

/// `BizError` extension trait
pub trait BizErrorExt: BizError + Sized {
    /// Add context
    #[track_caller]
    fn with_context(self, context: impl Into<String>) -> ContextualError<Self> {
        ContextualError::new(self, context)
    }
}

impl<T: BizError> BizErrorExt for T {}

/// HTTP request related errors
#[derive(ThisError)]
pub enum HttpRequestError {
    #[error("Failed to build HTTP request: {0}")]
    RequestBuild(#[from] std::io::Error),

    #[error("HTTP request failed with status {status}: {body}")]
    RequestFailed { status: u16, body: String },

    #[error("Failed to parse response body: {0}")]
    ResponseParse(String),

    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] std::string::FromUtf8Error),

    #[error("Request timeout")]
    Timeout,
}

impl BizError for HttpRequestError {
    fn code(&self) -> u16 {
        match self {
            Self::RequestBuild(_) => 8001,
            Self::RequestFailed { .. } => 8002,
            Self::ResponseParse(_) => 8003,
            Self::InvalidUrl { .. } => 8004,
            Self::Serialization(_) => 8005,
            Self::Timeout => 8006,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::RequestBuild(_) => "RequestBuild",
            Self::RequestFailed { .. } => "RequestFailed",
            Self::ResponseParse(_) => "ResponseParse",
            Self::InvalidUrl { .. } => "InvalidUrl",
            Self::Serialization(_) => "Serialization",
            Self::Timeout => "Timeout",
        }
    }

    fn msg(&self) -> Option<&str> {
        match self {
            Self::RequestBuild(_) => Some("Failed to build HTTP request"),
            Self::RequestFailed { .. } => Some("HTTP request failed"),
            Self::ResponseParse(_) => Some("Failed to parse response body"),
            Self::InvalidUrl { .. } => Some("Invalid URL"),
            Self::Serialization(_) => Some("Serialization error"),
            Self::Timeout => Some("Request timeout"),
        }
    }
}

/// Custom Debug implementation providing detailed error information
impl std::fmt::Debug for HttpRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: Code {}", self.name(), self.code())?;

        if let Some(msg) = self.msg() {
            write!(f, "\nMessage: {msg}")?;
        }

        write!(f, "\nDetails: {self}")?;

        if let Some(source) = self.source() {
            write!(f, "\nCaused by: {source}")?;
        }

        Ok(())
    }
}

/// Demonstrate how to implement `BizError` trait for custom errors
#[derive(Debug, thiserror::Error)]
pub enum CustomBusinessError {
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: u64 },

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Business rule validation failed: {rule}")]
    BusinessRuleViolation { rule: String },
}

impl BizError for CustomBusinessError {
    fn code(&self) -> u16 {
        match self {
            Self::UserNotFound { .. } => 2001,
            Self::InsufficientPermissions => 2002,
            Self::BusinessRuleViolation { .. } => 2003,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::UserNotFound { .. } => "UserNotFound",
            Self::InsufficientPermissions => "InsufficientPermissions",
            Self::BusinessRuleViolation { .. } => "BusinessRuleViolation",
        }
    }

    fn msg(&self) -> Option<&str> {
        match self {
            Self::UserNotFound { .. } => Some("User does not exist"),
            Self::InsufficientPermissions => Some("Insufficient permissions"),
            Self::BusinessRuleViolation { .. } => {
                Some("Business rule validation failed")
            }
        }
    }
}

fn main() {
    println!("🎯 BizError Trait Approach Examples");
    println!("===================================");

    // Example 1: Basic usage - only need error codes
    basic_usage_example();

    // Example 2: Contextual usage - need detailed information
    contextual_usage_example();

    // Example 3: Debug output demonstration
    debug_output_example();

    // Example 4: Result extension usage
    result_extension_example();

    // Example 5: Custom business error demonstration
    custom_business_error_example();
}

/// Example 1: Basic usage scenarios (90% of use cases)
fn basic_usage_example() {
    println!("\n=== Basic Usage Example ===");

    let error = HttpRequestError::Timeout;

    println!("Error code: {}", error.code()); // 8006
    println!("Error name: {}", error.name()); // Timeout
    println!("Error message: {:?}", error.msg()); // Some("Request timeout")
    println!("Display: {error}"); // Request timeout

    // Return error directly in function
    let result = simple_http_call();
    match result {
        Ok(response) => println!("Success: {response}"),
        Err(e) => {
            println!("Failed - Error code: {}, Type: {}", e.code(), e.name());
        }
    }
}

/// Example 2: Contextual usage scenarios (10% of use cases)
fn contextual_usage_example() {
    println!("\n=== Contextual Usage Example ===");

    // Method 1: Add context directly to error
    let error = HttpRequestError::InvalidUrl {
        url: "invalid-url".to_string(),
    };
    let contextual = error.with_context("Calling user service API");

    println!("Error code: {}", contextual.code()); // 8004
    println!("Error name: {}", contextual.name()); // InvalidUrl
    println!("Context: {}", contextual.context()); // Calling user service API
    println!("Location: {}", contextual.location()); // Show call location
    println!("Display:\n{contextual}"); // Full information with context

    // Method 2: Use through Result extension
    let result = complex_http_call();
    match result {
        Ok(response) => println!("Success: {response}"),
        Err(e) => {
            println!("Failure details:");
            println!("  Error code: {}", e.code());
            println!("  Error type: {}", e.name());
            println!("  Context: {}", e.context());
            println!("  Location: {}", e.location());
            if let Some(source) = e.source() {
                println!("  Original error: {source}");
            }
        }
    }
}

/// Example 3: Debug output demonstration
fn debug_output_example() {
    println!("\n=== Debug Output Example ===");

    // Create an error with source
    let io_error = std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Config file not found",
    );
    let http_error = HttpRequestError::from(io_error);

    println!("Standard Debug output:");
    println!("{http_error:?}");

    println!("\nContextual Debug output:");
    let contextual =
        http_error.with_context("Loading application configuration");
    println!("{contextual:?}");
}

/// Example 4: Result extension usage
fn result_extension_example() {
    println!("\n=== Result Extension Usage Example ===");

    // Chain calls, automatically add context and location information
    let result = perform_complex_operation();
    match result {
        Ok(data) => println!("Operation successful: {data}"),
        Err(e) => {
            println!("Operation failed:");
            println!("  Error message: {e}");
            println!("  Business error code: {}", e.code());
            println!("  Occurred at: {}", e.location());
        }
    }
}

/// Example 5: Custom business error demonstration
fn custom_business_error_example() {
    println!("\n=== Custom Business Error Example ===");

    let user_error = CustomBusinessError::UserNotFound { user_id: 12345 };
    println!(
        "User Error - Code: {}, Name: {}",
        user_error.code(),
        user_error.name()
    );
    println!("User Error - Message: {:?}", user_error.msg());

    let permission_error = CustomBusinessError::InsufficientPermissions;
    println!(
        "Permission Error - Code: {}, Name: {}",
        permission_error.code(),
        permission_error.name()
    );

    // Add context to business error
    let contextual_error =
        user_error.with_context("During user profile update");
    println!(
        "Contextual Error - Code: {}, Context: {}",
        contextual_error.code(),
        contextual_error.context()
    );
}

/// Simple HTTP call example
const fn simple_http_call() -> Result<String, HttpRequestError> {
    // Simulate network timeout
    Err(HttpRequestError::Timeout)
}

/// Complex HTTP call example (needs context)
fn complex_http_call() -> Result<String, ContextualError<HttpRequestError>> {
    // Simulate IO error and add business context
    std::fs::read_to_string("nonexistent.txt")
        .with_biz_context("Reading user configuration file")
}

/// Complex operation example
fn perform_complex_operation()
-> Result<String, ContextualError<HttpRequestError>> {
    // Step 1: Read configuration
    let _config = std::fs::read_to_string("config.json")
        .with_biz_context("Loading system configuration")?;

    // Step 2: Make network request (simulate failure)
    let _response = make_network_request()
        .with_biz_context("Fetching user data from external API")?;

    Ok("Success".to_string())
}

fn make_network_request() -> Result<String, std::io::Error> {
    // Simulate network error
    Err(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "Connection refused",
    ))
}
