use unicode_segmentation::UnicodeSegmentation;
use utils::email::EmailObject;
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

#[derive(serde::Serialize, serde::Deserialize, Validate)]
pub struct NewsletterContent {
    #[validate(length(min = 10, message = "Can not be empty"))]
    pub plain: String,
    #[validate(length(min = 10, message = "Can not be empty"))]
    pub html: String,
}

#[derive(serde::Serialize, serde::Deserialize, Validate)]
pub struct NewsletterPayload {
    #[validate(length(min = 1, message = "Can not be empty"))]
    pub title: String,

    pub content: NewsletterContent,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ConfirmedSubscriber {
    pub name: String,
    pub email: String,
}

impl ConfirmedSubscriber {
    pub fn form_email_object(&self, payload: &NewsletterPayload) -> EmailObject {
        EmailObject {
            sender: "".to_string(),
            to: self.email.clone(),
            subject: payload.title.clone(),
            plain: payload.content.plain.clone(),
            html: payload.content.html.clone(),
        }
    }
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
