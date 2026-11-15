//! Input validation shared by services. Messages are written for librarians.

use crate::error::{AppError, AppResult};

pub const PASSWORD_MIN: usize = 10;

/// Trims and checks a required text field.
pub fn required(label: &str, value: &str, max: usize) -> AppResult<String> {
    let v = value.trim();
    if v.is_empty() {
        return Err(AppError::Validation(format!("{label} is required.")));
    }
    if v.chars().count() > max {
        return Err(AppError::Validation(format!("{label} must be at most {max} characters.")));
    }
    Ok(v.to_string())
}

/// Trims an optional text field; empty becomes `None`.
pub fn optional(label: &str, value: Option<&str>, max: usize) -> AppResult<Option<String>> {
    match value.map(str::trim) {
        None | Some("") => Ok(None),
        Some(v) if v.chars().count() > max => {
            Err(AppError::Validation(format!("{label} must be at most {max} characters.")))
        }
        Some(v) => Ok(Some(v.to_string())),
    }
}

pub fn in_range(label: &str, value: i64, min: i64, max: i64) -> AppResult<i64> {
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(AppError::Validation(format!("{label} must be between {min} and {max}.")))
    }
}

/// Usernames: 3-40 characters of letters, digits, '.', '_' or '-'.
pub fn username(value: &str) -> AppResult<String> {
    let v = value.trim();
    let ok = (3..=40).contains(&v.chars().count())
        && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-');
    if ok {
        Ok(v.to_string())
    } else {
        Err(AppError::Validation(
            "Username must be 3 to 40 characters: letters, digits, '.', '_' or '-'.".into(),
        ))
    }
}

/// Length-based password rule (no composition rules). Never log the value.
pub fn password(value: &str) -> AppResult<()> {
    if value.chars().count() < PASSWORD_MIN {
        return Err(AppError::Validation(format!("Password must be at least {PASSWORD_MIN} characters.")));
    }
    if value.chars().count() > 200 {
        return Err(AppError::Validation("Password is too long.".into()));
    }
    Ok(())
}

/// A calendar date written YYYY-MM-DD (leap years respected).
pub fn date(label: &str, value: &str) -> AppResult<String> {
    let bad = || AppError::Validation(format!("{label} must be a real date written YYYY-MM-DD."));
    let v = value.trim();
    let parts: Vec<&str> = v.split('-').collect();
    let [y, m, d] = parts.as_slice() else { return Err(bad()) };
    if y.len() != 4 || m.len() != 2 || d.len() != 2 {
        return Err(bad());
    }
    let (y, m, d): (i32, u32, u32) = (
        y.parse().map_err(|_| bad())?,
        m.parse().map_err(|_| bad())?,
        d.parse().map_err(|_| bad())?,
    );
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if leap { 29 } else { 28 },
        _ => return Err(bad()),
    };
    if y < 1000 || d < 1 || d > days {
        return Err(bad());
    }
    Ok(v.to_string())
}

pub fn currency(value: &str) -> AppResult<String> {
    let v = value.trim().to_ascii_uppercase();
    if v.len() == 3 && v.chars().all(|c| c.is_ascii_uppercase()) {
        Ok(v)
    } else {
        Err(AppError::Validation("Currency must be a 3-letter code such as INR or USD.".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_trims_and_limits() {
        assert_eq!(required("Name", "  Library  ", 10).unwrap(), "Library");
        assert!(required("Name", "   ", 10).is_err());
        assert!(required("Name", "abcdefghijk", 10).is_err());
    }

    #[test]
    fn optional_empty_is_none() {
        assert_eq!(optional("Address", Some("  "), 5).unwrap(), None);
        assert_eq!(optional("Address", None, 5).unwrap(), None);
        assert!(optional("Address", Some("abcdef"), 5).is_err());
    }

    #[test]
    fn username_rules() {
        assert!(username("rekha.nair").is_ok());
        for bad in ["ab", "has space", "semi;colon", &"x".repeat(41)] {
            assert!(username(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn password_rules_are_length_based() {
        assert!(password("correct horse").is_ok());
        assert!(password("short").is_err());
        assert!(password(&"x".repeat(201)).is_err());
    }

    #[test]
    fn dates_are_real_calendar_dates() {
        for good in ["2026-11-15", "2024-02-29", "2000-02-29"] {
            assert_eq!(date("D", good).unwrap(), good);
        }
        for bad in ["2026-02-29", "1900-02-29", "2026-13-01", "2026-00-10", "2026-04-31", "26-01-01", "2026/01/01", "", "2026-1-1"] {
            assert!(date("D", bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn currency_normalises() {
        assert_eq!(currency(" inr ").unwrap(), "INR");
        assert!(currency("RS").is_err());
        assert!(currency("1NR").is_err());
    }
}
