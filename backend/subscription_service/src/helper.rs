use crate::errors::confirmation::ConfirmationError;
use crate::errors::subscribe::SubscribeError;
use crate::extractor::SubscriptionPayload;
use axum::response::IntoResponse;
use axum::Json;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};
use utils::state::AppState;
use uuid::Uuid;

pub fn get_link(s: &str) -> String {
    let links: Vec<_> = linkify::LinkFinder::new()
        .links(s)
        .filter(|l| *l.kind() == linkify::LinkKind::Url)
        .collect();
    assert_eq!(links.len(), 1);
    links[0].as_str().to_owned()
}

#[tracing::instrument(name = "Inserting subscriber to database", skip(transaction, payload))]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    payload: &SubscriptionPayload,
) -> Result<Uuid, SubscribeError> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        subscriber_id,
        payload.email,
        payload.name,
        time::OffsetDateTime::now_utc()
    )
    .execute(transaction)
    .await
    .map_err(SubscribeError::InsertSubscribeError)?;

    Ok(subscriber_id)
}

#[tracing::instrument(name = "Sending confirmation link", skip(state, payload))]
pub fn send_confirmation_link(
    state: &AppState,
    payload: SubscriptionPayload,
    token: String,
) -> Result<impl IntoResponse, SubscribeError> {
    let confirmation_link = format!(
        "{}/subscription/confirm?token={}",
        state.settings.application.full_url(),
        token
    );
    state
        .email_client
        .to_owned()
        .unwrap()
        .send_email(
            payload.email,
            "Welcome to our newsletter!".to_string(),
            format!(
                "Welcome to our newsletter. Please visit {} to confirm your subscription",
                { confirmation_link }
            ),
        )
        .map_err(SubscribeError::ConfirmationEmailError)?;
    Ok(Json(json!({"ok": 1})))
}

#[tracing::instrument(name = "Store token in database", skip(transaction))]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), SubscribeError> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscription_id)
        VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(SubscribeError::StoreTokenError)?;

    Ok(())
}

#[tracing::instrument(name = "Store token in database", skip(pool))]
pub async fn confirm_subscription(
    pool: &PgPool,
    subscriber_id: Uuid,
) -> Result<(), ConfirmationError> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1
        "#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(ConfirmationError::ConfirmationFailedError)?;
    Ok(())
}

#[tracing::instrument(name = "Store token in database", skip(pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Uuid, ConfirmationError> {
    let result = sqlx::query!(
        r#"SELECT subscription_id FROM subscription_tokens WHERE subscription_token = $1
        "#,
        subscription_token,
    )
    .fetch_optional(pool)
    .await
    .map_err(ConfirmationError::GetSubscriberError)?;
    let v = result
        .map(|r| r.subscription_id)
        .ok_or(ConfirmationError::SubscriptionNotFoundError {})?;
    Ok(v)
}

pub fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
