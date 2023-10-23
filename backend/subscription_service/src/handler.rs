use crate::errors::subscribe::SubscribeError;
use crate::extractor::SubscriptionPayload;
use crate::helper;
use crate::helper::{
    confirm_subscription, generate_subscription_token, get_subscriber_id_from_token, store_token,
};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Result};
use serde::Deserialize;
use sqlx::PgPool;
use utils::errors::ErrorPayload;
use utils::state::AppState;
use utils::validation::ValidatedForm;

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

#[tracing::instrument(name="Subscription request",
skip(state, payload), fields(
email= %payload.email,
name= %payload.name
))]
pub async fn subscribe(
    State(state): State<AppState>,
    ValidatedForm(payload): ValidatedForm<SubscriptionPayload>,
) -> Result<impl IntoResponse, ErrorPayload> {
    let pool = &state.connection;
    tracing::info!("Adding a new subscription");
    let mut transaction = pool.begin().await.map_err(SubscribeError::DatabaseError)?;

    let subscriber_id = helper::insert_subscriber(&mut transaction, &payload).await?;
    let subscription_token = generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscription_token).await?;
    transaction
        .commit()
        .await
        .map_err(SubscribeError::DatabaseError)?;
    let res = helper::send_confirmation_link(&state, payload, subscription_token)?;
    Ok(res)
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
