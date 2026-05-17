use once_cell::sync::Lazy;
use regex::Regex;

pub static ALPHANUMERIC_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9]+$").unwrap());
pub static WORLD_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z_]+$").unwrap());
pub static NAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9_-]+$").unwrap());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alphanumeric_accepts_letters_and_digits_only() {
        assert!(ALPHANUMERIC_REGEX.is_match("abc123"));
        assert!(ALPHANUMERIC_REGEX.is_match("ABC"));
        assert!(!ALPHANUMERIC_REGEX.is_match("abc_123"));
        assert!(!ALPHANUMERIC_REGEX.is_match("abc 123"));
        // Anchored: an empty string and embedded newlines are rejected.
        assert!(!ALPHANUMERIC_REGEX.is_match(""));
        assert!(!ALPHANUMERIC_REGEX.is_match("abc\n123"));
    }

    #[test]
    fn world_accepts_letters_and_underscore_only() {
        assert!(WORLD_REGEX.is_match("hello_world"));
        assert!(!WORLD_REGEX.is_match("hello1"));
        assert!(!WORLD_REGEX.is_match("hello-world"));
        assert!(!WORLD_REGEX.is_match(""));
    }

    #[test]
    fn name_accepts_word_chars_and_dash() {
        assert!(NAME_REGEX.is_match("front-door_01"));
        assert!(NAME_REGEX.is_match("Camera2"));
        assert!(!NAME_REGEX.is_match("bad name"));
        assert!(!NAME_REGEX.is_match("name!"));
        assert!(!NAME_REGEX.is_match(""));
    }
}
