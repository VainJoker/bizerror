# BizError Usage Guide

This guide provides detailed instructions on how to use the `bizerror` library for business error handling.

## Table of Contents

- [Core Concepts](#core-concepts)
- [Quick Start](#quick-start)
- [Configuration Options](#configuration-options)
- [Error Types](#error-types)
- [Context Handling](#context-handling)
- [Best Practices](#best-practices)
- [Advanced Usage](#advanced-usage)
- [FAQ](#faq)

## Core Concepts

### The 90/10 Principle

`bizerror` is designed around the 90/10 principle:
- **90%** of error handling scenarios only need error codes
- **10%** of scenarios need detailed contextual information

### Core Components

1. **`BizError` trait**: Defines the basic interface for business errors
2. **`ContextualError`**: Provides wrapper for errors that need context information
3. **Derive macro**: Automatically generates `BizError` implementations
4. **Extension traits**: Provide convenient methods for error handling

## Quick Start

### 1. Add Dependencies

```toml
[dependencies]
bizerror = "0.1.0"
thiserror = "1.0"
```

### 2. Define Error Types

```rust
use bizerror::BizError;
use thiserror::Error;

#[derive(BizError, Error)]
pub enum UserError {
    #[bizcode(1001)]
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: u64 },

    #[bizcode(1002)]
    #[error("User already exists: {username}")]
    UserExists { username: String },

    #[bizcode(1003)]
    #[error("Password too weak")]
    WeakPassword,
}
```

### 3. Use Errors

```rust
fn find_user(user_id: u64) -> Result<User, UserError> {
    // Simulate user lookup
    if user_id == 0 {
        return Err(UserError::UserNotFound { user_id });
    }
    
    // Return user
    Ok(User { id: user_id, name: "Alice".to_string() })
}

// Use error information
match find_user(0) {
    Ok(user) => println!("Found user: {}", user.name),
    Err(e) => {
        println!("Error code: {}", e.code());
        println!("Error name: {}", e.name());
        println!("Error message: {}", e);
    }
}
```

## Configuration Options

### Auto Code Assignment

```rust
#[derive(BizError, Error)]
#[bizconfig(auto_start = 1000, auto_increment = 10)]
pub enum ServiceError {
    #[error("First error")]
    FirstError,    // Code: 1000

    #[bizcode(2001)]
    #[error("Explicit error")]
    ExplicitError, // Code: 2001

    #[error("Second error")]
    SecondError,   // Code: 1010
}
```

### Custom Error Code Types

```rust
// String error codes
#[derive(BizError, Error)]
#[bizconfig(code_type = "&'static str")]
pub enum StringError {
    #[bizcode("USER_NOT_FOUND")]
    #[error("User not found")]
    UserNotFound,

    #[error("Auto string code")]
    AutoString, // Code: "0"
}

// Signed integer error codes
#[derive(BizError, Error)]
#[bizconfig(code_type = "i32", auto_start = -100, auto_increment = -1)]
pub enum SignedError {
    #[error("Negative error code")]
    NegativeCode, // Code: -100
}
```

## Error Types

### Unit Variants

```rust
#[derive(BizError, Error)]
pub enum SimpleError {
    #[bizcode(1001)]
    #[error("Simple error")]
    SimpleError,
}
```

### Struct Variants

```rust
#[derive(BizError, Error)]
pub enum StructError {
    #[bizcode(2001)]
    #[error("Validation failed: {field}")]
    ValidationError { field: String },
}
```

### Tuple Variants

```rust
#[derive(BizError, Error)]
pub enum TupleError {
    #[bizcode(3001)]
    #[error("Network error: {0}")]
    NetworkError(String),
}
```

### Variants with Source Errors

```rust
#[derive(BizError, Error)]
pub enum ChainError {
    #[bizcode(4001)]
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## Context Handling

### Basic Context

```rust
use bizerror::*;

fn load_config() -> Result<String, ContextualError<ConfigError>> {
    std::fs::read_to_string("config.json")
        .with_context("Loading application configuration")
}
```

### Multi-Layer Context

```rust
fn complex_operation() -> Result<String, ContextualError<ServiceError>> {
    // First layer operation
    let config = load_config()
        .with_context("Preparing configuration")?;
    
    // Second layer operation
    let data = process_data(&config)
        .with_context("Processing data")?;
    
    Ok(data)
}
```

### Error Chain Handling

```rust
match complex_operation() {
    Ok(result) => println!("Success: {}", result),
    Err(e) => {
        println!("Error code: {}", e.code());
        println!("Context: {}", e.context());
        println!("Location: {}:{}", e.location().file(), e.location().line());
        
        // Print error chain
        let mut source = std::error::Error::source(&e);
        while let Some(err) = source {
            println!("Caused by: {}", err);
            source = std::error::Error::source(err);
        }
    }
}
```

## Best Practices

### 1. Error Code Planning

Recommend allocating error code ranges by functional modules:

```rust
// User service: 1000-1999
#[derive(BizError, Error)]
#[bizconfig(auto_start = 1000, auto_increment = 1)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,
    
    #[error("User already exists")]
    UserExists,
}

// Payment service: 2000-2999
#[derive(BizError, Error)]
#[bizconfig(auto_start = 2000, auto_increment = 1)]
pub enum PaymentError {
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    #[error("Payment failed")]
    PaymentFailed,
}

// System errors: 8000-8999
#[derive(BizError, Error)]
#[bizconfig(auto_start = 8000, auto_increment = 1)]
pub enum SystemError {
    #[error("Database connection failed")]
    DatabaseError,
    
    #[error("Network timeout")]
    NetworkTimeout,
}
```

### 2. Error Conversion

Use `#[from]` attribute for automatic error conversion:

```rust
#[derive(BizError, Error)]
pub enum AppError {
    #[bizcode(9001)]
    #[error("User error: {0}")]
    UserError(#[from] UserError),
    
    #[bizcode(9002)]
    #[error("Payment error: {0}")]
    PaymentError(#[from] PaymentError),
    
    #[bizcode(9003)]
    #[error("System error: {0}")]
    SystemError(#[from] SystemError),
}
```

### 3. Context Usage Principles

- **90% scenarios**: Use only basic error codes
- **10% scenarios**: Add context information for debugging

```rust
// Basic scenario - only return error code
fn validate_user(user: &User) -> Result<(), UserError> {
    if user.name.is_empty() {
        return Err(UserError::InvalidName);
    }
    Ok(())
}

// Complex scenario - add context
fn process_user_registration(user: User) -> Result<u64, ContextualError<AppError>> {
    validate_user(&user)
        .with_context("Validating user information")?;
    
    save_user(&user)
        .with_context("Saving user to database")?;
    
    Ok(user.id)
}
```

### 4. Document Error Codes

```rust
/// User service errors
/// 
/// Error code range: 1000-1999
/// 
/// | Code | Name | Description |
/// |------|------|-------------|
/// | 1000 | UserNotFound | User not found |
/// | 1001 | UserExists | User already exists |
/// | 1002 | InvalidEmail | Invalid email format |
#[derive(BizError, Error)]
#[bizconfig(auto_start = 1000, auto_increment = 1)]
pub enum UserError {
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: u64 },
    
    #[error("User already exists: {username}")]
    UserExists { username: String },
    
    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },
}
```

## Advanced Usage

### Custom BizError Implementation

```rust
use bizerror::BizError;
use std::error::Error;

#[derive(Debug)]
pub struct CustomError {
    code: u32,
    message: String,
    details: Vec<String>,
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for CustomError {}

impl BizError for CustomError {
    type CodeType = u32;
    
    fn code(&self) -> Self::CodeType {
        self.code
    }
    
    fn name(&self) -> &str {
        "CustomError"
    }
}
```

### Error Aggregation

```rust
use bizerror::BizErrors;

fn validate_multiple_fields(data: &UserData) -> Result<(), BizErrors<ValidationError>> {
    let mut errors = BizErrors::new();
    
    if data.email.is_empty() {
        errors.push_simple(ValidationError::EmptyEmail);
    }
    
    if data.name.is_empty() {
        errors.push_simple(ValidationError::EmptyName);
    }
    
    if !errors.is_empty() {
        return Err(errors);
    }
    
    Ok(())
}
```

### Option to Result Conversion

```rust
use bizerror::*;

#[derive(BizError, Error)]
pub enum MyError {
    #[bizcode(4001)]
    #[error("Value not found")]
    NotFound,
}

let value: Option<String> = None;
let result = value.ok_or_biz(MyError::NotFound);
assert!(result.is_err());
```

### Conditional Error Handling

```rust
use bizerror::*;

fn conditional_error_handling(
    condition: bool
) -> Result<String, ContextualError<AppError>> {
    if condition {
        // Simple error handling
        simple_operation()
            .map_biz()
    } else {
        // Complex error handling (with context)
        complex_operation()
            .with_context("Executing complex operation")
    }
}
```

## FAQ

### Q: How to choose error code type?

A: Choose based on project requirements:
- **Numeric codes** (`u32`, `u16`): Suitable for most scenarios, memory efficient
- **String codes** (`&'static str`): Suitable for scenarios requiring readability
- **Signed codes** (`i32`): Suitable when negative numbers are needed for special cases

### Q: When to use ContextualError?

A: Use only when detailed debugging information is needed:
- Complex business logic
- Multi-layer call chains
- Scenarios requiring precise problem location

### Q: How to handle error code conflicts?

A: Use `bizconfig` to configure different starting codes:

```rust
#[derive(BizError, Error)]
#[bizconfig(auto_start = 1000, auto_increment = 1)]
pub enum ServiceAError { /* ... */ }

#[derive(BizError, Error)]
#[bizconfig(auto_start = 2000, auto_increment = 1)]
pub enum ServiceBError { /* ... */ }
```

### Q: Performance considerations?

A: `bizerror` is designed for zero overhead:
- Basic error handling has no extra performance cost
- Only slight overhead when using `ContextualError`
- Error code retrieval is constant time operation

### Q: Relationship with thiserror?

A: `bizerror` complements `thiserror`:
- `thiserror` provides error definition and display
- `bizerror` provides business error codes and context
- Both work perfectly together

## Example Projects

Check complete examples in the `examples/` directory:
- `simple.rs`: Basic usage examples
- `new_design_demo.rs`: New feature demonstrations
- `real_world_usage.rs`: Real-world usage scenarios

## References

- [API Documentation](https://docs.rs/bizerror)
- [GitHub Repository](https://github.com/vainjoker/bizerror)
- [Changelog](../CHANGELOG.md) 