use unicode_segmentation::UnicodeSegmentation;
use validator::{Validate, ValidationError};

#[derive(serde::Serialize, serde::Deserialize, Validate)]
pub struct SubscriptionPayload {
    #[validate(
        length(min = 1, message = "Can not be empty"),
        custom(function = "validate_forbidden_chars", message = "Invalid name passed")
    )]
    pub name: String,

    #[validate(email)]
    pub email: String,
}

fn validate_forbidden_chars(value: &str) -> Result<(), ValidationError> {
    let is_too_long = value.graphemes(true).count() > 256;
    let is_empty = value.trim().is_empty();

    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = value.chars().any(|g| forbidden_characters.contains(&g));

    if is_empty || contains_forbidden_characters || is_too_long {
        return Err(ValidationError::new("invalid_values"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a̐".repeat(256);
        assert_ok!(validate_forbidden_chars(&name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(validate_forbidden_chars(&name));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(validate_forbidden_chars(&name));
    }

    #[test]
    fn empty_names_are_rejected() {
        let name = "".to_string();
        assert_err!(validate_forbidden_chars(&name));
    }
}
