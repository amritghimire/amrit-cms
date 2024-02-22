use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use uuid::Uuid;

pub static SESSION_TOKEN_COOKIE: &str = "session_token";

#[derive(Debug, FromRow, Deserialize)]
pub struct UserSession {
    pub identifier: Uuid,
    pub verifier_hash: String,
    pub user_id: i32,
    pub extra_info: Value,
    pub expiration_date: DateTime<Utc>,
}

impl UserSession {
    pub fn new(user_id: i32, extra_info: Value) -> (Self, String) {
        let identifier = Uuid::new_v4();
        let verifier = Uuid::new_v4();

        let mut hasher = Sha256::new();

        // Write input message
        hasher.update(verifier.to_string().as_bytes());

        // Read hash digest and consume hasher
        let verifier_hash = format!("{:x}", hasher.finalize());
        let token = format!("{}.{}", identifier, verifier);

        (
            Self {
                identifier,
                verifier_hash,
                user_id,
                extra_info,
                expiration_date: Utc::now() + Duration::days(30),
            },
            token,
        )
    }

    pub fn is_expired(&self) -> bool {
        self.expiration_date < Utc::now()
    }

    pub fn should_extend(&mut self) -> bool {
        if self.expiration_date < Utc::now() + Duration::days(25) {
            self.expiration_date = Utc::now() + Duration::days(30);
            return true;
        }
        false
    }
}
