use crate::extractor::SubscriptionPayload;
use crate::helper;
use crate::helper::{
    confirm_subscription, generate_subscription_token, get_subscriber_id_from_token, store_token,
};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use sqlx::PgPool;
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
) -> Response {
    let pool = &state.connection;
    tracing::info!("Adding a new subscription");
    let subscriber_id = match helper::insert_subscriber(pool, &payload).await {
        Ok(subscriber_id) => subscriber_id,
        Err(err) => {
            tracing::error!("Error occurred {:?}", err);
            return helper::handle_database_error(err).into_response();
        }
    };
    let subscription_token = generate_subscription_token();
    if let Err(err) = store_token(pool, subscriber_id, &subscription_token).await {
        return helper::handle_database_error(err).into_response();
    }
    helper::send_confirmation_link(&state, payload, subscription_token)
}

#[tracing::instrument(name = "Confirmation request", skip(token, pool))]
pub async fn confirm(token: Query<TokenQuery>, State(pool): State<PgPool>) -> impl IntoResponse {
    let id = match get_subscriber_id_from_token(&pool, &token.token).await {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error occurred when trying to verify token",
            )
        }
    };
    match id {
        None => (StatusCode::UNAUTHORIZED, "Cannot find the token"),
        Some(subscriber_id) => {
            if confirm_subscription(&pool, subscriber_id).await.is_err() {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Cannot confirm the subscription",
                );
            }
            (StatusCode::OK, "Subscription verified successfully")
        }
    }
}
