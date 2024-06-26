use crate::errors::user::UserError;
use crate::extractors::user::User;
use once_cell::sync::Lazy;
use regex::Regex;
use rustrict::CensorStr;
use secrecy::Secret;
use serde::Deserialize;
use std::convert::TryFrom;
use validator::{Validate, ValidationError};

static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[0-9A-Za-z_.]+$").unwrap());

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterPayload {
    #[validate(
        length(min = 3, message = "Username cannot be empty", max = 255),
        non_control_character,
        regex = "USERNAME_REGEX",
        custom(
            function = "validate_profanity",
            message = "Invalid words in username."
        )
    )]
    pub username: String,
    #[validate(length(min = 8, max = 72, message = "Password must contains 8-72 characters"))]
    pub password: String,
    #[validate(length(min = 1, max = 255, message = "Email cannot be empty"), email)]
    pub email: String,
    #[validate(must_match(
        other = "password",
        message = "password and confirm password must match"
    ))]
    pub confirm_password: String,
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}

impl TryFrom<RegisterPayload> for User {
    type Error = UserError;

    fn try_from(value: RegisterPayload) -> Result<Self, Self::Error> {
        let normalized_username = User::normalize_username(&value.username)?;
        let password_hash = Secret::from(User::hash_password(&value.password)?);
        Ok(User {
            name: value.name,
            email: value.email,
            normalized_username,
            username: value.username,
            password_hash,
            ..Default::default()
        })
    }
}

fn validate_profanity(username: &str) -> Result<(), ValidationError> {
    if username.is_inappropriate() {
        return Err(ValidationError::new("username_not_valid"));
    }

    Ok(())
}
