# BizError: Effortless, Structured Business Errors for Rust

[![Crates.io](https://img.shields.io/crates/v/bizerror.svg)](https://crates.io/crates/bizerror)
[![Docs.rs](https://docs.rs/bizerror/badge.svg)](https://docs.rs/bizerror)
[![Build Status](https://github.com/vainjoker/bizerror/actions/workflows/integration.yml/badge.svg)](https://github.com/vainjoker/bizerror/actions)

`BizError` is a lightweight, ergonomic library for creating structured, business-oriented errors in Rust. It simplifies error handling by providing a clear distinction between technical errors and business-logic failures, all while integrating seamlessly with the standard `std::error::Error` trait and popular libraries like `thiserror`.

## 🎯 Core Philosophy: The 90/10 Rule

`BizError` is built on a simple observation:
- **90% of the time**, you just need a structured **error code** to identify a business failure.
- **10% of the time**, you need rich **contextual information** for debugging.

This library is optimized for both scenarios, providing a zero-overhead solution for the common case and powerful context-aware tools for the complex cases.

## ✨ Features

- **Derive Macro for Enums**: Automatically implement `BizError` and `Debug` for your error enums.
- **Structured Error Info**: Get a unique `code()` and `name()` for every error variant.
- **Automatic Code Assignment**: Let the macro assign error codes for you, or specify them explicitly.
- **Customizable Code Types**: Use numeric types (`u32`, `i16`, etc.) or string types (`&'static str`) for your codes.
- **Rich Contextual Errors**: Wrap any error to add context, file locations, and backtraces—only when you need it.
- **Error Aggregation**: Collect multiple errors with `BizErrors` for scenarios like validation.
- **Ecosystem Friendly**: Works perfectly with `thiserror` and the standard library.

## 📦 Installation

Add `bizerror` and `thiserror` to your `Cargo.toml`:

```toml
[dependencies]
bizerror = "0.1.1"
thiserror = "1.0" # Recommended for Display trait implementation
```

## 🚀 Quick Start: Define Your Errors

Defining business errors is incredibly simple. Just derive `BizError` and `thiserror::Error`, then annotate your variants.

```rust
use bizerror::BizError;
use thiserror::Error;

#[derive(Debug, BizError, Error)]
#[bizconfig(code_type = "u16")] // Optional: configure the code type
pub enum ApiError {
    #[bizcode(4001)]
    #[error("Invalid input for field: {field}")]
    ValidationError { field: String },

    #[bizcode(4004)]
    #[error("Resource not found with ID: {id}")]
    NotFound { id: u64 },

    #[bizcode(5000)]
    #[error("Internal server error")]
    ServerError,
}

// --- Using your error ---

let error = ApiError::NotFound { id: 123 };

// Get structured information
assert_eq!(error.code(), 4004);
assert_eq!(error.name(), "NotFound");
assert_eq!(error.to_string(), "Resource not found with ID: 123");

// The generated Debug trait provides structured logging
println!("{:?}", error);
// Output: ApiError { variant: "NotFound", code: 4004, message: "Resource not found with ID: 123" }
```

## ⚙️ Advanced Usage

### Automatic Code Assignment

Let `bizerror` handle the codes for you. Use `#[bizconfig]` to set a starting point and increment step. You can still override any code with `#[bizcode]`.

```rust
use bizerror::BizError;
use thiserror::Error;

#[derive(Debug, BizError, Error)]
#[bizconfig(auto_start = 1000, auto_increment = 10)]
pub enum ServiceError {
    #[error("Authentication failed")]
    AuthError, // Auto-assigned code: 1000

    #[bizcode(2001)] // Explicit code takes precedence
    #[error("Business rule violation")]
    BusinessRuleError,

    #[error("Database connection failed")]
    DatabaseError, // Auto-assigned code: 1010
}

assert_eq!(ServiceError::AuthError.code(), 1000);
assert_eq!(ServiceError::BusinessRuleError.code(), 2001);
assert_eq!(ServiceError::DatabaseError.code(), 1010);
```

### String-Based Error Codes

Error codes aren't limited to numbers. Use strings for more descriptive identifiers.

```rust
use bizerror::BizError;
use thiserror::Error;

#[derive(Debug, BizError, Error)]
#[bizconfig(code_type = "&'static str")]
pub enum PaymentError {
    #[bizcode("INSUFFICIENT_FUNDS")]
    #[error("The account has insufficient funds")]
    InsufficientFunds,

    #[bizcode("CARD_EXPIRED")]
    #[error("The credit card has expired")]
    CardExpired,
}

assert_eq!(PaymentError::InsufficientFunds.code(), "INSUFFICIENT_FUNDS");
```

### Adding Context to Errors

For the 10% of cases where you need more insight, wrap your errors in a `ContextualError`. This adds a descriptive message and captures the exact location where the error was handled.

```rust
use bizerror::{BizError, BizErrorExt, ContextualError};
use thiserror::Error;
use std::fs;

#[derive(Debug, BizError, Error)]
#[bizconfig(code_type = "u16")]
pub enum ConfigError {
    #[bizcode(101)]
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
}

fn read_config() -> Result<String, ContextualError<ConfigError>> {
    fs::read_to_string("app.config")
        // with_context is available on any Result<T, E> where E: Into<BizError>
        .with_context("Attempting to load application configuration")
}

match read_config() {
    Ok(config) => println!("Config loaded!"),
    Err(e) => {
        println!("Error Code: {}", e.code());
        println!("Context: {}", e.context());
        println!("Location: {}", e.location());
        println!("Root Cause: {}", e.inner());
    }
}
```

### Aggregating Multiple Errors

Instead of failing on the first error, use `BizErrors` to collect them all. This is perfect for validating user input or processing a batch of items.

```rust
use bizerror::{BizError, BizErrors};
use thiserror::Error;

#[derive(Debug, Clone, BizError, Error, PartialEq, Eq)]
#[bizconfig(code_type = "u16")]
pub enum ValidationError {
    #[bizcode(1)]
    #[error("Username must be at least 3 characters long")]
    UsernameTooShort,

    #[bizcode(2)]
    #[error("Password must contain a special character")]
    PasswordRequiresSpecialChar,
}

fn validate_signup(username: &str, password: &str) -> Result<(), BizErrors<ValidationError>> {
    let mut errors = BizErrors::new();

    if username.len() < 3 {
        errors.push_simple(ValidationError::UsernameTooShort);
    }

    if !password.chars().any(|c| "!@#$%^&*()".contains(c)) {
        errors.push_simple(ValidationError::PasswordRequiresSpecialChar);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

let result = validate_signup("me", "password");
assert!(result.is_err());

let collected_errors = result.unwrap_err();
assert_eq!(collected_errors.len(), 2);
assert_eq!(
    collected_errors.error_codes(),
    vec![
        ValidationError::UsernameTooShort.code(),
        ValidationError::PasswordRequiresSpecialChar.code()
    ]
);

println!("Validation failed with {} errors:
{}", collected_errors.len(), collected_errors);
```

## 📊 Implementation Roadmap

### Phase 1: Core Enhancements 
- ✅ Robust attribute parsing with `syn`
- ✅ Flexible code type support
- 🔄 Serde integration
- 🔄 Code validation enhancements

### Phase 2: Developer Experience 
- 📋 Tracing integration
- 📋 Test utilities and assertion macros
- 📋 Documentation generation tools

### Phase 3: Advanced Features 
- 📋 CLI tooling for error management
- 📋 Internationalization support
- 📋 Error analytics and monitoring

### Phase 4: Ecosystem Maturity
- 📋 No-std support for embedded systems
- 📋 Advanced error handling patterns
- 📋 Performance optimizations

## 🤝 Contributing to Development

We welcome contributions! Whether you're interested in:
- Implementing new features from our TODO list
- Improving documentation and examples
- Adding test coverage
- Reporting bugs or suggesting enhancements

Check out our [TODO.md](TODO.md) for detailed implementation plans and effort estimates.

## 📈 Success Metrics

We track our progress through:
- **Adoption**: Downloads on crates.io and GitHub stars
- **Integration**: Usage in popular Rust projects and frameworks
- **Performance**: Benchmarks against standard error handling
- **Community**: User feedback and contribution engagement

---

*Note: This roadmap is subject to change based on community feedback and real-world usage patterns. Priority may be adjusted based on user demand and contribution availability.*



## 📄 License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.