use crate::errors::user::UserError;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chrono::DateTime;
use chrono::Utc;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{types::Json, FromRow};
use uuid::Uuid;
use zxcvbn::zxcvbn;

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub normalized_username: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: Secret<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
    pub is_confirmed: bool,
}

impl Default for User {
    fn default() -> Self {
        User {
            id: 0,
            name: "".to_string(),
            email: "".to_string(),
            normalized_username: "".to_string(),
            username: "".to_string(),
            password_hash: Secret::from("".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: true,
            is_confirmed: false,
        }
    }
}

impl User {
    /// # Examples
    ///
    /// ```
    /// use auth_service::errors::user::UserError;
    /// use auth_service::extractor::User;
    ///
    /// assert_eq!(User::normalize_username("Apple1").unwrap(), ("applel".to_string()));
    /// assert_eq!(User::normalize_username("0123456789").unwrap(), ("olzeasbtbg".to_string()));
    /// assert_eq!(User::normalize_username("bob").unwrap(), ("bob".to_string()));
    /// assert_eq!(User::normalize_username("5neak").unwrap(), ("sneak".to_string()));
    /// assert!(User::normalize_username("!@").is_err());
    /// assert!(User::normalize_username("").is_err());
    /// ```
    pub fn normalize_username(username: &str) -> Result<String, UserError> {
        if !(username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_'))
        {
            return Err(UserError::NormalizeUserFailed(
                "Invalid characters in username".to_string(),
            ));
        }
        let normalized_username: String = username
            .chars()
            .map(|g| match g {
                '0' => 'o',
                '1' => 'l',
                '2' => 'z',
                '3' => 'e',
                '4' => 'a',
                '5' => 's',
                '6' => 'b',
                '7' => 't',
                '8' => 'b',
                '9' => 'g',
                _ => g,
            })
            .collect();

        if normalized_username.is_empty() {
            return Err(UserError::NormalizeUserFailed(
                "Username cannot be empty".to_string(),
            ));
        }

        Ok(normalized_username.to_lowercase())
    }

    pub fn hash_password(password: &str) -> Result<String, UserError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let result = argon2
            .hash_password(password.as_ref(), &salt)
            .map_err(|_| UserError::PasswordHashError("Failed to hash password".to_string()))?;

        Ok(result.to_string())
    }

    pub fn check_acceptable_password(password: &str, inputs: &[&str]) -> Result<(), UserError> {
        let estimator = zxcvbn(password, inputs).map_err(UserError::PasswordCheckFailed)?;
        if estimator.score() < 3 {
            return Err(UserError::WeakPassword);
        }
        Ok(())
    }
}

impl User {
    pub fn check_password(&self, password: &str) -> bool {
        let parsed_hash = PasswordHash::new(self.password_hash.expose_secret());
        if let Ok(parsed_hash) = parsed_hash {
            return Argon2::default()
                .verify_password(password.as_ref(), &parsed_hash)
                .is_ok();
        }
        false
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConfirmationActionType {
    UserVerification,
}

#[derive(Debug, FromRow, Deserialize)]
pub struct Confirmation {
    pub confirmation_id: Uuid,
    pub details: Json<Value>,
    pub verifier_hash: String,
    pub user_id: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub action_type: ConfirmationActionType,
    #[sqlx(skip)]
    pub token: Option<Secret<String>>,
}

impl Confirmation {
    pub fn new(user_id: i32, action_type: ConfirmationActionType, details: Value) -> Self {
        let confirmation_id = Uuid::new_v4();
        let verifier = Uuid::new_v4();

        let mut hasher = Sha256::new();

        // Write input message
        hasher.update(verifier.to_string().as_bytes());

        // Read hash digest and consume hasher
        let verifier_hash = format!("{:x}", hasher.finalize());
        let token = format!("{}.{}", confirmation_id, verifier);

        Self {
            confirmation_id,
            details: details.into(),
            verifier_hash,
            user_id,
            created_at: Utc::now(),
            expires_at: Utc::now(),
            action_type,
            token: Some(Secret::new(token.to_string())),
        }
    }

    pub fn confirmation_url(&self, full_url: &str) -> Result<String, UserError> {
        if let Some(token) = &self.token {
            let confirmation_link = format!(
                "{}/subscription/confirm?token={}",
                full_url,
                token.expose_secret()
            );
            return Ok(confirmation_link);
        }
        Err(UserError::ConfirmationTokenMissing)
    }

    pub fn email_contents(&self, confirmation_link: &str) -> (String, String) {
        match self.action_type {
            ConfirmationActionType::UserVerification => (
                format!(
                    "Welcome to our newsletter. Please visit {} to confirm your account",
                    { confirmation_link }
                ),
                format!(
                    "<b>Welcome to our newsletter.\
                 Please click <a href='{}' target='_blank'>here </a>\
                  or copy the link below to confirm your subscription.<br>\
                 \
                 {}
                 ",
                    { confirmation_link },
                    { confirmation_link }
                ),
            ),
        }
    }
}
