use bizerror::*;
use thiserror::Error as ThisError;

/// Test basic auto-assignment with default config
#[derive(BizError, ThisError)]
pub enum SimpleError {
    #[error("First error")]
    First,

    #[error("Second error")]
    Second,

    #[error("Third error")]
    Third,
}

/// Test mixed explicit and auto codes
#[derive(BizError, ThisError)]
#[bizconfig(auto_start = 100, auto_increment = 10)]
pub enum MixedError {
    #[error("Auto error 1")]
    Auto1, // Should be 100

    #[bizcode(999)]
    #[error("Explicit error")]
    Explicit, // Should be 999

    #[error("Auto error 2")]
    Auto2, // Should be 110
}

/// Test string codes
#[derive(BizError, ThisError)]
#[bizconfig(code_type = "&'static str")]
pub enum StringError {
    #[error("Auto string error")]
    AutoString, // Should be "0"

    #[bizcode("CUSTOM")]
    #[error("Custom string error")]
    CustomString, // Should be "CUSTOM"

    #[error("Next auto string")]
    NextAutoString, // Should be "1"
}

/// Test duplicate codes (should be allowed)
#[derive(BizError, ThisError)]
pub enum DuplicateError {
    #[bizcode(500)]
    #[error("First 500")]
    First500,

    #[bizcode(500)]
    #[error("Second 500")]
    Second500, // Duplicate allowed

    #[error("Auto")]
    Auto, // Should be 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_auto_assignment() {
        let first = SimpleError::First;
        let second = SimpleError::Second;
        let third = SimpleError::Third;

        assert_eq!(first.code(), 0);
        assert_eq!(second.code(), 1);
        assert_eq!(third.code(), 2);

        assert_eq!(first.name(), "First");
        assert_eq!(second.name(), "Second");
        assert_eq!(third.name(), "Third");
    }

    #[test]
    fn test_mixed_explicit_auto() {
        let auto1 = MixedError::Auto1;
        let explicit = MixedError::Explicit;
        let auto2 = MixedError::Auto2;

        assert_eq!(auto1.code(), 100);
        assert_eq!(explicit.code(), 999);
        assert_eq!(auto2.code(), 110);

        assert_eq!(auto1.name(), "Auto1");
        assert_eq!(explicit.name(), "Explicit");
        assert_eq!(auto2.name(), "Auto2");
    }

    #[test]
    fn test_string_codes() {
        let auto_string = StringError::AutoString;
        let custom_string = StringError::CustomString;
        let next_auto_string = StringError::NextAutoString;

        assert_eq!(auto_string.code(), "0");
        assert_eq!(custom_string.code(), "CUSTOM");
        assert_eq!(next_auto_string.code(), "1");
    }

    #[test]
    fn test_duplicate_codes_allowed() {
        let first_500 = DuplicateError::First500;
        let second_500 = DuplicateError::Second500;
        let auto = DuplicateError::Auto;

        assert_eq!(first_500.code(), 500);
        assert_eq!(second_500.code(), 500); // Duplicate allowed
        assert_eq!(auto.code(), 0);
    }

    #[test]
    fn test_debug_output() {
        let error = MixedError::Auto1;
        let debug_str = format!("{error:?}");

        assert!(debug_str.contains("MixedError"));
        assert!(debug_str.contains("Auto1"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_contextual_error() {
        let error = SimpleError::First;
        let contextual = error.with_context("Test context");

        assert_eq!(contextual.code(), 0);
        assert_eq!(contextual.name(), "First");
        assert_eq!(contextual.context(), "Test context");
    }
}
