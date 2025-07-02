//! # bizerror - Structured Business Error Handling
//!
//! `bizerror` provides business error codes on top of `thiserror`, combining:
//! - **ThisError**: Handles all standard Error trait implementations (Display, Debug, Error, From)
//! - **BizError**: Adds business error code methods
//!
//! ## Usage
//!
//! ### Ideal Experience (Zero Configuration)
//!
//! Simply use the `#[bizerror]` attribute:
//!
//! ```rust
//! use bizerror::bizerror;
//!
//! #[bizerror]
//! pub enum ApiError {
//!     #[bizerror(4001, "Invalid input: {field}")]
//!     ValidationError { field: String },
//!
//!     #[bizerror(8001, "Database connection failed")]
//!     DatabaseError(#[from] std::io::Error),
//! }
//!
//! // Access business error codes
//! let error = ApiError::ValidationError { field: "email".to_string() };
//! assert_eq!(error.code(), 4001);
//! 
//! // ThisError handles Display, Debug, Error traits automatically
//! println!("{}", error); // "Invalid input: email"
//! println!("{:?}", error); // Full debug info
//! ```
//!
//! ### Alternative: Derive Macro (Manual Setup)
//!
//! For more control, use the derive macro with manual setup:
//!
//! ```rust,ignore
//! use bizerror::BizError;
//!
//! #[derive(BizError, thiserror::Error, Debug)]
//! pub enum ApiError {
//!     #[bizerror(4001, "Invalid input: {field}")]
//!     #[error("Invalid input: {field}")]
//!     ValidationError { field: String },
//! }
//! ```
//!
//! ## Design Philosophy
//!
//! - **ThisError**: Mature, battle-tested Error trait implementations
//! - **BizError**: Focused on business error codes
//! - **No duplication**: Don't reinvent what ThisError already does well
//!
//! ## Error Code Categories
//!
//! - `1xxx`: Configuration errors
//! - `2xxx`: Authentication errors  
//! - `3xxx`: Authorization errors
//! - `4xxx`: Validation errors
//! - `5xxx`: Business logic errors
//! - `6xxx`: Rate limiting errors
//! - `7xxx`: External service errors
//! - `8xxx`: System errors
//! - `9xxx`: Unknown errors
//!

// Re-export the macros
pub use bizerror_impl::{BizError, bizerror};

// Re-export thiserror and common traits that users might need
pub use thiserror;
pub use std::error::Error;

