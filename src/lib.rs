//! # `BizError` - Structured Business Error Handling for Rust
//!
//! A lightweight, flexible business error handling library that provides
//! structured error codes and contextual information while maintaining full
//! compatibility with Rust's error ecosystem.
//!
//! ## 🎯 Design Philosophy
//!
//! **90/10 Principle**: 90% of error handling scenarios only need error codes,
//! while 10% require detailed context information.
//!
//! - **Minimal Core**: `BizError` trait contains only essential business error
//!   identification
//! - **Optional Context**: Use `ContextualError` wrapper only when detailed
//!   context is needed
//! - **Zero Overhead**: Basic usage scenarios have no additional performance
//!   cost
//! - **Full Compatibility**: Seamlessly integrates with thiserror and the
//!   entire Rust error ecosystem
//!
//! ## 🚀 Basic Usage with Derive Macro
//!
//! ```rust
//! use bizerror::BizError;
//!
//! #[derive(BizError, thiserror::Error)]
//! pub enum ApiError {
//!     #[bizcode(4001)]
//!     #[error("Invalid input: {field}")]
//!     ValidationError { field: String },
//!
//!     #[bizcode(8001)]
//!     #[error("Database connection failed")]
//!     DatabaseError(#[from] std::io::Error),
//!
//!     #[bizcode(8006)]
//!     #[error("Request timeout")]
//!     Timeout,
//! }
//!
//! // Use the error
//! let error = ApiError::ValidationError { field: "email".to_string() };
//! assert_eq!(error.code(), 4001);
//! assert_eq!(error.name(), "ValidationError");
//! assert_eq!(error.msg(), "Invalid input: email"); // Uses Display implementation
//! ```
//!
//! ## 🔧 Advanced Usage with Context
//!
//! ```rust
//! use bizerror::*;
//!
//! #[derive(BizError, thiserror::Error)]
//! pub enum ApiError {
//!     #[bizcode(8001)]
//!     #[error("Database connection failed")]
//!     DatabaseError(#[from] std::io::Error),
//! }
//!
//! fn load_user_config() -> Result<String, ContextualError<ApiError>> {
//!     std::fs::read_to_string("config.json")
//!         .with_biz_context("Loading user configuration")
//! }
//!
//! # fn example_usage() {
//! match load_user_config() {
//!     Ok(config) => println!("Config loaded: {}", config),
//!     Err(e) => {
//!         println!("Error code: {}", e.code());
//!         println!("Context: {}", e.context());
//!         println!("Location: {}", e.location());
//!     }
//! }
//! # }
//! ```

use core::panic::Location;
use std::error::Error;

// Re-export the BizError derive macro
pub use bizerror_impl::BizError;

/// Core business error trait
///
/// This trait provides the essential functionality for business error
/// identification:
/// - `code()`: Returns a unique business error code
/// - `name()`: Returns the error type name (typically the enum variant name)
/// - `msg()`: Returns the error message using the Display implementation
pub trait BizError: Error + Send + Sync + 'static {
    /// The type of the error code
    type CodeType: Copy
        + std::fmt::Display
        + std::fmt::Debug
        + Send
        + Sync
        + 'static;

    /// Get the business error code
    fn code(&self) -> Self::CodeType;

    /// Get the error type name
    fn name(&self) -> &str;

    /// Get the business error message (uses Display implementation)
    fn msg(&self) -> String {
        self.to_string()
    }
}

/// Contextual error wrapper (only used when detailed context is needed)
///
/// This wrapper allows you to add context information and automatic location
/// tracking to any `BizError` without changing the original error type.
#[derive(Debug)]
pub struct ContextualError<E: BizError> {
    error:    E,
    context:  String,
    location: &'static Location<'static>,
}

impl<E: BizError> ContextualError<E> {
    /// Create a new contextual error with automatic location tracking
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
    type CodeType = E::CodeType;

    fn code(&self) -> Self::CodeType {
        self.error.code()
    }

    fn name(&self) -> &str {
        self.error.name()
    }

    fn msg(&self) -> String {
        self.error.msg()
    }
}

/// Result extension trait
///
/// Provides convenient methods to add business context to any Result.
pub trait ResultExt<T, E> {
    /// Add contextual information and convert to `ContextualError`
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
///
/// Provides convenient methods to add context to any `BizError`.
pub trait BizErrorExt: BizError + Sized {
    /// Add context with automatic location tracking
    #[track_caller]
    fn with_context(self, context: impl Into<String>) -> ContextualError<Self> {
        ContextualError::new(self, context)
    }
}

impl<T: BizError> BizErrorExt for T {}
