use bizerror::*;

#[derive(BizError, thiserror::Error)]
pub enum ApiError {
    #[bizcode(2001)]
    #[error("UTF-8 conversion error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[bizcode(2002)]
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

fn from_utf8_error() -> Result<String, ApiError> {
    let bytes = vec![0, 159]; // Invalid UTF-8
    let s = String::from_utf8(bytes)?;
    Ok(s)
}

fn main() {
    println!("🎯 BizError Simple Example");
    println!("=========================");

    // Test basic error functionality
    let result = from_utf8_error();
    match result {
        Ok(s) => println!("Success: {s}"),
        Err(e) => {
            println!("Error occurred:");
            println!("  Code: {}", e.code());
            println!("  Name: {}", e.name());
            println!("  Message: {e}");
            println!("  Display: {e}");
        }
    }

    // Test contextual error
    let contextual_result: Result<String, ContextualError<ApiError>> =
        from_utf8_error().with_context("Processing user input");

    match contextual_result {
        Ok(s) => println!("Success: {s}"),
        Err(e) => {
            println!("\nContextual error:");
            println!("  Code: {}", e.code());
            println!("  Name: {}", e.name());
            println!("  Context: {}", e.context());
            println!(
                "  Location: {}:{}:{}",
                e.location().file(),
                e.location().line(),
                e.location().column()
            );
        }
    }
}
