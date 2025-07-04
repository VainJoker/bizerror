# Bizerror Extended Features Guide

This document details the extended features in the bizerror library that make error handling more convenient and powerful.

## 📋 Feature Overview

### 🔄 Core Transformation Methods
- [`with_context`](#with_context) - Add context with automatic location tracking
- [`map_biz`](#map_biz) - Simple error type conversion

### 🛡️ Option Handling
- [`ok_or_biz`](#ok_or_biz) - Convert Option to Result with business errors

### 🎯 Conditional Processing
- [`with_context_if`](#with_context_if) - Conditionally add context

### 🔗 Chain Operations
- [`and_then_biz`](#and_then_biz) - Chain operations with error conversion

### 📊 Error Aggregation
- [`BizErrors`](#bizerrors) - Collect and manage multiple errors

---

## 🔧 Detailed Feature Descriptions

### <a name="with_context"></a>1. `with_context` - Add Context with Location Tracking

**Most commonly used method** that adds contextual information to errors with automatic location tracking.

```rust
use bizerror::*;

#[derive(BizError, thiserror::Error)]
pub enum AppError {
    #[bizcode(8001)]
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// Add context to any error that can be converted to your business error type
let result = std::fs::read_to_string("config.txt")
    .with_context("Loading configuration file");
```

**Use cases:**
- Most common error handling pattern
- When you need to add contextual information for debugging
- Provides automatic location tracking (file, line, column)

### <a name="map_biz"></a>2. `map_biz` - Simple Error Type Conversion

Convert error types without adding context information.

```rust
// Convert error type without context
let result = operation().map_biz::<AppError>();

// Equivalent to:
let result = operation().map_err(AppError::from);
```

**Use cases:**
- When you only need error type conversion without context
- Keeping error chains simple and clean
- Better performance when context is not needed

### <a name="ok_or_biz"></a>3. `ok_or_biz` - Convert Option to Result

Convert `Option<T>` to `Result<T, BusinessError>` with business errors.

```rust
use bizerror::*;

#[derive(BizError, thiserror::Error)]
pub enum MyError {
    #[bizcode(4001)]
    #[error("User not found")]
    UserNotFound,
}

let user_id: Option<u64> = None;
let result = user_id.ok_or_biz(MyError::UserNotFound);
assert!(result.is_err());
```

**Use cases:**
- Converting optional values to business errors
- Handling missing data scenarios
- API responses where absence indicates specific business error

### <a name="with_context_if"></a>4. `with_context_if` - Conditional Context Addition

Add context only when a condition is met.

```rust
let result = operation()
    .with_context_if::<AppError>(
        debug_mode,
        format!("Debug: operation with params: {:?}", params),
    );
```

**Use cases:**
- Debug mode with detailed information
- Adding different levels of error information based on user permissions
- Adjusting error detail based on environment (development/production)

### <a name="and_then_biz"></a>5. `and_then_biz` - Chain Operations with Error Conversion

Chain operations while handling error type conversions.

```rust
let result = parse_config()
    .and_then_biz::<ProcessedConfig, _, AppError>(|config| {
        validate_config(config)
    })
    .and_then(|config| {
        apply_config(config)
    });
```

**Use cases:**
- Multi-step processing workflows
- Chained operations that need error type conversion
- Functional programming style error handling

### <a name="bizerrors"></a>6. `BizErrors` - Error Aggregation

Collect and manage multiple errors instead of failing on the first error.

```rust
use bizerror::*;

#[derive(BizError, thiserror::Error)]
pub enum ValidationError {
    #[bizcode(4001)]
    #[error("Invalid email")]
    InvalidEmail,
    
    #[bizcode(4002)]
    #[error("Invalid name")]
    InvalidName,
}

fn validate_user(user: &User) -> Result<(), BizErrors<ValidationError>> {
    let mut errors = BizErrors::new();
    
    if user.email.is_empty() {
        errors.push_simple(ValidationError::InvalidEmail);
    }
    
    if user.name.is_empty() {
        errors.push_simple(ValidationError::InvalidName);
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

**Core functionality:**
- `new()` - Create empty error collection
- `push()` - Add contextual error
- `push_simple()` - Add simple error
- `push_with_context()` - Add error with context

**Advanced features:**
- `collect_from()` - Separate successes and errors from iterator
- `collect_errors()` - Collect only errors from iterator
- `contains_code()` - Check for specific error codes
- `error_codes()` - Get unique error codes
- `filter()` - Filter errors by predicate
- Full iterator support

**Use cases:**
- Form validation (collect all validation errors)
- Batch processing (continue processing despite individual failures)
- Data validation (report all issues at once)

---

## 🔄 Combined Usage Examples

### Complete Error Handling Pipeline

```rust
use bizerror::*;

#[derive(BizError, thiserror::Error)]
pub enum AppError {
    #[bizcode(8001)]
    #[error("Configuration error: {0}")]
    ConfigError(#[from] std::io::Error),
    
    #[bizcode(8002)]
    #[error("Validation error: {msg}")]
    ValidationError { msg: String },
}

fn load_and_process_config(debug: bool) -> Result<String, ContextualError<AppError>> {
    std::fs::read_to_string("app.config")
        .with_context("Loading configuration file")
        .or_else(|e| {
            if debug {
                eprintln!("Config load failed: {}", e);
            }
            // Fallback to default config
            Ok("default_config".to_string())
        })
        .and_then_biz::<String, _, AppError>(|config| {
            if config.is_empty() {
                Err(AppError::ValidationError {
                    msg: "Config cannot be empty".to_string(),
                })
            } else {
                Ok(config.to_uppercase())
            }
        })
        .with_context_if::<AppError>(
            debug,
            "Processing configuration with debug enabled",
        )
}
```

### Batch Processing with Error Collection

```rust
fn process_users(users: Vec<User>) -> Result<Vec<ProcessedUser>, BizErrors<ValidationError>> {
    let mut errors = BizErrors::new();
    let mut processed = Vec::new();
    
    for user in users {
        match validate_and_process_user(&user) {
            Ok(processed_user) => processed.push(processed_user),
            Err(e) => errors.push_with_context(e, format!("Processing user {}", user.id)),
        }
    }
    
    if errors.is_empty() {
        Ok(processed)
    } else {
        Err(errors)
    }
}
```

### Error Analysis and Reporting

```rust
fn analyze_processing_errors(errors: &BizErrors<ValidationError>) {
    println!("Total errors: {}", errors.len());
    
    // Get all unique error codes
    let codes = errors.error_codes();
    println!("Error codes found: {:?}", codes);
    
    // Check for specific error types
    if errors.contains_code(4001) {
        println!("Email validation errors found");
    }
    
    // Filter specific error types
    let email_errors: Vec<_> = errors
        .filter(|e| e.code() == 4001)
        .collect();
    
    println!("Email errors: {}", email_errors.len());
}
```

---

## 🎯 Best Practices

### 1. Choose the Right Method

- **High frequency use**: `with_context` for most error handling scenarios
- **Simple conversion**: Use `map_biz` when context is not needed
- **Optional values**: Use `ok_or_biz` for converting None to business errors
- **Conditional handling**: Use `with_context_if` for debug/production differences

### 2. Error Chain Construction

```rust
// Good practice: Clear error chain
operation()
    .map_biz::<AppError>()
    .with_context("Processing user data")
    .and_then(|data| process_data(data))
    .or_else(|e| {
        log::error!("Processing failed: {}", e);
        provide_fallback()
    })
```

### 3. Performance Considerations

- `with_context` has minimal overhead on success path
- `BizErrors` collection is more efficient than individual error handling for batch operations
- `map_biz` is zero-cost when no conversion is needed
- Avoid excessive use of `with_context_if` due to condition evaluation overhead

### 4. Readability Optimization

```rust
// Chain calls with line breaks for readability
result
    .with_context("Step 1: Data preparation")
    .and_then_biz(|data| process_step_2(data))
    .with_context("Step 2: Processing completed")
```

---

## 📈 Performance Impact

All extended methods follow the zero-overhead principle:
- Success path has minimal additional overhead
- Error path overhead mainly comes from context string allocation
- Error aggregation is optimized for batch scenarios
- Closure calls are optimized by the compiler and usually inlined

## 🔧 Type Inference

Most cases can rely on type inference:

```rust
// Type can usually be inferred
let result = operation().with_context("context");

// Explicit type may be needed in complex cases
let result = operation().with_context::<MyError>("context");
```

---

## 🔍 When to Use Each Feature

| Feature | Use Case | Example Scenario |
|---------|----------|------------------|
| `with_context` | Most common error handling | File operations, API calls |
| `map_biz` | Simple type conversion | Converting library errors |
| `ok_or_biz` | Optional value handling | Database query results |
| `with_context_if` | Conditional context | Debug vs production mode |
| `and_then_biz` | Chain processing | Multi-step workflows |
| `BizErrors` | Multiple error collection | Form validation, batch processing |

---

These extended features make bizerror more than just an error definition library - they provide a complete toolkit for robust error handling in Rust applications, making error management both simple and powerful. 