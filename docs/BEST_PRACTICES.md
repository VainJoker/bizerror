# BizError Best Practices Guide

This document provides best practice recommendations for using the `bizerror` library to build robust and maintainable error handling systems.

## Table of Contents

- [Error Code Design](#error-code-design)
- [Error Type Organization](#error-type-organization)
- [Context Usage Strategy](#context-usage-strategy)
- [Performance Optimization](#performance-optimization)
- [Testing Strategy](#testing-strategy)
- [Documentation Guidelines](#documentation-guidelines)
- [Version Management](#version-management)

## Error Code Design

### 1. Error Code Range Planning

Allocate error code ranges by business module or service:

```rust
// User management module: 1000-1999
#[derive(BizError, Error)]
#[bizconfig(auto_start = 1000, auto_increment = 1)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,
    
    #[error("User already exists")]
    UserExists,
    
    #[error("Permission denied")]
    PermissionDenied,
}

// Order management module: 2000-2999
#[derive(BizError, Error)]
#[bizconfig(auto_start = 2000, auto_increment = 1)]
pub enum OrderError {
    #[error("Order not found")]
    OrderNotFound,
    
    #[error("Order cancelled")]
    OrderCanceled,
}

// Payment module: 3000-3999
#[derive(BizError, Error)]
#[bizconfig(auto_start = 3000, auto_increment = 1)]
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
    #[error("Database error")]
    DatabaseError,
    
    #[error("Network timeout")]
    NetworkTimeout,
    
    #[error("Configuration error")]
    ConfigError,
}
```

### 2. Error Code Naming Conventions

Use clear, consistent naming conventions:

```rust
#[derive(BizError, Error)]
pub enum ValidationError {
    // ✅ Good naming - clearly describes error type
    #[bizcode(4001)]
    #[error("Invalid email format: {email}")]
    InvalidEmailFormat { email: String },
    
    #[bizcode(4002)]
    #[error("Password too weak")]
    PasswordTooWeak,
    
    #[bizcode(4003)]
    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },
    
    // ❌ Avoid these naming patterns - unclear and vague
    // Error1,
    // BadInput,
    // Fail,
}
```

### 3. Error Code Reservation Strategy

Reserve error code space for future expansion:

```rust
#[derive(BizError, Error)]
#[bizconfig(auto_start = 1000, auto_increment = 10)]
pub enum UserError {
    #[error("User not found")]
    UserNotFound,      // 1000
    
    #[error("User already exists")]
    UserExists,        // 1010
    
    // Reserve 1001-1009, 1011-1019 etc. for future expansion
}
```

## Error Type Organization

### 1. Layered Error Structure

Establish a clear error hierarchy:

```rust
// Domain layer errors
#[derive(BizError, Error)]
#[bizconfig(auto_start = 1000, auto_increment = 1)]
pub enum DomainError {
    #[error("Business rule violation")]
    BusinessRuleViolation,
    
    #[error("Data validation failed")]
    ValidationFailed,
}

// Application layer errors
#[derive(BizError, Error)]
#[bizconfig(auto_start = 2000, auto_increment = 1)]
pub enum ApplicationError {
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),
    
    #[error("User error: {0}")]
    UserError(#[from] UserError),
    
    #[error("Permission error: {0}")]
    PermissionError(#[from] PermissionError),
}

// Infrastructure layer errors
#[derive(BizError, Error)]
#[bizconfig(auto_start = 8000, auto_increment = 1)]
pub enum InfrastructureError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] NetworkError),
    
    #[error("External service error: {0}")]
    ExternalServiceError(#[from] ExternalServiceError),
}
```

### 2. Error Aggregation Pattern

Create unified error types at application boundaries:

```rust
#[derive(BizError, Error)]
#[bizconfig(auto_start = 9000, auto_increment = 1)]
pub enum AppError {
    #[error("Application error: {0}")]
    ApplicationError(#[from] ApplicationError),
    
    #[error("Infrastructure error: {0}")]
    InfrastructureError(#[from] InfrastructureError),
    
    #[error("External dependency error: {0}")]
    ExternalDependencyError(#[from] ExternalDependencyError),
    
    #[error("Unknown error")]
    Unknown,
}
```

## Context Usage Strategy

### 1. Apply the 90/10 Principle

```rust
// 90% scenario - simple error handling
fn validate_email(email: &str) -> Result<(), ValidationError> {
    if !email.contains('@') {
        return Err(ValidationError::InvalidEmailFormat { 
            email: email.to_string() 
        });
    }
    Ok(())
}

// 10% scenario - complex error handling (needs context)
fn process_user_registration(
    user_data: UserData
) -> Result<u64, ContextualError<AppError>> {
    // Validate user input
    validate_user_input(&user_data)
        .with_context("Validating user registration data")?;
    
    // Check if user exists
    check_user_existence(&user_data.email)
        .with_context("Checking if user already exists")?;
    
    // Save user
    let user_id = save_user_to_database(&user_data)
        .with_context("Saving user information to database")?;
    
    // Send welcome email
    send_welcome_email(&user_data.email)
        .with_context("Sending welcome email")?;
    
    Ok(user_id)
}
```

### 2. Hierarchical Context

```rust
fn complex_business_process() -> Result<ProcessResult, ContextualError<AppError>> {
    // Step 1: Data preparation
    let data = prepare_data()
        .with_context("Preparing business data")?;
    
    // Step 2: Business logic processing
    let processed = process_business_logic(&data)
        .with_context("Executing business logic processing")?;
    
    // Step 3: Result persistence
    let result = persist_result(&processed)
        .with_context("Persisting processing result")?;
    
    Ok(result)
}

// Nested context example
fn prepare_data() -> Result<BusinessData, ContextualError<AppError>> {
    let config = load_configuration()
        .with_context("Loading application configuration")?;
    
    let external_data = fetch_external_data(&config)
        .with_context("Fetching external data")?;
    
    let validated_data = validate_data(&external_data)
        .with_context("Validating data integrity")?;
    
    Ok(validated_data)
}
```

## Performance Optimization

### 1. Avoid Unnecessary Allocations

```rust
// ✅ Good practice - use &'static str to reduce allocations
#[derive(BizError, Error)]
#[bizconfig(code_type = "&'static str")]
pub enum OptimizedError {
    #[bizcode("USER_NOT_FOUND")]
    #[error("User not found")]
    UserNotFound,
    
    #[bizcode("INVALID_INPUT")]
    #[error("Invalid input")]
    InvalidInput,
}

// ❌ Avoid - frequent string allocations
impl BizError for ExpensiveError {
    fn msg(&self) -> String {
        format!("Error occurred at: {}", chrono::Utc::now()) // Allocates on every call
    }
}
```

### 2. Conditional Context Addition

```rust
fn conditional_context_handling(
    debug_mode: bool,
    data: &InputData
) -> Result<ProcessedData, AppError> {
    if debug_mode {
        // Development/debug mode - add detailed context
        process_data(data)
            .with_context(format!("Processing data: {:?}", data))
            .map_err(|e| e.into_inner())
    } else {
        // Production mode - simple error handling
        process_data(data)
            .map_biz()
    }
}
```

### 3. Error Code Caching

```rust
use std::sync::OnceLock;

static ERROR_MESSAGES: OnceLock<HashMap<u32, &'static str>> = OnceLock::new();

impl BizError for CachedError {
    fn code(&self) -> u32 {
        match self {
            CachedError::UserNotFound => 1001,
            CachedError::PermissionDenied => 1002,
            CachedError::InvalidData => 1003,
        }
    }
    
    fn name(&self) -> &str {
        match self {
            CachedError::UserNotFound => "UserNotFound",
            CachedError::PermissionDenied => "PermissionDenied", 
            CachedError::InvalidData => "InvalidData",
        }
    }
}
```

## Testing Strategy

### 1. Error Code Consistency Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_codes_consistency() {
        // Ensure error code consistency
        let error1 = UserError::UserNotFound { user_id: 1 };
        let error2 = UserError::UserNotFound { user_id: 2 };
        
        assert_eq!(error1.code(), error2.code());
        assert_eq!(error1.name(), error2.name());
    }
    
    #[test]
    fn test_error_code_uniqueness() {
        // Ensure error code uniqueness (within same enum)
        let errors = vec![
            UserError::UserNotFound { user_id: 1 },
            UserError::UserExists { username: "test".to_string() },
            UserError::PermissionDenied,
        ];
        
        let mut codes = HashSet::new();
        for error in errors {
            assert!(codes.insert(error.code()), "Duplicate error code: {}", error.code());
        }
    }
}
```

### 2. Context Information Tests

```rust
#[test]
fn test_contextual_error_information() {
    let base_error = UserError::UserNotFound { user_id: 123 };
    let contextual = base_error.with_context("User query operation");
    
    assert_eq!(contextual.code(), 1000);
    assert_eq!(contextual.name(), "UserNotFound");
    assert_eq!(contextual.context(), "User query operation");
    assert!(contextual.location().file().contains("test"));
}
```

### 3. Error Chain Tests

```rust
#[test]
fn test_error_chain() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let system_error = SystemError::from(io_error);
    let app_error = AppError::from(system_error);
    let contextual = app_error.with_context("Application initialization");
    
    // Verify error chain
    assert_eq!(contextual.code(), 8000);
    assert!(std::error::Error::source(&contextual).is_some());
    
    // Verify error chain integrity
    let mut source = std::error::Error::source(&contextual);
    let mut chain_length = 0;
    while let Some(err) = source {
        chain_length += 1;
        source = std::error::Error::source(err);
    }
    assert!(chain_length >= 2);
}
```

## Documentation Guidelines

### 1. Error Code Documentation

```rust
/// User management related errors
/// 
/// # Error Code Ranges
/// - 1000-1099: User query errors
/// - 1100-1199: User creation errors
/// - 1200-1299: User update errors
/// - 1300-1399: User deletion errors
/// 
/// # Error Code Reference
/// 
/// | Code | Name | Description | Solution |
/// |------|------|-------------|----------|
/// | 1000 | UserNotFound | User not found | Check if user ID is correct |
/// | 1001 | UserExists | User already exists | Use different username or email |
/// | 1002 | PermissionDenied | Permission denied | Check user permission settings |
/// 
/// # Usage Example
/// 
/// ```rust
/// match find_user(user_id) {
///     Ok(user) => println!("Found user: {}", user.name),
///     Err(e) => match e.code() {
///         1000 => println!("User not found, please check user ID"),
///         1002 => println!("Permission denied, please contact administrator"),
///         _ => println!("Other error: {}", e),
///     }
/// }
/// ```
#[derive(BizError, Error)]
#[bizconfig(auto_start = 1000, auto_increment = 1)]
pub enum UserError {
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: u64 },
    
    #[error("User already exists: {username}")]
    UserExists { username: String },
    
    #[error("Permission denied")]
    PermissionDenied,
}
```

### 2. API Documentation Examples

```rust
/// Create a new user
/// 
/// # Arguments
/// - `user_data`: User data
/// 
/// # Returns
/// - `Ok(u64)`: Successfully created user ID
/// - `Err(ContextualError<UserError>)`: Creation failed error information
/// 
/// # Possible Errors
/// - `UserError::UserExists`: User already exists
/// - `UserError::InvalidEmail`: Invalid email format
/// - `UserError::WeakPassword`: Password too weak
/// 
/// # Example
/// 
/// ```rust
/// let user_data = UserData {
///     username: "alice".to_string(),
///     email: "alice@example.com".to_string(),
///     password: "secure_password".to_string(),
/// };
/// 
/// match create_user(user_data) {
///     Ok(user_id) => println!("User created successfully, ID: {}", user_id),
///     Err(e) => {
///         eprintln!("User creation failed:");
///         eprintln!("Error code: {}", e.code());
///         eprintln!("Error message: {}", e);
///         eprintln!("Context: {}", e.context());
///     }
/// }
/// ```
pub fn create_user(user_data: UserData) -> Result<u64, ContextualError<UserError>> {
    // Implementation omitted...
}
```

## Version Management

### 1. Error Code Version Compatibility

```rust
// v1.0.0 version
#[derive(BizError, Error)]
pub enum UserErrorV1 {
    #[bizcode(1000)]
    #[error("User not found")]
    UserNotFound,
    
    #[bizcode(1001)]
    #[error("User already exists")]
    UserExists,
}

// v2.0.0 version - maintain backward compatibility
#[derive(BizError, Error)]
pub enum UserErrorV2 {
    #[bizcode(1000)]  // Keep original error code
    #[error("User not found")]
    UserNotFound,
    
    #[bizcode(1001)]  // Keep original error code
    #[error("User already exists")]
    UserExists,
    
    #[bizcode(1002)]  // New error code
    #[error("Permission denied")]
    PermissionDenied,
}
```

### 2. Deprecated Error Code Handling

```rust
#[derive(BizError, Error)]
pub enum UserError {
    #[bizcode(1000)]
    #[error("User not found")]
    UserNotFound,
    
    #[bizcode(1001)]
    #[error("User already exists")]
    UserExists,
    
    #[bizcode(1002)]
    #[deprecated(since = "2.0.0", note = "Use PermissionDenied instead")]
    #[error("Access denied")]
    AccessDenied,
    
    #[bizcode(1003)]
    #[error("Permission denied")]
    PermissionDenied,
}
```

### 3. Error Code Migration Guide

```rust
// Migration helper functions
impl UserError {
    /// Convert old version error codes to new version
    pub fn migrate_from_v1(old_code: u32) -> Option<Self> {
        match old_code {
            1000 => Some(UserError::UserNotFound),
            1001 => Some(UserError::UserExists),
            1002 => Some(UserError::PermissionDenied), // Old AccessDenied
            _ => None,
        }
    }
}
```

## Summary

Following these best practices will help you:

1. **Build consistent error handling systems**: Through unified error code standards and naming conventions
2. **Improve code maintainability**: Through clear error hierarchies and documentation
3. **Optimize performance**: By avoiding unnecessary allocations and conditional context
4. **Ensure quality**: Through comprehensive testing strategies
5. **Maintain backward compatibility**: Through proper version management strategies

Remember, good error handling is not just about technical implementation - it's an important part of user experience. 