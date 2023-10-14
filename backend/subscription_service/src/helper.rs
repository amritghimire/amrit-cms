use crate::extractor::SubscriptionPayload;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::json;
use sqlx::{Error, PgPool, Postgres, Transaction};
use utils::errors::ErrorPayload;
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
) -> Result<Uuid, Error> {
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
    .await?;

    Ok(subscriber_id)
}

pub fn handle_database_error(err: Error) -> ErrorPayload {
    if let Some(e) = err.into_database_error() {
        let message: &str = e.message();
        if message.contains("subscriptions_email_key") && message.contains("duplicate key value") {
            tracing::info!("Email already exists");
            return ErrorPayload::new("Email already subscribed", Some("error"), Some(400));
        }
    }

    ErrorPayload::new("Unable to add to subscription", Some("error"), Some(500))
}

#[tracing::instrument(name = "Sending confirmation link", skip(state, payload))]
pub fn send_confirmation_link(
    state: &AppState,
    payload: SubscriptionPayload,
    token: String,
) -> Response {
    let confirmation_link = format!(
        "{}/subscription/confirm?token={}",
        state.settings.application.full_url(),
        token
    );
    match state.email_client.to_owned().unwrap().send_email(
        payload.email,
        "Welcome to our newsletter!".to_string(),
        format!(
            "Welcome to our newsletter. Please visit {} to confirm your subscription",
            { confirmation_link }
        ),
    ) {
        Ok(_) => Json(json!({"ok": 1})).into_response(),
        Err(err) => {
            tracing::error!("Error occurred {:?}", err);
            ErrorPayload::new("Unable to send email", Some("error"), Some(400)).into_response()
        }
    }
}

#[tracing::instrument(name = "Store token in database", skip(transaction))]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscription_id)
        VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(name = "Store token in database", skip(pool))]
pub async fn confirm_subscription(pool: &PgPool, subscriber_id: Uuid) -> Result<(), Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1
        "#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(name = "Store token in database", skip(pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, Error> {
    let result = sqlx::query!(
        r#"SELECT subscription_id FROM subscription_tokens WHERE subscription_token = $1
        "#,
        subscription_token,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;
    Ok(result.map(|r| r.subscription_id))
}

pub fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
