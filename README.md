# bizerror

[![Crates.io](https://img.shields.io/crates/v/bizerror.svg)](https://crates.io/crates/bizerror)
[![docs.rs](https://docs.rs/bizerror/badge.svg)](https://docs.rs/bizerror)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/vainjoker/bizerror)

**bizerror** 是一个业务错误增强库，为 Rust 错误处理提供结构化的错误码和业务功能，内部集成了完整的错误处理能力。

## ✨ 核心特性

- 🎯 **极简体验** - 只需一个 `#[bizerror]` 即可获得完整功能
- 🏷️ **业务错误码** - 结构化错误码管理，自动分类
- 🔄 **完整错误处理** - 内置 Display、Debug、Error trait 实现
- 📍 **智能转换** - 自动 From 实现和错误链支持
- 🛡️ **安全分层** - Display 显示安全消息，Debug 显示完整调试信息
- 🎯 **零学习成本** - 符合 Rust 标准错误处理模式

## 🚀 快速开始

添加依赖到 `Cargo.toml`:

```toml
[dependencies]
bizerror = "0.2"
```

## 📖 使用方式

### 🎯 理想体验（零配置，推荐）

只需要一个 `#[bizerror]` 属性：

```rust
use bizerror::bizerror;

#[bizerror]
pub enum ApiError {
    #[bizerror(4001, "Invalid input: {field}")]
    InvalidInput { field: String },
    
    #[bizerror(8001, "Database connection failed")]
    DatabaseError(#[from] std::io::Error),
    
    #[bizerror(2001, "User authentication failed")]
    AuthenticationFailed,
}

fn example() -> Result<(), ApiError> {
    Err(ApiError::InvalidInput { 
        field: "email".to_string() 
    })
}

fn main() {
    if let Err(error) = example() {
        // 对外显示：安全消息
        println!("Error: {}", error);                    // "Invalid input: email"
        
        // 对内调试：完整信息
        println!("Debug: {:?}", error);                  // 包含错误码、上下文等完整信息
        
        // 业务方法
        println!("Code: {}", error.code());              // 4001
    }
}
```

### 🔧 手动设置（向后兼容）

需要手动添加 derives 和属性：

```rust
use bizerror::BizError;

#[derive(BizError, thiserror::Error, Debug)]
pub enum ApiError {
    #[bizerror(4001, "Invalid input: {field}")]
    #[error("Invalid input: {field}")]
    InvalidInput { field: String },
    
    #[bizerror(8001, "Database connection failed")]
    #[error("Database connection failed")]
    DatabaseError(#[from] std::io::Error),
}
```

### 自动错误转换

```rust
#[derive(BizError)]
pub enum AppError {
    #[bizerror(8001, "Network error occurred")]
    NetworkError(#[from] std::io::Error),
    
    #[bizerror(8002, "JSON parsing failed")]
    JsonError(#[from] serde_json::Error),
}

fn network_operation() -> Result<String, AppError> {
    // 自动转换 std::io::Error 到 AppError::NetworkError
    let content = std::fs::read_to_string("config.json")?;
    
    // 自动转换 serde_json::Error 到 AppError::JsonError  
    let parsed: serde_json::Value = serde_json::from_str(&content)?;
    
    Ok(parsed.to_string())
}
```

### 字段插值支持

```rust
#[derive(BizError)]
pub enum ValidationError {
    #[bizerror(4001, "Invalid email format: {email}")]
    InvalidEmail { email: String },
    
    #[bizerror(4002, "Password too short, minimum {min_length} characters required")]
    PasswordTooShort { min_length: usize },
    
    #[bizerror(4003, "User {user_id} does not have {permission} permission")]
    InsufficientPermission { user_id: u64, permission: String },
}
```

## 🔧 自动提供的功能

每个 `#[derive(BizError)]` 自动获得：

### 核心方法
```rust
impl YourError {
    /// 获取业务错误码
    pub fn code(&self) -> u16;
    
    /// 获取错误上下文（TODO）
    pub fn context(&self) -> &[String];
    
    /// 获取错误位置（TODO）  
    pub fn location(&self) -> Option<&'static Location>;
    
    /// 添加上下文（TODO）
    pub fn with_context(self, ctx: impl ToString) -> Self;
    
    /// 添加位置信息（TODO）
    pub fn with_location(self) -> Self;
}
```

### 标准 Trait 实现
- **Display**: 显示用户友好的错误消息
- **Debug**: 显示包含错误码、上下文等完整调试信息
- **Error**: 完整的错误链支持
- **From**: 为带有 `#[from]` 的变体自动生成转换

## 🎯 设计理念

### 极简用户体验
- **理想方案** - 只需 `#[bizerror]` 零配置
- **兼容方案** - 支持传统 derive 模式
- **符合直觉** - 遵循 Rust 标准错误处理模式

### 安全分层显示
- **Display** (对外) - 显示用户安全的错误消息
- **Debug** (对内) - 显示完整的调试信息，包含敏感细节

### 业务友好
- **结构化错误码** - 便于监控、统计和分析
- **类型安全** - 编译时保证错误处理完整性

## 🛠️ 开发

```bash
# 克隆并构建
git clone https://github.com/vainjoker/bizerror.git
cd bizerror
cargo build

# 运行测试
cargo test

# 运行示例
cargo run --example simple
```

## 🗺️ 路线图

### Version 0.2.0 ✅
- [x] 完整的独立错误处理实现
- [x] 自动 Display/Debug/Error trait 生成
- [x] 字段插值支持
- [x] **理想体验：零配置 `#[bizerror]` 属性**

### Version 0.3.0 🚧  
- [ ] 上下文和位置追踪实现
- [ ] 序列化支持 (serde integration)
- [ ] Web 框架集成帮助宏
- [ ] 国际化 (i18n) 支持

### Version 0.4.0 📋
- [ ] 监控和指标集成
- [ ] 自定义错误码范围
- [ ] 高级业务逻辑属性

## 📄 许可证

在以下许可证之一下获得许可：
- [Apache License, Version 2.0](LICENSE-APACHE)  
- [MIT License](LICENSE-MIT)

由您选择。

---

## 🙏 致谢

特别感谢：
- **thiserror** - 出色的基础和启发
- **Rust Error Handling Working Group** - 生态系统指导
- **社区贡献者** - 反馈和用例

**准备好增强您的错误处理了吗？将 `bizerror` 添加到您的项目中，一行代码获得结构化业务错误码！🚀**