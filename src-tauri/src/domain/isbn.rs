//! ISBN-10 / ISBN-13 validation and conversion. Pure functions, no I/O.

/// Removes hyphens and spaces and upper-cases a trailing X.
pub fn clean(raw: &str) -> String {
    raw.chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect::<String>()
        .to_ascii_uppercase()
}

pub fn is_valid_isbn10(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 {
        return false;
    }
    let mut sum = 0u32;
    for (i, &c) in b.iter().enumerate() {
        let v = match c {
            b'0'..=b'9' => (c - b'0') as u32,
            b'X' if i == 9 => 10,
            _ => return false,
        };
        sum += v * (10 - i as u32);
    }
    sum % 11 == 0
}

pub fn is_valid_isbn13(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 13 || !b.iter().all(u8::is_ascii_digit) {
        return false;
    }
    let sum: u32 = b
        .iter()
        .enumerate()
        .map(|(i, &c)| (c - b'0') as u32 * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    sum % 10 == 0
}

fn check13(first12: &str) -> char {
    let sum: u32 = first12
        .bytes()
        .enumerate()
        .map(|(i, c)| (c - b'0') as u32 * if i % 2 == 0 { 1 } else { 3 })
        .sum();
    char::from_digit((10 - sum % 10) % 10, 10).unwrap_or('0')
}

/// ISBN-10 (valid, cleaned) to ISBN-13 with the 978 prefix.
pub fn isbn10_to_13(isbn10: &str) -> Option<String> {
    if !is_valid_isbn10(isbn10) {
        return None;
    }
    let body = format!("978{}", &isbn10[..9]);
    let check = check13(&body);
    Some(format!("{body}{check}"))
}

/// ISBN-13 starting with 978 back to ISBN-10. 979 numbers have no ISBN-10.
pub fn isbn13_to_10(isbn13: &str) -> Option<String> {
    if !is_valid_isbn13(isbn13) || !isbn13.starts_with("978") {
        return None;
    }
    let body = &isbn13[3..12];
    let sum: u32 = body
        .bytes()
        .enumerate()
        .map(|(i, c)| (c - b'0') as u32 * (10 - i as u32))
        .sum();
    let check = (11 - sum % 11) % 11;
    let ch = if check == 10 {
        'X'
    } else {
        char::from_digit(check, 10).unwrap_or('0')
    };
    Some(format!("{body}{ch}"))
}

/// Both forms of a valid ISBN, or `None` if it is not a valid ISBN-10 or ISBN-13.
pub fn both_forms(raw: &str) -> Option<(Option<String>, String)> {
    let s = clean(raw);
    if is_valid_isbn13(&s) {
        Some((isbn13_to_10(&s), s))
    } else if is_valid_isbn10(&s) {
        isbn10_to_13(&s).map(|i13| (Some(s), i13))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_known_numbers() {
        assert!(is_valid_isbn10("0306406152"));
        assert!(is_valid_isbn10("080442957X"));
        assert!(is_valid_isbn13("9780306406157"));
        assert!(!is_valid_isbn10("0306406153"));
        assert!(!is_valid_isbn13("9780306406158"));
        assert!(!is_valid_isbn10("03064061"));
        assert!(!is_valid_isbn13("97803064061A7"));
    }

    #[test]
    fn converts_both_ways() {
        assert_eq!(isbn10_to_13("0306406152").as_deref(), Some("9780306406157"));
        assert_eq!(isbn13_to_10("9780306406157").as_deref(), Some("0306406152"));
        assert_eq!(isbn10_to_13("080442957X").as_deref(), Some("9780804429573"));
        assert_eq!(isbn13_to_10("9780804429573").as_deref(), Some("080442957X"));
        assert_eq!(
            isbn13_to_10("9791090636071"),
            None,
            "979 numbers have no ISBN-10"
        );
    }

    #[test]
    fn both_forms_cleans_hyphens_and_lowercase_x() {
        assert_eq!(
            both_forms("0-306-40615-2"),
            Some((Some("0306406152".into()), "9780306406157".into()))
        );
        assert_eq!(
            both_forms("978-0-306-40615-7"),
            Some((Some("0306406152".into()), "9780306406157".into()))
        );
        assert_eq!(
            both_forms("0-8044-2957-x"),
            Some((Some("080442957X".into()), "9780804429573".into()))
        );
        assert_eq!(
            both_forms("9791090636071"),
            Some((None, "9791090636071".into()))
        );
        assert_eq!(both_forms("12345"), None);
        assert_eq!(both_forms("0306406153"), None);
    }
}
