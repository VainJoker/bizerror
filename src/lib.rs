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
//! The simplest way to use `BizError` is with the derive macro:
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
//! assert_eq!(error.to_string(), "Invalid input: email"); // Uses Display implementation
//! ```
//!
//! ## 🏗️ Automatic Code Assignment
//!
//! You can configure automatic code assignment for variants without explicit
//! codes:
//!
//! ```rust
//! use bizerror::BizError;
//!
//! #[derive(BizError, thiserror::Error)]
//! #[bizconfig(auto_start = 1000, auto_increment = 10)]
//! pub enum ServiceError {
//!     #[error("Auto-assigned code")]
//!     AutoError1, // code: 1000
//!
//!     #[bizcode(2001)]
//!     #[error("Explicit code")]
//!     ExplicitError, // code: 2001
//!
//!     #[error("Another auto-assigned")]
//!     AutoError2, // code: 1010
//! }
//! ```
//!
//! ## 🔧 Advanced Usage with Context
//!
//! For scenarios requiring detailed context information:
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
//!         .with_context("Loading user configuration")
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
//!
//! ## 📊 Custom Code Types
//!
//! You can use different types for error codes:
//!
//! ```rust
//! use bizerror::BizError;
//!
//! // String codes
//! #[derive(BizError, thiserror::Error)]
//! #[bizconfig(code_type = "&'static str")]
//! pub enum StringError {
//!     #[bizcode("USER_NOT_FOUND")]
//!     #[error("User not found")]
//!     UserNotFound,
//!
//!     #[error("Auto string code")]
//!     AutoString, // code: "0"
//! }
//!
//! // Signed integer codes
//! #[derive(BizError, thiserror::Error)]
//! #[bizconfig(code_type = "i32", auto_start = -100)]
//! pub enum SignedError {
//!     #[error("Negative code")]
//!     NegativeCode, // code: -100
//! }
//! ```
//!
//! ## 🎨 Structured Debug Output
//!
//! The derive macro automatically generates structured debug output:
//!
//! ```rust
//! # use bizerror::BizError;
//! # #[derive(BizError, thiserror::Error)]
//! # pub enum ApiError {
//! #     #[bizcode(4001)]
//! #     #[error("Invalid input: {field}")]
//! #     ValidationError { field: String },
//! # }
//! let error = ApiError::ValidationError { field: "email".to_string() };
//! println!("{:?}", error);
//! // Output: ApiError { variant: "ValidationError", code: 4001, message: "Invalid input: email" }
//! ```
//!
//! ## 🔗 Error Chains and Context
//!
//! Build comprehensive error chains with context:
//!
//! ```rust
//! use bizerror::*;
//!
//! #[derive(BizError, thiserror::Error)]
//! pub enum ServiceError {
//!     #[bizcode(8001)]
//!     #[error("Database error: {0}")]
//!     DatabaseError(#[from] std::io::Error),
//! }
//!
//! fn complex_operation() -> Result<String, ContextualError<ServiceError>> {
//!     // Multiple layers of context
//!     std::fs::read_to_string("data.json")
//!         .with_context("Loading configuration")
//!         .and_then(|_| {
//!             std::fs::read_to_string("user.json")
//!                 .with_context("Loading user data")
//!         })
//! }
//! ```
//!
//! ## 🏆 Best Practices
//!
//! 1. **Use meaningful error codes**: Group related errors by code ranges
//!    - 1000-1999: Validation errors
//!    - 2000-2999: Authentication errors
//!    - 8000-8999: System errors
//!
//! 2. **Leverage automatic assignment**: Use `bizconfig` for consistent code
//!    spacing
//!
//! 3. **Add context sparingly**: Only use `ContextualError` when you need
//!    detailed debugging
//!
//! 4. **Chain errors properly**: Use `#[from]` for automatic conversions
//!
//! 5. **Document error codes**: Include code meanings in your API documentation

use core::panic::Location;
use std::{
    borrow::Cow,
    error::Error,
};

// Re-export the BizError derive macro
pub use bizerror_impl::BizError;

/// Core business error trait
///
/// This trait provides the essential functionality for business error
/// identification:
/// - `code()`: Returns a unique business error code
/// - `name()`: Returns the error type name (typically the enum variant name)
///
/// For error messages, use the standard `Display` trait implementation.
///
/// ## Example Implementation
///
/// ```rust
/// use std::error::Error;
///
/// use bizerror::BizError;
///
/// #[derive(Debug)]
/// pub struct CustomError {
///     code:    u32,
///     message: String,
/// }
///
/// impl std::fmt::Display for CustomError {
///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
///         write!(f, "{}", self.message)
///     }
/// }
///
/// impl Error for CustomError {}
///
/// impl BizError for CustomError {
///     type CodeType = u32;
///
///     fn code(&self) -> Self::CodeType {
///         self.code
///     }
///
///     fn name(&self) -> &str {
///         "CustomError"
///     }
/// }
/// ```
pub trait BizError: Error + Send + Sync + 'static {
    /// The type of the error code
    ///
    /// Can be any type that implements `Copy + Display + Debug + Send + Sync +
    /// 'static`. Common choices include:
    /// - `u32` or `u16` for numeric codes
    /// - `&'static str` for string codes
    /// - `i32` for signed numeric codes
    type CodeType: Copy
        + std::fmt::Display
        + std::fmt::Debug
        + Send
        + Sync
        + std::hash::Hash
        + PartialEq
        + Eq
        + 'static;

    /// Get the business error code
    ///
    /// This should return a unique identifier for this specific error type.
    /// The code should be consistent across different instances of the same
    /// error variant.
    fn code(&self) -> Self::CodeType;

    /// Get the error type name
    ///
    /// This typically returns the enum variant name for derived
    /// implementations. For custom implementations, this should return a
    /// consistent, descriptive name.
    fn name(&self) -> &str;
}

/// Contextual error wrapper (only used when detailed context is needed)
///
/// This wrapper allows you to add context information and automatic location
/// tracking to any `BizError` without changing the original error type.
///
/// ## When to Use
///
/// Use `ContextualError` when you need:
/// - Detailed debugging information
/// - Location tracking for where the error occurred
/// - Additional context about the operation that failed
/// - Multiple layers of context in error chains
///
/// ## Example
///
/// ```rust
/// use bizerror::*;
///
/// #[derive(BizError, thiserror::Error)]
/// pub enum ServiceError {
///     #[bizcode(8001)]
///     #[error("Database connection failed")]
///     DatabaseError(#[from] std::io::Error),
/// }
///
/// fn load_config() -> Result<String, ContextualError<ServiceError>> {
///     std::fs::read_to_string("config.json")
///         .with_context("Loading application configuration")
/// }
/// ```
pub struct ContextualError<E: BizError> {
    error:    E,
    context:  Cow<'static, str>, // Avoids allocation for static strings,
    location: &'static Location<'static>,
}

impl<E: BizError> ContextualError<E> {
    /// Create a new contextual error with automatic location tracking
    ///
    /// The location is automatically captured using `#[track_caller]`,
    /// providing precise information about where the error context was added.
    #[track_caller]
    pub fn new(error: E, context: impl Into<String>) -> Self {
        Self {
            error,
            context: Cow::Owned(context.into()),
            location: Location::caller(),
        }
    }

    /// Get the original error
    ///
    /// This provides access to the underlying `BizError` instance.
    pub const fn inner(&self) -> &E {
        &self.error
    }

    /// Get the context
    ///
    /// Returns the contextual information that was added to this error.
    pub fn context(&self) -> &str {
        &self.context
    }

    /// Get the location
    ///
    /// Returns the location where the context was added to this error.
    pub const fn location(&self) -> &'static Location<'static> {
        self.location
    }

    /// Add additional context to the existing context
    ///
    /// This method appends new context information to the existing context,
    /// creating a layered context description.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bizerror::*;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum MyError {
    ///     #[bizcode(8001)]
    ///     #[error("IO error")]
    ///     IoError,
    /// }
    ///
    /// let error = MyError::IoError;
    /// let contextual = error.with_context("Loading file");
    /// let layered = contextual.add_context("During startup");
    /// assert_eq!(layered.context(), "Loading file -> During startup");
    /// ```
    #[track_caller]
    #[must_use]
    pub fn add_context(self, additional: impl Into<String>) -> Self {
        let new_context = format!("{} -> {}", self.context, additional.into());
        Self {
            error:    self.error,
            context:  Cow::Owned(new_context),
            location: Location::caller(),
        }
    }

    /// Unwrap the contextual error, returning the inner error
    ///
    /// This method consumes the `ContextualError` and returns the underlying
    /// business error, discarding the context information.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bizerror::*;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum MyError {
    ///     #[bizcode(8001)]
    ///     #[error("IO error")]
    ///     IoError,
    /// }
    ///
    /// let error = MyError::IoError;
    /// let contextual = error.with_context("Some context");
    /// let original = contextual.into_inner();
    /// // original is now MyError::IoError again
    /// ```
    pub fn into_inner(self) -> E {
        self.error
    }

    /// Find the first error in the chain of a specific type
    ///
    /// This method traverses the error chain and returns the first error
    /// of the specified type. Useful for extracting specific error types
    /// from a complex error chain.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io;
    ///
    /// use bizerror::*;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum MyError {
    ///     #[bizcode(8001)]
    ///     #[error("IO error: {0}")]
    ///     IoError(#[from] io::Error),
    /// }
    ///
    /// let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let my_error = MyError::IoError(io_error);
    /// let contextual = my_error.with_context("Loading config");
    ///
    /// // Find the original io::Error in the chain
    /// let found_io_error = contextual.find_root::<io::Error>();
    /// assert!(found_io_error.is_some());
    /// ```
    pub fn find_root<T>(&self) -> Option<&T>
    where
        T: Error + 'static,
    {
        let mut current: &dyn Error = self;
        while let Some(source) = current.source() {
            if let Some(target) = source.downcast_ref::<T>() {
                return Some(target);
            }
            current = source;
        }
        None
    }

    /// Count the depth of the error chain
    ///
    /// Returns the number of errors in the chain, including this error.
    /// Useful for understanding the complexity of error propagation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bizerror::*;
    /// use std::io;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum MyError {
    ///     #[bizcode(8001)]
    ///     #[error("IO error: {0}")]
    ///     IoError(#[from] io::Error),
    /// }
    ///
    /// let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let my_error = MyError::IoError(io_error);
    /// let contextual = my_error.with_context("Loading config");
    ///
    /// assert_eq!(contextual.chain_depth(), 3); // ContextualError -> MyError -> io::Error
    /// ```
    pub fn chain_depth(&self) -> usize {
        let mut depth = 1;
        let mut current: &dyn Error = self;
        while let Some(source) = current.source() {
            depth += 1;
            current = source;
        }
        depth
    }

    /// Get the root cause message of the error chain
    ///
    /// Returns the deepest error message in the chain, which is typically the
    /// original cause of the error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io;
    ///
    /// use bizerror::*;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum MyError {
    ///     #[bizcode(8001)]
    ///     #[error("IO error: {0}")]
    ///     IoError(#[from] io::Error),
    /// }
    ///
    /// let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let my_error = MyError::IoError(io_error);
    /// let contextual = my_error.with_context("Loading config");
    ///
    /// let root_cause = contextual.root_cause_message();
    /// assert_eq!(root_cause, "file not found");
    /// ```
    pub fn root_cause_message(&self) -> String {
        let mut current: &dyn Error = self;
        while let Some(source) = current.source() {
            current = source;
        }
        current.to_string()
    }

    /// Collect all error messages in the chain
    ///
    /// Returns a vector of all error messages in the chain, from the current
    /// error to the root cause. Useful for comprehensive error reporting.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io;
    ///
    /// use bizerror::*;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum MyError {
    ///     #[bizcode(8001)]
    ///     #[error("IO error: {0}")]
    ///     IoError(#[from] io::Error),
    /// }
    ///
    /// let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let my_error = MyError::IoError(io_error);
    /// let contextual = my_error.with_context("Loading config");
    ///
    /// let chain = contextual.error_chain_messages();
    /// assert_eq!(chain.len(), 3);
    /// ```
    pub fn error_chain_messages(&self) -> Vec<String> {
        let mut chain = vec![self.to_string()];
        let mut current = self.source();
        while let Some(source) = current {
            chain.push(source.to_string());
            current = source.source();
        }
        chain
    }

    /// Check if the error chain contains a specific error type
    ///
    /// Returns true if any error in the chain is of the specified type.
    /// Useful for conditional error handling.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::io;
    ///
    /// use bizerror::*;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum MyError {
    ///     #[bizcode(8001)]
    ///     #[error("IO error: {0}")]
    ///     IoError(#[from] io::Error),
    /// }
    ///
    /// let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let my_error = MyError::IoError(io_error);
    /// let contextual = my_error.with_context("Loading config");
    ///
    /// assert!(contextual.contains_error::<io::Error>());
    /// assert!(!contextual.contains_error::<std::fmt::Error>());
    /// ```
    pub fn contains_error<T>(&self) -> bool
    where
        T: Error + 'static,
    {
        self.find_root::<T>().is_some()
    }

    /// Check if the error chain contains a specific business error code
    ///
    /// Returns true if any `BizError` in the chain has the specified code.
    /// Useful for conditional error handling based on business error codes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bizerror::*;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum MyError {
    ///     #[bizcode(8001)]
    ///     #[error("IO error")]
    ///     IoError,
    ///
    ///     #[bizcode(8002)]
    ///     #[error("Network error")]
    ///     NetworkError,
    /// }
    ///
    /// let error = MyError::IoError;
    /// let contextual = error.with_context("Operation failed");
    ///
    /// assert!(contextual.chain_contains_code(8001));
    /// assert!(!contextual.chain_contains_code(8002));
    /// ```
    pub fn chain_contains_code<C>(&self, code: C) -> bool
    where
        C: PartialEq<E::CodeType> + Copy,
    {
        let mut current: &dyn Error = self;
        loop {
            if let Some(biz_error) = current.downcast_ref::<E>() &&
                code == biz_error.code()
            {
                return true;
            }
            if let Some(contextual) = current.downcast_ref::<Self>() &&
                code == contextual.error.code()
            {
                return true;
            }
            if let Some(source) = current.source() {
                current = source;
            } else {
                break;
            }
        }
        false
    }
}

impl<E: BizError> std::fmt::Debug for ContextualError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextualError")
            .field("type", &self.error.name())
            .field("code", &self.error.code())
            .field("message", &self.error.to_string())
            .field("context", &self.context.as_ref())
            .field(
                "location",
                &format!(
                    "{}:{}:{}",
                    self.location.file(),
                    self.location.line(),
                    self.location.column()
                ),
            )
            .finish()
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
}

/// Result extension trait (simplified)
///
/// Provides convenient methods to add business context to any Result.
/// This trait is automatically implemented for all `Result<T, E>` types
/// where `E` implements `Error`.
///
/// ## Core Methods
///
/// - `with_context()` - Add context and convert to `ContextualError`
/// - `map_biz()` - Simple error type conversion
/// - `with_context_if()` - Conditional context addition
///
/// ## Example
///
/// ```rust
/// use bizerror::*;
///
/// #[derive(BizError, thiserror::Error)]
/// pub enum MyError {
///     #[bizcode(8001)]
///     #[error("IO error: {0}")]
///     IoError(#[from] std::io::Error),
/// }
///
/// fn read_file() -> Result<String, ContextualError<MyError>> {
///     std::fs::read_to_string("important.txt")
///         .with_context("Reading critical configuration file")
/// }
/// ```
pub trait ResultExt<T, E> {
    /// Add contextual information and convert to `ContextualError`
    ///
    /// This method allows you to add context to any `Result` that contains
    /// an error that can be converted to your business error type.
    ///
    /// The context is captured with automatic location tracking.
    fn with_context<B>(
        self,
        context: impl Into<String>,
    ) -> Result<T, ContextualError<B>>
    where
        B: BizError + From<E>;

    /// Convert error type without adding context
    ///
    /// This is a convenience method that converts the error type to a business
    /// error. It's equivalent to `.map_err(B::from)`.
    fn map_biz<B>(self) -> Result<T, B>
    where
        B: BizError + From<E>;

    /// Add context conditionally
    ///
    /// This method adds context only when the condition is true.
    /// If the condition is false, it still converts the error type but without
    /// context.
    fn with_context_if<B>(
        self,
        condition: bool,
        context: impl Into<String>,
    ) -> Result<T, ContextualError<B>>
    where
        B: BizError + From<E>;

    /// Chain operations with error conversion
    ///
    /// This method allows you to chain operations while converting errors
    /// to business error types. It's a convenience method that combines
    /// `and_then` with automatic error type conversion.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bizerror::*;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum MyError {
    ///     #[bizcode(8001)]
    ///     #[error("IO error: {0}")]
    ///     IoError(#[from] std::io::Error),
    /// }
    ///
    /// let initial_err: Result<u32, std::io::Error> = Err(std::io::Error::new(
    ///     std::io::ErrorKind::BrokenPipe,
    ///     "pipe broken",
    /// ));
    /// let chained_err: Result<String, MyError> =
    ///     initial_err.and_then_biz(|val| Ok(format!("Value is {val}")));
    /// assert!(chained_err.is_err()); // true
    /// ```
    fn and_then_biz<U, F, B>(self, f: F) -> Result<U, B>
    where
        F: FnOnce(T) -> Result<U, B>,
        B: BizError + From<E>;
}

impl<T, E: Error + 'static> ResultExt<T, E> for Result<T, E> {
    #[track_caller]
    fn with_context<B>(
        self,
        context: impl Into<String>,
    ) -> Result<T, ContextualError<B>>
    where
        B: BizError + From<E>,
    {
        self.map_err(|e| ContextualError::new(B::from(e), context))
    }

    fn map_biz<B>(self) -> Result<T, B>
    where
        B: BizError + From<E>,
    {
        self.map_err(|e| B::from(e))
    }

    fn with_context_if<B>(
        self,
        condition: bool,
        context: impl Into<String>,
    ) -> Result<T, ContextualError<B>>
    where
        B: BizError + From<E>,
    {
        if condition {
            self.with_context(context)
        } else {
            self.map_err(|e| {
                ContextualError::new(B::from(e), "no context".to_string())
            })
        }
    }

    fn and_then_biz<U, F, B>(self, f: F) -> Result<U, B>
    where
        F: FnOnce(T) -> Result<U, B>,
        B: BizError + From<E>,
    {
        match self {
            Ok(t) => f(t),
            Err(e) => Err(B::from(e)),
        }
    }
}

/// `BizError` extension trait
///
/// Provides convenient methods for adding context to business errors.
/// This trait is automatically implemented for all types that implement
/// `BizError`.
///
/// ## Example
///
/// ```rust
/// use bizerror::*;
///
/// #[derive(BizError, thiserror::Error)]
/// pub enum ApiError {
///     #[bizcode(4001)]
///     #[error("Validation failed")]
///     ValidationError,
/// }
///
/// let error = ApiError::ValidationError;
/// let contextual = error.with_context("Processing user registration");
/// ```
pub trait BizErrorExt: BizError + Sized {
    /// Add context with automatic location tracking
    ///
    /// This method wraps the error in a `ContextualError` with the provided
    /// context and automatic location tracking.
    #[track_caller]
    fn with_context(self, context: impl Into<String>) -> ContextualError<Self> {
        ContextualError::new(self, context)
    }
}

impl<T: BizError> BizErrorExt for T {}

/// Option extension trait
///
/// Provides convenient methods to convert `Option` to `Result` with business
/// errors. This trait is automatically implemented for all `Option<T>` types.
///
/// ## Example
///
/// ```rust
/// use bizerror::*;
///
/// #[derive(BizError, thiserror::Error)]
/// pub enum MyError {
///     #[bizcode(4001)]
///     #[error("Value not found")]
///     NotFound,
/// }
///
/// let value: Option<String> = None;
/// let result = value.ok_or_biz(MyError::NotFound);
/// assert!(result.is_err());
/// ```
pub trait OptionExt<T> {
    /// Convert `Option<T>` to `Result<T, B>` with a business error
    ///
    /// This is a convenience method that converts `None` to an error
    /// of the specified business error type.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bizerror::*;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum MyError {
    ///     #[bizcode(4001)]
    ///     #[error("User not found")]
    ///     UserNotFound,
    /// }
    ///
    /// fn find_user(id: u32) -> Option<String> {
    ///     None // simulate not found
    /// }
    ///
    /// let result = find_user(123).ok_or_biz(MyError::UserNotFound);
    /// assert!(result.is_err());
    /// assert_eq!(result.unwrap_err().code(), 4001);
    /// ```
    fn ok_or_biz<B>(self, error: B) -> Result<T, B>
    where
        B: BizError;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_biz<B>(self, error: B) -> Result<T, B>
    where
        B: BizError,
    {
        self.ok_or(error)
    }
}

/// Business errors collection for aggregating multiple errors
///
/// This type is useful for scenarios where you need to collect all errors
/// instead of failing on the first one, such as form validation or batch
/// processing.
///
/// ## Example
///
/// ```rust
/// use bizerror::*;
///
/// #[derive(BizError, thiserror::Error)]
/// pub enum ValidationError {
///     #[bizcode(4001)]
///     #[error("Invalid email: {email}")]
///     InvalidEmail { email: String },
///
///     #[bizcode(4002)]
///     #[error("Password too short")]
///     PasswordTooShort,
/// }
///
/// fn validate_user(
///     email: &str,
///     password: &str,
/// ) -> Result<(), BizErrors<ValidationError>> {
///     let mut errors = BizErrors::new();
///
///     if !email.contains('@') {
///         errors.push_simple(ValidationError::InvalidEmail {
///             email: email.to_string(),
///         });
///     }
///
///     if password.len() < 8 {
///         errors.push_simple(ValidationError::PasswordTooShort);
///     }
///
///     if errors.is_empty() {
///         Ok(())
///     } else {
///         Err(errors)
///     }
/// }
/// ```
pub struct BizErrors<E: BizError> {
    errors: Vec<ContextualError<E>>,
}

impl<E: BizError> BizErrors<E> {
    /// Create a new empty error collection
    pub const fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Create a new error collection with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            errors: Vec::with_capacity(capacity),
        }
    }

    /// Add a contextual error to the collection
    pub fn push(&mut self, error: ContextualError<E>) {
        self.errors.push(error);
    }

    /// Add a simple business error to the collection
    ///
    /// The error will be wrapped in a `ContextualError` with minimal context.
    #[track_caller]
    pub fn push_simple(&mut self, error: E) {
        self.errors.push(ContextualError::new(error, ""));
    }

    /// Add a business error with context to the collection
    #[track_caller]
    pub fn push_with_context(&mut self, error: E, context: impl Into<String>) {
        self.errors.push(ContextualError::new(error, context));
    }

    /// Get the number of errors in the collection
    pub const fn len(&self) -> usize {
        self.errors.len()
    }

    /// Check if the error collection is empty
    pub const fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Get an iterator over the errors
    pub fn iter(&self) -> impl Iterator<Item = &ContextualError<E>> {
        self.errors.iter()
    }

    /// Get a reference to the errors vector
    pub fn as_slice(&self) -> &[ContextualError<E>] {
        &self.errors
    }

    /// Convert into the underlying errors vector
    pub fn into_vec(self) -> Vec<ContextualError<E>> {
        self.errors
    }

    /// Get the first error in the collection
    pub fn first(&self) -> Option<&ContextualError<E>> {
        self.errors.first()
    }

    /// Get the last error in the collection
    pub fn last(&self) -> Option<&ContextualError<E>> {
        self.errors.last()
    }

    /// Collect successful results and errors from an iterator
    ///
    /// Returns a tuple containing all successful values and optionally
    /// the collected errors (if any occurred).
    ///
    /// # Example
    ///
    /// ```rust
    /// use bizerror::*;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum ProcessError {
    ///     #[bizcode(5001)]
    ///     #[error("Invalid value: {value}")]
    ///     InvalidValue { value: i32 },
    /// }
    ///
    /// let results: Vec<Result<i32, ContextualError<ProcessError>>> = vec![
    ///     Ok(1),
    ///     Ok(2),
    ///     Err(ProcessError::InvalidValue { value: 3 }
    ///         .with_context("Processing item 3")),
    ///     Ok(4),
    ///     Err(ProcessError::InvalidValue { value: 5 }
    ///         .with_context("Processing item 5")),
    /// ];
    ///
    /// let (successes, errors) = BizErrors::collect_from(results.into_iter());
    /// assert_eq!(successes, vec![1, 2, 4]);
    /// assert!(errors.is_some());
    /// assert_eq!(errors.unwrap().len(), 2);
    /// ```
    pub fn collect_from<T, I>(iter: I) -> (Vec<T>, Option<Self>)
    where
        I: Iterator<Item = Result<T, ContextualError<E>>>,
    {
        let mut successes = Vec::new();
        let mut errors = Self::new();

        for result in iter {
            match result {
                Ok(value) => successes.push(value),
                Err(error) => errors.push(error),
            }
        }

        let errors = if errors.is_empty() {
            None
        } else {
            Some(errors)
        };

        (successes, errors)
    }

    /// Collect all errors from an iterator of Results
    ///
    /// Returns `None` if no errors occurred, or `Some(BizErrors)` with all
    /// errors.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bizerror::*;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum ValidationError {
    ///     #[bizcode(4001)]
    ///     #[error("Invalid field")]
    ///     InvalidField,
    /// }
    ///
    /// let results: Vec<Result<(), ContextualError<ValidationError>>> = vec![
    ///     Ok(()),
    ///     Err(ValidationError::InvalidField.with_context("Field 1")),
    ///     Err(ValidationError::InvalidField.with_context("Field 2")),
    /// ];
    ///
    /// let errors = BizErrors::collect_errors(results.into_iter());
    /// assert!(errors.is_some());
    /// assert_eq!(errors.unwrap().len(), 2);
    /// ```
    pub fn collect_errors<T, I>(iter: I) -> Option<Self>
    where
        I: Iterator<Item = Result<T, ContextualError<E>>>,
    {
        let mut errors = Self::new();

        for result in iter {
            if let Err(error) = result {
                errors.push(error);
            }
        }

        if errors.is_empty() {
            None
        } else {
            Some(errors)
        }
    }

    /// Check if any error in the collection has the specified code
    pub fn contains_code<C>(&self, code: C) -> bool
    where
        C: PartialEq<E::CodeType> + Copy,
    {
        self.errors.iter().any(|error| code == error.code())
    }

    /// Get all unique error codes in the collection
    pub fn error_codes(&self) -> Vec<E::CodeType> {
        let mut codes: Vec<E::CodeType> =
            self.errors.iter().map(BizError::code).collect();
        codes.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
        codes.dedup();
        codes
    }

    /// Filter errors by a predicate
    ///
    /// Returns an iterator over the errors that satisfy the given predicate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bizerror::*;
    ///
    /// #[derive(BizError, thiserror::Error)]
    /// pub enum MyError {
    ///     #[bizcode(4001)]
    ///     #[error("Validation error")]
    ///     ValidationError,
    ///
    ///     #[bizcode(8001)]
    ///     #[error("System error")]
    ///     SystemError,
    /// }
    ///
    /// let mut errors = BizErrors::new();
    /// errors.push_simple(MyError::ValidationError);
    /// errors.push_simple(MyError::SystemError);
    ///
    /// // Filter only validation errors (4xxx codes)
    /// let validation_errors: Vec<_> = errors
    ///     .filter(|e| e.code() >= 4000 && e.code() < 5000)
    ///     .collect();
    /// assert_eq!(validation_errors.len(), 1);
    /// ```
    pub fn filter<F>(
        &self,
        predicate: F,
    ) -> impl Iterator<Item = &ContextualError<E>>
    where
        F: Fn(&ContextualError<E>) -> bool,
    {
        self.errors.iter().filter(move |e| predicate(*e))
    }
}

impl<E: BizError> Default for BizErrors<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: BizError> IntoIterator for BizErrors<E> {
    type Item = ContextualError<E>;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

impl<E: BizError> std::fmt::Debug for BizErrors<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.errors.is_empty() {
            f.debug_struct("BizErrors").field("count", &0).finish()
        } else if self.errors.len() == 1 {
            f.debug_struct("BizErrors")
                .field("count", &1)
                .field("error", &self.errors[0])
                .finish()
        } else {
            let mut debug_struct = f.debug_struct("BizErrors");
            debug_struct.field("count", &self.errors.len());

            let codes: Vec<_> =
                self.errors.iter().map(BizError::code).collect();
            debug_struct.field("codes", &codes);

            // Show first few errors for detailed view
            if self.errors.len() <= 3 {
                debug_struct.field("errors", &self.errors);
            } else {
                debug_struct.field("first_3_errors", &&self.errors[0..3]);
                debug_struct.field(
                    "note",
                    &format!("... and {} more", self.errors.len() - 3),
                );
            }

            debug_struct.finish()
        }
    }
}

impl<E: BizError> std::fmt::Display for BizErrors<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.errors.is_empty() {
            write!(f, "No errors")
        } else if self.errors.len() == 1 {
            write!(f, "{}", self.errors[0])
        } else {
            writeln!(
                f,
                "Multiple errors occurred ({} total):",
                self.errors.len()
            )?;
            for (i, error) in self.errors.iter().enumerate() {
                writeln!(f, "  {}. {}", i + 1, error)?;
            }
            Ok(())
        }
    }
}

impl<E: BizError> Error for BizErrors<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Return the first error as the source
        self.errors.first().map(|e| e as &dyn Error)
    }
}

impl<E: BizError> BizError for BizErrors<E> {
    type CodeType = E::CodeType;

    fn code(&self) -> Self::CodeType {
        // Return the code of the first error
        self.errors
            .first()
            .map_or_else(|| panic!("BizErrors is empty"), BizError::code)
    }

    fn name(&self) -> &'static str {
        "BizErrors"
    }
}

impl<'a, E: BizError> IntoIterator for &'a BizErrors<E> {
    type Item = &'a ContextualError<E>;
    type IntoIter = std::slice::Iter<'a, ContextualError<E>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.iter()
    }
}

// Allow collecting Results into BizErrors
impl<E: BizError> FromIterator<ContextualError<E>> for BizErrors<E> {
    fn from_iter<T: IntoIterator<Item = ContextualError<E>>>(iter: T) -> Self {
        Self {
            errors: iter.into_iter().collect(),
        }
    }
}

impl<E: BizError> FromIterator<E> for BizErrors<E> {
    #[track_caller]
    fn from_iter<T: IntoIterator<Item = E>>(iter: T) -> Self {
        Self {
            errors: iter
                .into_iter()
                .map(|e| ContextualError::new(e, ""))
                .collect(),
        }
    }
}
