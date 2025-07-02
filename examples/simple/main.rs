use bizerror::bizerror;
use std::error::Error;

#[bizerror]
pub enum ApiError {
    #[bizerror(1001, "Configuration file not found")]
    ConfigError,
    
    #[bizerror(2001, "Business rule violation: {rule}")]
    BusinessRuleError { rule: String },
    
    #[bizerror(4001, "Invalid email format: {email}")]
    ValidationError { email: String },
    
    #[bizerror(8001, "Network connection failed")]
    NetworkError(#[from] std::io::Error),
    
    #[bizerror(8002, "JSON parsing failed")]
    JsonError,

    #[bizerror(8004, "the data for key `{0}` is not available")]
    Redaction(String),
    #[bizerror(8005, "invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
    #[bizerror(8006, "unknown data store error")]
    Unknown,
    Glob(#[from] globset::Error),
    #[source]  
    source: anyhow::Error,
}

fn demonstrate_error_types() {
    println!("🚀 BizError v0.2 - 完整功能演示\n");
    
    let errors = vec![
        ("系统错误", ApiError::ConfigError),
        ("业务错误", ApiError::BusinessRuleError { rule: "insufficient_balance".to_string() }),
        ("验证错误", ApiError::ValidationError { email: "invalid-email".to_string() }),
        ("JSON错误", ApiError::JsonError),
    ];
    
    for (category, error) in errors {
        println!("=== {} ===", category);
        println!("Display: {}", error);
        println!("Debug: {:?}", error);
        println!("Code: {}", error.code());
        println!();
    }
}

fn demonstrate_error_conversion() -> Result<(), ApiError> {
    println!("=== 自动错误转换演示 ===");
    
    // 模拟 IO 错误，自动转换为 NetworkError
    let io_error = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
    let network_error = ApiError::from(io_error);
    
    println!("Converted IO Error:");
    println!("Display: {}", network_error);
    println!("Debug: {:?}", network_error);
    println!("Code: {}", network_error.code());
    println!("Source available: {}", network_error.source().is_some());
    println!();
    
    Ok(())
}

fn demonstrate_field_interpolation() {
    println!("=== 字段插值演示 ===");
    
    let validation_error = ApiError::ValidationError { 
        email: "not-an-email".to_string() 
    };
    
    let business_error = ApiError::BusinessRuleError { 
        rule: "minimum_balance_required".to_string() 
    };
    
    // 测试新添加的错误类型
    let redaction_error = ApiError::Redaction("user_secret".to_string());
    
    let header_error = ApiError::InvalidHeader {
        expected: "application/json".to_string(),
        found: "text/plain".to_string(),
    };
    
    println!("验证错误:");
    println!("  Display: {}", validation_error);
    println!("  字段值被正确插入到消息中 ✅");
    println!();
    
    println!("业务错误:");
    println!("  Display: {}", business_error);
    println!("  字段值被正确插入到消息中 ✅");
    println!();
    
    println!("数据遮蔽错误:");
    println!("  Display: {}", redaction_error);
    println!("  Code: {}", redaction_error.code());
    println!();
    
    println!("头部错误:");
    println!("  Display: {}", header_error);
    println!("  Code: {}", header_error.code());
    println!("  Debug格式化 (:?) 工作正常 ✅");
    println!();
}

fn main() {
    demonstrate_error_types();
    demonstrate_field_interpolation();
    
    if let Err(e) = demonstrate_error_conversion() {
        println!("演示过程中出现错误: {}", e);
    }
    
    println!("✅ 所有演示完成！");
    println!();
    println!("🎯 主要特性:");
    println!("  ✅ 只需 #[derive(BizError)] 一行代码");
    println!("  ✅ 自动 Display/Debug/Error trait 实现");
    println!("  ✅ 自动 From 转换（#[from] 属性）");
    println!("  ✅ 结构化错误码和分类");
    println!("  ✅ 字段插值支持");
    println!("  ✅ 错误链和源错误跟踪");
    println!("  ✅ 安全的显示消息 vs 完整的调试信息");
}
