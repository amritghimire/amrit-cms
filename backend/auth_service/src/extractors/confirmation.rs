use chrono::{DateTime, Duration, Utc};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum ConfirmationActionType {
    UserVerification,
    PasswordReset,
    Invalid,
}

impl From<String> for ConfirmationActionType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "userverification" => ConfirmationActionType::UserVerification,
            "passwordreset" => ConfirmationActionType::PasswordReset,
            _ => ConfirmationActionType::Invalid,
        }
    }
}

impl From<ConfirmationActionType> for String {
    fn from(value: ConfirmationActionType) -> Self {
        match value {
            ConfirmationActionType::UserVerification => "userverification".to_string(),
            ConfirmationActionType::Invalid => "invalid".to_string(),
            ConfirmationActionType::PasswordReset => "passwordreset".to_string(),
        }
    }
}

#[derive(Debug, FromRow, Deserialize)]
pub struct Confirmation {
    pub confirmation_id: Uuid,
    pub details: Option<Value>,
    pub verifier_hash: String,
    pub user_id: i32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub action_type: ConfirmationActionType,
}

impl Confirmation {
    pub fn new(
        user_id: i32,
        action_type: ConfirmationActionType,
        details: Value,
    ) -> (Self, String) {
        let confirmation_id = Uuid::new_v4();
        let verifier = Uuid::new_v4();

        let mut hasher = Sha256::new();

        // Write input message
        hasher.update(verifier.to_string().as_bytes());

        // Read hash digest and consume hasher
        let verifier_hash = format!("{:x}", hasher.finalize());
        let token = format!("{}.{}", confirmation_id, verifier);

        (
            Self {
                confirmation_id,
                details: details.into(),
                verifier_hash,
                user_id,
                created_at: Utc::now(),
                expires_at: Utc::now() + Duration::try_hours(24).unwrap(),
                action_type,
            },
            token,
        )
    }

    pub fn subject(&self) -> String {
        match self.action_type {
            ConfirmationActionType::UserVerification => {
                "Please verify your account to proceed".to_string()
            }
            ConfirmationActionType::PasswordReset => {
                "Please proceed to reset the password".to_string()
            }
            ConfirmationActionType::Invalid => {
                unreachable!()
            }
        }
    }

    pub fn confirmation_url(&self, full_url: &str, token: Secret<String>) -> String {
        match self.action_type {
            ConfirmationActionType::PasswordReset => {
                format!("{}/auth/reset-password/{}", full_url, token.expose_secret())
            }
            _ => {
                format!("{}/auth/confirm/{}", full_url, token.expose_secret())
            }
        }
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
            ConfirmationActionType::PasswordReset => {
                (
                    format!(
                        "You have requested to reset your password. Please visit {} to reset your password",
                        { confirmation_link }
                    ),
                    format!(
                        "<b>You have requested to reset your password.\
                     Please click <a href='{}' target='_blank'>here </a>\
                      or copy the link below to reset your password.<br>\
                     \
                     {}
                     ",
                        { confirmation_link },
                        { confirmation_link }
                    ),
                )
            }
            ConfirmationActionType::Invalid => {
                unreachable!()
            }
        }
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}
