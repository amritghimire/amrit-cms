use validator::{Validate, ValidationError};
use unicode_segmentation::UnicodeSegmentation;


#[derive(serde::Serialize, serde::Deserialize, Validate)]
pub struct SubscriptionPayload {
    #[validate(length(min = 1, message = "Can not be empty"), custom = "validate_forbidden_chars")]
    pub name: String,

    #[validate(email)]
    pub email: String,
}

fn validate_forbidden_chars(value: &str) -> Result<(), ValidationError> {
    let is_too_long = value.graphemes(true).count() > 256;

    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = value.chars().any(
        |g| forbidden_characters.contains(&g)
    );

    if contains_forbidden_characters || is_too_long {
        return Err(ValidationError::new("invalid_values"));
    }

    Ok(())
}
