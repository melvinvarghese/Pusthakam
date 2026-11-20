//! Accession numbers: a configurable prefix plus a zero-padded counter, e.g. "LIB-000123".

use crate::error::{AppError, AppResult};

pub const MAX_PREFIX: usize = 12;

pub fn validate_prefix(prefix: &str) -> AppResult<String> {
    let p = prefix.trim();
    let ok = p.chars().count() <= MAX_PREFIX
        && p.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '/' | '.'));
    if ok {
        Ok(p.to_string())
    } else {
        Err(AppError::Validation(format!(
            "The prefix can be up to {MAX_PREFIX} letters, digits, '-', '_', '/' or '.'."
        )))
    }
}

pub fn format(prefix: &str, padding: u32, n: i64) -> String {
    format!("{prefix}{n:0width$}", width = padding as usize)
}

/// A manually typed accession number or barcode: printable ASCII without spaces (so it scans and prints reliably).
pub fn validate_code(label: &str, code: &str) -> AppResult<String> {
    let c = code.trim();
    let ok = !c.is_empty() && c.len() <= 40 && c.bytes().all(|b| b.is_ascii_graphic());
    if ok {
        Ok(c.to_string())
    } else {
        Err(AppError::Validation(format!(
            "{label} must be 1 to 40 letters, digits or symbols, with no spaces."
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_with_prefix_and_padding() {
        assert_eq!(format("", 6, 42), "000042");
        assert_eq!(format("LIB-", 5, 123), "LIB-00123");
        assert_eq!(format("X", 3, 12345), "X12345", "never truncates");
    }

    #[test]
    fn prefix_rules() {
        assert_eq!(validate_prefix(" LIB/ ").unwrap(), "LIB/");
        assert!(validate_prefix("has space").is_err());
        assert!(validate_prefix("ABCDEFGHIJKLM").is_err());
        assert!(validate_prefix("").is_ok());
    }

    #[test]
    fn codes_must_be_scannable() {
        assert!(validate_code("Barcode", "004217").is_ok());
        assert!(validate_code("Barcode", "A-1/b").is_ok());
        assert_eq!(
            validate_code("Barcode", "  A1  ").unwrap(),
            "A1",
            "surrounding spaces are trimmed"
        );
        for bad in ["", "   ", "has space", "ta\tb", "മലയാളം", &"9".repeat(41)] {
            assert!(validate_code("Barcode", bad).is_err(), "{bad:?}");
        }
    }
}
