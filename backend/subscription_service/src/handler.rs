use crate::errors::subscribe::SubscribeError;
use crate::extractor::{NewsletterPayload, SubscriptionPayload};
use crate::helper;
use crate::helper::{
    confirm_subscription, generate_subscription_token, get_confirmed_subscribers,
    get_subscriber_id_from_token, send_newsletter_email, store_token,
};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Result};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use utils::errors::ErrorPayload;
use utils::state::AppState;
use utils::validation::ValidatedForm;
use validator::HasLen;

#[derive(Deserialize, Debug)]
pub struct TokenQuery {
    token: String,
}

impl Default for TokenQuery {
    fn default() -> Self {
        Self {
            token: "".to_string(),
        }
    }
}

#[tracing::instrument(name = "Starting a subscription",
skip(state, payload), fields(
name= %payload.name,
email= %payload.email
))]
pub async fn subscribe(
    State(state): State<AppState>,
    ValidatedForm(payload): ValidatedForm<SubscriptionPayload>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;
    tracing::info!("Adding a new subscription");
    let mut transaction = pool.begin().await.map_err(SubscribeError::PoolError)?;

    let subscriber_id = helper::insert_subscriber(&mut *transaction, &payload).await?;
    let subscription_token = generate_subscription_token();
    store_token(&mut *transaction, subscriber_id, &subscription_token).await?;
    transaction
        .commit()
        .await
        .map_err(SubscribeError::TransactionCommitError)?;
    helper::send_confirmation_link(&state, payload, subscription_token).await?;
    Ok(Json(json!({"ok": 1})))
}

#[tracing::instrument(name = "Confirmation request", skip(token, pool))]
pub async fn confirm(
    token: Query<TokenQuery>,
    State(pool): State<PgPool>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let id = get_subscriber_id_from_token(&pool, &token.token).await?;
    confirm_subscription(&pool, id).await?;
    Ok("Subscription verified successfully")
}

#[tracing::instrument(name = "Publish newsletter",
skip(state, payload), fields(
title= %payload.title,
content= %payload.content.plain
))]
pub async fn publish_newsletter(
    State(state): State<AppState>,
    ValidatedForm(payload): ValidatedForm<NewsletterPayload>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;

    if payload.content.html.length() == 0 || payload.content.plain.length() == 0 {
        return Err(ErrorPayload::new(
            "any of the content cannot be empty",
            "error".into(),
            400.into(),
        ));
    }
    let confirmed_users = get_confirmed_subscribers(pool).await?;

    let count = send_newsletter_email(&state, payload, confirmed_users).await;
    Ok(format!("Sent email to {} subscribers", count))
}
