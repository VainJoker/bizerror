use bizerror::bizerror;

#[bizerror]
pub enum IdealApiError {
    #[bizerror(2001, "FromUtf8Error Error")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[bizerror(2002, "InvalidEmail Error {email}")]
    InvalidEmail { email: String },
}

/// 测试理想体验的多个错误使用同一错误码
#[bizerror]
pub enum IdealValidationError {
    // 这些错误都使用同一个错误码 4001，表示"输入验证失败"
    #[bizerror(4001, "Invalid email format: {email}")]
    InvalidEmail { email: String },
    
    #[bizerror(4001, "Invalid phone number: {phone}")]
    InvalidPhone { phone: String },
    
    #[bizerror(4001, "Invalid password: too short")]
    PasswordTooShort,
    
    // 不同类别的验证错误使用不同错误码
    #[bizerror(4002, "Required field missing: {field}")]
    MissingField { field: String },
}

#[test]
fn test_ideal_from_utf8_error() {
    #[allow(clippy::unwrap_used)]
    let res = ideal_from_utf8_error().unwrap_err();
    println!("{res}");
    println!("Debug: {res:?}");
    
    // 测试错误码
    assert_eq!(res.code(), 2001);
}

fn ideal_from_utf8_error() -> Result<String, IdealApiError> {
    let bytes = vec![0, 159];
    let s = String::from_utf8(bytes)?; // 自动转换
    Ok(s)
}

#[test]
fn test_ideal_invalid_email() {
    let error = IdealApiError::InvalidEmail { 
        email: "invalid-email".to_string() 
    };
    
    assert_eq!(error.code(), 2002);
    
    // 测试显示消息
    let display_msg = format!("{}", error);
    assert!(display_msg.contains("InvalidEmail Error"));
    
    println!("Display: {}", error);
    println!("Debug: {:?}", error);
}

#[test]
fn test_ideal_same_error_code_different_errors() {
    // 创建多个使用相同错误码的错误
    let email_error = IdealValidationError::InvalidEmail { 
        email: "not-an-email".to_string() 
    };
    let phone_error = IdealValidationError::InvalidPhone { 
        phone: "123".to_string() 
    };
    let password_error = IdealValidationError::PasswordTooShort;
    let missing_field_error = IdealValidationError::MissingField { 
        field: "username".to_string() 
    };
    
    // 验证错误码
    assert_eq!(email_error.code(), 4001);
    assert_eq!(phone_error.code(), 4001);
    assert_eq!(password_error.code(), 4001);
    assert_eq!(missing_field_error.code(), 4002); // 不同的错误码
    
    // 验证消息内容不同
    println!("=== 理想体验：相同错误码(4001)的不同错误 ===");
    println!("Email Error: {}", email_error);
    println!("Phone Error: {}", phone_error);
    println!("Password Error: {}", password_error);
    println!("Missing Field Error (4002): {}", missing_field_error);
    
    // 验证显示的消息确实不同
    assert!(format!("{}", email_error).contains("Invalid email format"));
    assert!(format!("{}", phone_error).contains("Invalid phone number"));
    assert!(format!("{}", password_error).contains("Invalid password"));
    assert!(format!("{}", missing_field_error).contains("Required field missing"));
}

#[test]
fn test_ideal_error_code_grouping_use_case() {
    println!("=== 理想体验业务场景：错误码分组 ===");
    
    // 模拟用户注册表单验证
    let validation_errors = vec![
        IdealValidationError::InvalidEmail { email: "bad-email".to_string() },
        IdealValidationError::InvalidPhone { phone: "123".to_string() },
        IdealValidationError::PasswordTooShort,
        IdealValidationError::MissingField { field: "username".to_string() },
    ];
    
    // 按错误码分组统计
    let mut code_counts = std::collections::HashMap::new();
    for error in &validation_errors {
        *code_counts.entry(error.code()).or_insert(0) += 1;
    }
    
    println!("错误码统计:");
    for (code, count) in code_counts {
        println!("  错误码 {}: {} 个错误", code, count);
    }
    
    // 验证分组结果
    assert_eq!(validation_errors.iter().filter(|e| e.code() == 4001).count(), 3);
    assert_eq!(validation_errors.iter().filter(|e| e.code() == 4002).count(), 1);
} 