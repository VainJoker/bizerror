#![allow(unused)]
use std::{
    collections::HashMap,
    fs,
    io,
};

use bizerror::*;
use thiserror::Error as ThisError;

/// User service errors
#[derive(BizError, ThisError)]
#[bizconfig(auto_start = 1000, auto_increment = 1)]
pub enum UserServiceError {
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: u64 },

    #[bizcode(1100)]
    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },

    #[error("Password too weak")]
    WeakPassword,

    #[error("User already exists: {username}")]
    UserExists { username: String },

    #[error("Database error: {0}")]
    DatabaseError(#[from] io::Error),

    #[error("Authentication failed")]
    AuthenticationFailed,
}

/// Payment service errors
#[derive(BizError, ThisError)]
#[bizconfig(auto_start = 2000, auto_increment = 1)]
pub enum PaymentServiceError {
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: f64, available: f64 },

    #[bizcode(2100)]
    #[error("Payment method not supported: {method}")]
    UnsupportedPaymentMethod { method: String },

    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },

    #[error("Card expired: {card_number}")]
    CardExpired { card_number: String },

    #[error("User service error: {0}")]
    UserServiceError(#[from] UserServiceError),
}

/// Order service errors
#[derive(BizError, ThisError)]
#[bizconfig(auto_start = 3000, auto_increment = 1)]
pub enum OrderServiceError {
    #[error("Product not found: {product_id}")]
    ProductNotFound { product_id: String },

    #[error(
        "Insufficient inventory: {product_id}, requested {requested}, \
         available {available}"
    )]
    InsufficientInventory {
        product_id: String,
        requested:  u32,
        available:  u32,
    },

    #[error("Order not found: {order_id}")]
    OrderNotFound { order_id: String },

    #[error("Payment error: {0}")]
    PaymentError(#[from] PaymentServiceError),

    #[error("User service error: {0}")]
    UserServiceError(#[from] UserServiceError),
}

/// Application-level errors
#[derive(BizError, ThisError)]
#[bizconfig(auto_start = 9000, auto_increment = 1)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] io::Error),

    #[error("Order service error: {0}")]
    OrderServiceError(#[from] OrderServiceError),

    #[error("User service error: {0}")]
    UserServiceError(#[from] UserServiceError),

    #[error("Payment service error: {0}")]
    PaymentServiceError(#[from] PaymentServiceError),

    #[error("Internal server error")]
    InternalError,
}

/// Mock user database
struct UserDatabase {
    users: HashMap<u64, User>,
}

#[derive(Clone)]
struct User {
    id:       u64,
    username: String,
    email:    String,
    balance:  f64,
}

impl UserDatabase {
    fn new() -> Self {
        let mut users = HashMap::new();
        users.insert(1, User {
            id:       1,
            username: "john_doe".to_string(),
            email:    "john@example.com".to_string(),
            balance:  100.0,
        });
        users.insert(2, User {
            id:       2,
            username: "jane_smith".to_string(),
            email:    "jane@example.com".to_string(),
            balance:  250.0,
        });

        Self { users }
    }

    fn get_user(&self, user_id: u64) -> Result<&User, UserServiceError> {
        self.users
            .get(&user_id)
            .ok_or(UserServiceError::UserNotFound { user_id })
    }

    fn create_user(
        &mut self,
        username: String,
        email: String,
    ) -> Result<u64, UserServiceError> {
        // Validate email format
        if !email.contains('@') {
            return Err(UserServiceError::InvalidEmail { email });
        }

        // Check if user already exists
        if self.users.values().any(|u| u.username == username) {
            return Err(UserServiceError::UserExists { username });
        }

        let user_id = self.users.len() as u64 + 1;
        let user = User {
            id: user_id,
            username,
            email,
            balance: 0.0,
        };

        self.users.insert(user_id, user);
        Ok(user_id)
    }

    fn update_balance(
        &mut self,
        user_id: u64,
        amount: f64,
    ) -> Result<(), UserServiceError> {
        let user = self
            .users
            .get_mut(&user_id)
            .ok_or(UserServiceError::UserNotFound { user_id })?;

        user.balance += amount;
        Ok(())
    }
}

/// Payment service
struct PaymentService {
    user_db: UserDatabase,
}

impl PaymentService {
    const fn new(user_db: UserDatabase) -> Self {
        Self { user_db }
    }

    fn process_payment(
        &mut self,
        user_id: u64,
        amount: f64,
    ) -> Result<String, PaymentServiceError> {
        let user = self.user_db.get_user(user_id)?;

        if user.balance < amount {
            return Err(PaymentServiceError::InsufficientFunds {
                required:  amount,
                available: user.balance,
            });
        }

        // Simulate payment processing
        self.user_db.update_balance(user_id, -amount)?;

        Ok(format!("Payment of ${amount:.2} processed successfully"))
    }
}

/// Order service
struct OrderService {
    payment_service: PaymentService,
    inventory:       HashMap<String, u32>,
}

impl OrderService {
    fn new(payment_service: PaymentService) -> Self {
        let mut inventory = HashMap::new();
        inventory.insert("laptop".to_string(), 5);
        inventory.insert("mouse".to_string(), 20);
        inventory.insert("keyboard".to_string(), 10);

        Self {
            payment_service,
            inventory,
        }
    }

    fn create_order(
        &mut self,
        user_id: u64,
        product_id: &str,
        quantity: u32,
    ) -> Result<String, OrderServiceError> {
        // Check inventory
        let available = self.inventory.get(product_id).ok_or_else(|| {
            OrderServiceError::ProductNotFound {
                product_id: product_id.to_string(),
            }
        })?;

        if *available < quantity {
            return Err(OrderServiceError::InsufficientInventory {
                product_id: product_id.to_string(),
                requested:  quantity,
                available:  *available,
            });
        }

        // Calculate price (simplified)
        let price = match product_id {
            "laptop" => 999.99,
            "mouse" => 29.99,
            "keyboard" => 79.99,
            _ => 0.0,
        };

        let total = price * f64::from(quantity);

        // Process payment
        let payment_result =
            self.payment_service.process_payment(user_id, total)?;

        // Update inventory
        *self
            .inventory
            .get_mut(product_id)
            .expect("product_id not found") -= quantity;

        Ok(format!(
            "Order created: {quantity} x {product_id} for ${total:.2}. \
             {payment_result}"
        ))
    }
}

/// Application service that ties everything together
struct Application {
    order_service: OrderService,
}

impl Application {
    fn new() -> Self {
        // Load configuration (simulated) - using new extended methods
        let _config = fs::read_to_string("app.config")
            .with_context::<AppError>("Loading application configuration")
            .unwrap_or_else(|_| "default_config".to_string());

        let user_db = UserDatabase::new();
        let payment_service = PaymentService::new(user_db);
        let order_service = OrderService::new(payment_service);

        Self { order_service }
    }

    fn place_order(
        &mut self,
        user_id: u64,
        product_id: &str,
        quantity: u32,
    ) -> Result<String, ContextualError<AppError>> {
        // 使用标准库方法替代已删除的方法
        self.order_service
            .create_order(user_id, product_id, quantity)
            .map_err(|e| {
                // 记录错误用于监控
                eprintln!("Order creation failed: {e}");
                e
            })
            .with_context::<AppError>("Placing customer order")
            .map_err(|e| {
                // 尝试提供有用的错误恢复
                match e.inner() {
                    AppError::OrderServiceError(
                        OrderServiceError::ProductNotFound { product_id: _ },
                    ) => {
                        // 可以建议替代产品
                        e
                    }
                    _ => e,
                }
            })
    }

    /// Demonstrate conditional context addition
    fn place_order_with_debug(
        &mut self,
        user_id: u64,
        product_id: &str,
        quantity: u32,
        debug_mode: bool,
    ) -> Result<String, ContextualError<AppError>> {
        self.order_service
            .create_order(user_id, product_id, quantity)
            .map_err(|e| {
                if debug_mode {
                    eprintln!("Debug: Order error details: {e:?}");
                }
                e
            })
            .with_context_if::<AppError>(
                debug_mode,
                format!("Debug order placement for user {user_id}"),
            )
            .map_err(|e| {
                if debug_mode {
                    e
                } else {
                    // 在生产环境中，替换为简单的上下文 - 不需要克隆！
                    e.add_context("Order processing failed")
                }
            })
    }

    /// Demonstrate chaining multiple operations
    fn process_bulk_orders(
        &mut self,
        orders: Vec<(u64, &str, u32)>,
    ) -> Vec<String> {
        let mut results = Vec::new();

        for (user_id, product_id, quantity) in orders {
            let result = self
                .order_service
                .create_order(user_id, product_id, quantity)
                .map_err(|e| {
                    eprintln!(
                        "Bulk order failed for product {product_id}: {e}"
                    );
                    e
                })
                .and_then_biz::<String, _, AppError>(|order_result| {
                    // Additional processing after successful order
                    let processed = format!("Bulk: {order_result}");
                    Ok(processed)
                })
                .unwrap_or_else(|_| {
                    format!(
                        "FAILED: Order for {product_id} could not be processed"
                    )
                });

            results.push(result);
        }

        results
    }

    /// Demonstrate advanced error chain navigation
    #[allow(clippy::unused_self)]
    fn analyze_error_chain(&self, error: &ContextualError<AppError>) -> String {
        use std::fmt::Write;
        let mut analysis = String::new();

        // Basic error information
        let _ = writeln!(analysis, "Error Code: {}", error.code());
        let _ = writeln!(analysis, "Error Type: {}", error.name());
        let _ = writeln!(analysis, "Context: {}", error.context());
        let _ = writeln!(
            analysis,
            "Location: {}:{}",
            error.location().file(),
            error.location().line()
        );

        // Error chain analysis
        let _ = writeln!(analysis, "Chain Depth: {}", error.chain_depth());

        // Root cause analysis
        let root_cause = error.root_cause_message();
        let _ = writeln!(analysis, "Root Cause: {root_cause}");

        // Specific error type detection
        if error.contains_error::<UserServiceError>() {
            analysis.push_str("Contains UserServiceError: Yes\n");
            if let Some(user_error) = error.find_root::<UserServiceError>() {
                let _ = writeln!(
                    analysis,
                    "  User Error Code: {}",
                    user_error.code()
                );
            }
        }

        if error.contains_error::<PaymentServiceError>() {
            analysis.push_str("Contains PaymentServiceError: Yes\n");
            if let Some(payment_error) =
                error.find_root::<PaymentServiceError>()
            {
                let _ = writeln!(
                    analysis,
                    "  Payment Error Code: {}",
                    payment_error.code()
                );
            }
        }

        if error.contains_error::<OrderServiceError>() {
            analysis.push_str("Contains OrderServiceError: Yes\n");
            if let Some(order_error) = error.find_root::<OrderServiceError>() {
                let _ = writeln!(
                    analysis,
                    "  Order Error Code: {}",
                    order_error.code()
                );
            }
        }

        if error.contains_error::<std::io::Error>() {
            analysis.push_str("Contains IO Error: Yes\n");
            if let Some(io_error) = error.find_root::<std::io::Error>() {
                let _ = writeln!(
                    analysis,
                    "  IO Error Kind: {:?}",
                    io_error.kind()
                );
            }
        }

        // Full error chain
        analysis.push_str("Full Error Chain:\n");
        for (i, err_msg) in error.error_chain_messages().iter().enumerate() {
            let _ = writeln!(analysis, "  {}. {}", i + 1, err_msg);
        }

        analysis
    }

    /// Demonstrate `ResultExt` tier usage examples
    fn demonstrate_result_ext_tiers(&mut self) {
        println!("=== ResultExt Tier Usage Examples ===\n");

        // 🔥 Tier 1: Core High-Frequency Methods (80% of use cases)
        println!("🔥 Tier 1 Examples:");

        // with_context - most common usage
        let _config = std::fs::read_to_string("config.json")
            .with_context::<AppError>("Loading application configuration")
            .unwrap_or_else(|_| "default_config".to_string());

        // map_biz - simple conversion
        let _user_result = self
            .order_service
            .payment_service
            .user_db
            .get_user(1)
            .map_biz::<AppError>();

        // with_context - add context to existing error
        let _file_result = std::fs::read_to_string("data.json")
            .with_context::<AppError>("Reading application data");

        // 🔧 Tier 2: Standard Library Methods (替代已删除的方法)
        println!("🔧 Tier 2 Examples (using standard library methods):");

        // 使用标准库的 unwrap_or 替代 or_biz_default
        let default_quantity = "invalid".parse::<u32>().unwrap_or(1);
        println!("Default quantity: {default_quantity}");

        // 使用标准库的 unwrap_or_else 替代 or_biz_else
        let computed_default = "invalid".parse::<u32>().unwrap_or_else(|_| {
            println!("Computing default value...");
            42
        });
        println!("Computed default: {computed_default}");

        // 使用标准库的 or_else 替代 recover_biz_error
        let _recovered_result =
            std::fs::read_to_string("missing.txt").or_else(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok("default file content".to_string())
                } else {
                    Err(e)
                }
            });

        // ⚡ Tier 3: Observation and Advanced Methods
        println!("⚡ Tier 3 Examples:");

        // 使用标准库的 inspect_err 替代 inspect_biz_error
        let _observed = std::fs::read_to_string("debug.log").inspect_err(|e| {
            eprintln!("Debug file read failed: {e}");
        });

        // 使用标准库的 inspect_err 替代 tap_biz_error
        let _tapped =
            std::fs::read_to_string("metrics.json").inspect_err(|e| {
                // Log to monitoring system
                eprintln!("Metrics load failed: {e}");
            });

        // with_context_if - conditional context
        let debug_mode = true;
        let _conditional = std::fs::read_to_string("sensitive.json")
            .with_context_if::<AppError>(
                debug_mode,
                "Loading sensitive data (debug mode)",
            );

        // and_then_biz - chaining operations
        let _chained = std::fs::read_to_string("config.json")
            .and_then_biz::<String, _, AppError>(|content| {
                // Process the content
                Ok(content.to_uppercase())
            });
    }
}

/// Error handling demonstration
fn demonstrate_error_handling() {
    println!("=== Bizerror Real-World Usage Demo ===\n");

    // Initialize application
    let mut app = Application::new();

    // Demonstrate ResultExt tier usage
    println!("🚀 Demonstrating ResultExt Tier Usage:");
    app.demonstrate_result_ext_tiers();
    println!();

    // Successful order
    println!("1. Successful order:");
    match app.place_order(1, "laptop", 1) {
        Ok(result) => println!("   ✅ {result}"),
        Err(e) => print_error("   ❌", &e),
    }

    // User not found
    println!("\n2. User not found:");
    match app.place_order(999, "mouse", 1) {
        Ok(result) => println!("   ✅ {result}"),
        Err(e) => {
            print_error("   ❌", &e);
            println!("\n   🔍 Detailed Error Analysis:");
            println!("{}", app.analyze_error_chain(&e));
        }
    }

    // Product not found
    println!("\n3. Product not found:");
    match app.place_order(1, "tablet", 1) {
        Ok(result) => println!("   ✅ {result}"),
        Err(e) => {
            print_error("   ❌", &e);
            println!("\n   🔍 Detailed Error Analysis:");
            println!("{}", app.analyze_error_chain(&e));
        }
    }

    // Insufficient funds
    println!("\n4. Insufficient funds:");
    match app.place_order(1, "laptop", 10) {
        Ok(result) => println!("   ✅ {result}"),
        Err(e) => {
            print_error("   ❌", &e);
            println!("\n   🔍 Detailed Error Analysis:");
            println!("{}", app.analyze_error_chain(&e));
        }
    }

    // Insufficient inventory
    println!("\n5. Insufficient inventory:");
    match app.place_order(2, "laptop", 10) {
        Ok(result) => println!("   ✅ {result}"),
        Err(e) => {
            print_error("   ❌", &e);
            println!("\n   🔍 Detailed Error Analysis:");
            println!("{}", app.analyze_error_chain(&e));
        }
    }

    // Demonstrate bulk order processing
    println!("\n6. Bulk order processing:");
    let bulk_orders = vec![
        (1, "mouse", 2),
        (2, "keyboard", 1),
        (1, "nonexistent", 1), // This will fail
    ];

    app.process_bulk_orders(bulk_orders);
}

fn print_error(prefix: &str, error: &ContextualError<AppError>) {
    println!("{prefix} Error occurred:");
    println!("     Code: {}", error.code());
    println!("     Type: {}", error.name());
    println!("     Message: {error}");
    println!("     Context: {}", error.context());
    println!(
        "     Location: {}:{}",
        error.location().file(),
        error.location().line()
    );

    // Print error chain
    let mut source = std::error::Error::source(error);
    let mut level = 1;
    while let Some(err) = source {
        println!("     Caused by (level {level}): {err}");
        source = std::error::Error::source(err);
        level += 1;
    }
}

fn main() {
    demonstrate_error_handling();
}
