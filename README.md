# BizError - Structured Business Error Handling for Rust

A lightweight, flexible business error handling library that provides structured error codes and contextual information while maintaining full compatibility with Rust's error ecosystem.

## 🎯 Design Philosophy

**90/10 Principle**: 90% of error handling scenarios only need error codes, while 10% require detailed context information.

- **Minimal Core**: `BizError` trait contains only essential business error identification
- **Optional Context**: Use `ContextualError` wrapper only when detailed context is needed
- **Zero Overhead**: Basic usage scenarios have no additional performance cost
- **Full Compatibility**: Seamlessly integrates with thiserror and the entire Rust error ecosystem

## 🚀 Core Features

### Automatic Derive Macro

Use `#[derive(BizError)]` and `#[bizcode(code)]` attributes to automatically generate `BizError` trait implementations:

```rust
use bizerror::BizError;

#[derive(BizError, thiserror::Error)]
pub enum ApiError {
    #[bizcode(4001)]
    #[error("Invalid input: {field}")]
    ValidationError { field: String },

    #[bizcode(8001)]
    #[error("Database connection failed")]
    DatabaseError(#[from] std::io::Error),

    #[bizcode(8006)]
    #[error("Request timeout")]
    Timeout,
}

// Using the error
let error = ApiError::ValidationError { field: "email".to_string() };
assert_eq!(error.code(), 4001);                    // Error code
assert_eq!(error.name(), "ValidationError");       // Error name (auto-generated from variant name)
assert_eq!(error.msg(), "Invalid input: email");   // Error message (uses Display implementation)
```

### Automatic Debug Implementation

The derive macro also automatically generates structured Debug implementations:

```rust
let error = ApiError::ValidationError { field: "email".to_string() };
println!("{:?}", error);

// Output:
// ApiError { variant: "ValidationError", code: 4001, message: "Invalid input: email" }
```

### Core Components

1. **BizError Trait** - Business error identification
   - `code()` - Get business error code
   - `name()` - Get error type name
   - `msg()` - Get error message

2. **ContextualError<E>** - Optional context wrapper
   - Adds context information to any BizError
   - Automatic location tracking with `#[track_caller]`
   - Preserves original error chain

3. **Extension Traits** - Convenient helper methods
   - `ResultExt` - Adds `.with_biz_context()` to Results
   - `BizErrorExt` - Adds `.with_context()` to BizErrors

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
bizerror = "0.1.0"
thiserror = "1.0"
```

## 🔧 Usage

### Basic Usage (90% of scenarios)

```rust
use bizerror::BizError;

#[derive(BizError, thiserror::Error)]
pub enum ApiError {
    #[bizcode(4001)]
    #[error("Invalid input: {field}")]
    ValidationError { field: String },
    
    #[bizcode(8001)]
    #[error("Database connection failed")]
    DatabaseError(#[from] std::io::Error),
    
    #[bizcode(8006)]
    #[error("Request timeout")]
    Timeout,
}

// Using the error
let error = ApiError::ValidationError { field: "email".to_string() };
println!("Error code: {}", error.code());  // 4001
println!("Error name: {}", error.name());  // ValidationError
println!("Error message: {}", error.msg()); // Invalid input: email
```

### Contextual Usage (10% of scenarios)

```rust
use bizerror::*;

#[derive(BizError, thiserror::Error)]
pub enum ApiError {
    #[bizcode(8001)]
    #[error("Database connection failed")]
    DatabaseError(#[from] std::io::Error),
}

fn load_user_config() -> Result<String, ContextualError<ApiError>> {
    std::fs::read_to_string("config.json")
        .with_biz_context("Loading user configuration")
}

// Handle contextual errors
match load_user_config() {
    Ok(config) => println!("Configuration loaded successfully: {}", config),
    Err(e) => {
        println!("Error code: {}", e.code());
        println!("Context: {}", e.context());
        println!("Location: {}", e.location());
        println!("Original error: {}", e.inner());
    }
}
```

### Advanced Usage with Error Chains

```rust
fn complex_operation() -> Result<String, ContextualError<ApiError>> {
    // Step 1: Load configuration
    let _config = std::fs::read_to_string("app.json")
        .with_biz_context("Loading application configuration")?;
    
    // Step 2: Make API call
    let _response = make_api_call()
        .with_biz_context("Fetching user data from external service")?;
    
    Ok("Operation completed successfully".to_string())
}

// Error information flows through the chain
match complex_operation() {
    Ok(result) => println!("Success: {}", result),
    Err(e) => {
        println!("Business error code: {}", e.code());
        println!("Context: {}", e.context());
        println!("Location: {}", e.location());
        // Access the full error chain
        let mut source = std::error::Error::source(&e);
        while let Some(err) = source {
            println!("Caused by: {}", err);
            source = std::error::Error::source(err);
        }
    }
}
```

## 🎨 Auto-Generated Features

### Error Codes and Names

```rust
#[derive(BizError, thiserror::Error)]
pub enum HttpError {
    #[bizcode(8001)]
    #[error("Connection failed")]
    ConnectionFailed,
    
    #[bizcode(8002)]
    #[error("Request timeout")]
    Timeout,
}

let error = HttpError::ConnectionFailed;
assert_eq!(error.code(), 8001);                    // From bizcode attribute
assert_eq!(error.name(), "ConnectionFailed");      // From variant name
assert_eq!(error.msg(), "Connection failed");      // From Display implementation
```

### Structured Debug Output

```rust
let error = HttpError::ConnectionFailed;
println!("{:?}", error);

// Output:
// HttpError { variant: "ConnectionFailed", code: 8001, message: "Connection failed" }

// With source error:
let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
let error = HttpError::from(io_error);
println!("{:?}", error);

// Output:
// HttpError { variant: "ConnectionFailed", code: 8001, message: "Connection failed", source: ... }
```

### Support for All Variant Types

```rust
#[derive(BizError, thiserror::Error)]
pub enum AllVariantTypes {
    #[bizcode(1001)]
    #[error("Unit variant")]
    UnitVariant,
    
    #[bizcode(1002)]
    #[error("Tuple variant: {0}")]
    TupleVariant(String),
    
    #[bizcode(1003)]
    #[error("Struct variant: {field}")]
    StructVariant { field: String },
    
    #[bizcode(1004)]
    #[error("From conversion: {0}")]
    FromConversion(#[from] std::io::Error),
}
```

## 📊 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              BizError Ecosystem                                │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────────────┐  │
│  │  #[derive(      │    │ ContextualError  │    │    Extension Traits         │  │
│  │   BizError)]    │    │   (Optional)     │    │                             │  │
│  │                 │    │                  │    │  - ResultExt                │  │
│  │ Auto-generates: │────│ • Wraps BizError │    │  - BizErrorExt              │  │
│  │ • code()        │    │ • Adds context   │    │                             │  │
│  │ • name()        │    │ • Tracks location│    │  Provides convenient        │  │
│  │ • Debug impl    │    │                  │    │  .with_context() methods    │  │
│  └─────────────────┘    └──────────────────┘    └─────────────────────────────┘  │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                         Integrates with Standard Rust                          │
│                                                                                 │
│  ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────────────┐  │
│  │   std::error    │    │    thiserror     │    │      Your Error Types       │  │
│  │    ::Error      │    │                  │    │                             │  │
│  │                 │    │ • Display        │    │  Any error type that        │  │
│  │ Standard Error  │    │ • Debug          │    │  implements:                │  │
│  │ trait provides  │    │ • Error          │    │  • std::error::Error        │  │
│  │ foundation      │    │ • From/Into      │    │  • BizError (auto-derived)  │  │
│  │                 │    │                  │    │  • thiserror::Error         │  │
│  └─────────────────┘    └──────────────────┘    └─────────────────────────────┘  │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 🧪 Usage Patterns

### Pattern 1: Basic Error Handling (90% of cases)

```rust
// Define error type
#[derive(BizError, thiserror::Error)]
pub enum ServiceError {
    #[bizcode(2001)]
    #[error("User not found: {id}")]
    UserNotFound { id: u64 },
    
    #[bizcode(2002)]
    #[error("Network timeout")]
    NetworkTimeout,
}

// Use directly - zero overhead
fn find_user(id: u64) -> Result<User, ServiceError> {
    // ... implementation
    Err(ServiceError::UserNotFound { id })
}

// Access business information
match find_user(123) {
    Ok(user) => println!("Found user: {:?}", user),
    Err(e) => {
        // Log with business error code
        log::error!("Service error {}: {}", e.code(), e);
        
        // Return structured error response
        return json!({
            "error_code": e.code(),
            "error_name": e.name(),
            "message": e.to_string()
        });
    }
}
```

### Pattern 2: Contextual Error Handling (10% of cases)

```rust
// When you need detailed context and location tracking
fn process_user_request(user_id: u64) -> Result<Response, ContextualError<ServiceError>> {
    // Each step can add relevant context
    let user = find_user(user_id)
        .with_biz_context("Validating user access")?;
    
    let permissions = check_permissions(&user)
        .with_biz_context("Checking user permissions")?;
    
    let result = perform_operation(&user, &permissions)
        .with_biz_context("Executing user operation")?;
    
    Ok(result)
}

// Rich error information available
match process_user_request(123) {
    Ok(response) => send_response(response),
    Err(e) => {
        // Log with full context
        log::error!(
            "Request failed: {} (code: {}) at {} - Context: {}",
            e.inner(),
            e.code(),
            e.location(),
            e.context()
        );
        
        // Return detailed error for debugging
        return json!({
            "error_code": e.code(),
            "error_name": e.name(),
            "context": e.context(),
            "location": e.location().to_string(),
            "message": e.to_string()
        });
    }
}
```

## 📋 API Reference

### BizError Trait

```rust
pub trait BizError: Error + Send + Sync + 'static {
    /// Get the business error code
    fn code(&self) -> u16;
    
    /// Get the error type name
    fn name(&self) -> &str;
    
    /// Get the business error message (uses Display implementation)
    fn msg(&self) -> String {
        self.to_string()
    }
}
```

### ContextualError<E>

```rust
impl<E: BizError> ContextualError<E> {
    /// Create a new contextual error
    #[track_caller]
    pub fn new(error: E, context: impl Into<String>) -> Self;
    
    /// Get the original error
    pub fn inner(&self) -> &E;
    
    /// Get the context string
    pub fn context(&self) -> &str;
    
    /// Get the location where context was added
    pub fn location(&self) -> &'static Location<'static>;
}
```

### Extension Traits

```rust
// For Result<T, E> where E: Error
pub trait ResultExt<T, E> {
    /// Add business context to a Result
    fn with_biz_context<B>(self, context: impl Into<String>) -> Result<T, ContextualError<B>>
    where B: BizError + From<E>;
}

// For any BizError
pub trait BizErrorExt: BizError + Sized {
    /// Add context to a BizError
    #[track_caller]
    fn with_context(self, context: impl Into<String>) -> ContextualError<Self>;
}
```

## 🔄 Migration Guide

### From Plain std::error::Error

```rust
// Before
#[derive(Debug)]
struct MyError {
    message: String,
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MyError {}

// After
#[derive(BizError, thiserror::Error)]
#[error("{message}")]
struct MyError {
    message: String,
}

// BizError implementation is auto-generated by derive macro
```

### From thiserror

```rust
// Before
#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
}

// After - just add BizError derive and bizcode attributes
#[derive(BizError, thiserror::Error)]
pub enum MyError {
    #[bizcode(3001)]
    #[error("Network error: {0}")]
    Network(String),
    
    #[bizcode(3002)]
    #[error("Parse error: {0}")]
    Parse(String),
}

// Both BizError implementation and Debug implementation are auto-generated
```

## 🎛️ Configuration & Best Practices

### Error Code Organization

```rust
// Organize error codes by domain
pub mod error_codes {
    // Authentication: 1000-1999
    pub const AUTH_INVALID_TOKEN: u16 = 1001;
    pub const AUTH_EXPIRED_TOKEN: u16 = 1002;
    pub const AUTH_INSUFFICIENT_PERMISSIONS: u16 = 1003;
    
    // Validation: 2000-2999
    pub const VALIDATION_REQUIRED_FIELD: u16 = 2001;
    pub const VALIDATION_INVALID_FORMAT: u16 = 2002;
    pub const VALIDATION_OUT_OF_RANGE: u16 = 2003;
    
    // External Services: 3000-3999
    pub const EXTERNAL_SERVICE_UNAVAILABLE: u16 = 3001;
    pub const EXTERNAL_SERVICE_TIMEOUT: u16 = 3002;
    pub const EXTERNAL_SERVICE_INVALID_RESPONSE: u16 = 3003;
}

#[derive(BizError, thiserror::Error)]
pub enum AuthError {
    #[bizcode(error_codes::AUTH_INVALID_TOKEN)]
    #[error("Invalid authentication token")]
    InvalidToken,
    
    #[bizcode(error_codes::AUTH_EXPIRED_TOKEN)]
    #[error("Authentication token has expired")]
    ExpiredToken,
}
```

### Context Guidelines

```rust
// Good: Specific, actionable context
.with_biz_context("Loading user preferences for dashboard")

// Good: Business operation context
.with_biz_context("Processing payment transaction")

// Avoid: Generic context
.with_biz_context("An error occurred")

// Avoid: Too verbose
.with_biz_context("Loading user preferences from database table user_preferences for user ID 123 in the dashboard initialization process")
```

### Performance Considerations

```rust
// Zero overhead - use for most cases
fn fast_operation() -> Result<String, MyError> {
    // Just return business errors directly
    Err(MyError::SomeError)
}

// Small overhead - use when context is valuable
fn complex_operation() -> Result<String, ContextualError<MyError>> {
    // Add context for better debugging
    fast_operation()
        .with_biz_context("During complex operation")
}
```

## 🧪 Testing

The library includes comprehensive tests covering:

- Basic BizError trait functionality
- ContextualError wrapper behavior
- Extension trait methods
- Error chain preservation
- Location tracking accuracy
- Display and Debug output formatting

Run tests:

```bash
cargo test
```

## 📈 Performance

- **Basic usage**: Zero overhead compared to standard Rust errors
- **Contextual usage**: Minimal overhead from storing context string and location
- **Memory efficient**: No heap allocations for basic error codes
- **Compile-time optimized**: Monomorphization ensures optimal performance

## 🤝 Contributing

Contributions are welcome! Please feel free to:

1. Report bugs and request features via GitHub issues
2. Submit pull requests with improvements
3. Improve documentation and examples
4. Share your use cases and feedback

## 📄 License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🔗 Related Projects

- [`thiserror`](https://github.com/dtolnay/thiserror) - Derive macros for Error trait
- [`anyhow`](https://github.com/dtolnay/anyhow) - Flexible error handling
- [`snafu`](https://github.com/shepmaster/snafu) - Structured error handling

---

**BizError** provides the missing piece for business error handling in Rust - structured error codes with minimal overhead and maximum flexibility.