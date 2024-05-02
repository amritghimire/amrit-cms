use crate::errors::confirmation::ConfirmationError;
use crate::errors::subscribe::SubscribeError;
use crate::extractor::{ConfirmedSubscriber, NewsletterPayload, SubscriptionPayload};
use chrono::Utc;
use email_clients::email::{EmailAddress, EmailObject};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::{PgConnection, PgPool};

use crate::errors::newsletter::NewsletterError;
use utils::state::AppState;
use uuid::Uuid;

#[tracing::instrument(name = "Inserting subscriber to database", skip(transaction, payload))]
pub async fn insert_subscriber(
    transaction: &mut PgConnection,
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
        Utc::now()
    )
    .execute(transaction)
    .await
    .map_err(SubscribeError::InsertSubscribeError)?;

    Ok(subscriber_id)
}

#[tracing::instrument(name = "Sending confirmation link", skip(state, payload))]
pub async fn send_confirmation_link(
    state: &AppState,
    payload: SubscriptionPayload,
    token: String,
) -> Result<(), SubscribeError> {
    let confirmation_link = format!(
        "{}/subscription/confirm?token={}",
        state.settings.application.full_url(),
        token
    );

    let client = state.email_client.to_owned().unwrap();
    let email_object = EmailObject {
        sender: client.get_sender(),
        to: vec![EmailAddress {
            name: payload.name.clone(),
            email: payload.email.clone(),
        }],

        subject: "Welcome to our newsletter!".to_string(),
        plain: format!(
            "Welcome to our newsletter. Please visit {} to confirm your subscription",
            { confirmation_link.clone() }
        ),
        html: format!(
            "<b>Welcome to our newsletter.\
                 Please click <a href='{}' target='_blank'>here </a>\
                  or copy the link below to confirm your subscription.<br>\
                 \
                 {}
                 ",
            { confirmation_link.clone() },
            { confirmation_link }
        ),
    };
    let res = client.send_emails(email_object).await;
    res.map_err(SubscribeError::ConfirmationEmailError)?;
    Ok(())
}

#[tracing::instrument()]
#[tracing::instrument(name = "Store token in database", skip(transaction))]
pub async fn store_token(
    transaction: &mut PgConnection,
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

#[tracing::instrument(name = "get the list of confirmed subscriptions", skip(pool))]
pub async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<ConfirmedSubscriber>, NewsletterError> {
    let result = sqlx::query_as!(
        ConfirmedSubscriber,
        r#"SELECT email, name from subscriptions where status = 'confirmed'"#
    )
    .fetch_all(pool)
    .await
    .map_err(NewsletterError::ConfirmedSubscribersError)?;
    Ok(result)
}

#[tracing::instrument(
    name = "send the confirmation email",
    skip(state, payload, confirmed_users)
)]
pub async fn send_newsletter_email(
    state: &AppState,
    payload: NewsletterPayload,
    confirmed_users: Vec<ConfirmedSubscriber>,
) -> u64 {
    let count = confirmed_users.len();
    let email_object = ConfirmedSubscriber::form_email_object(confirmed_users, &payload);
    let client = state.email_client.to_owned();
    if client.unwrap().send_emails(email_object).await.is_ok() {
        count as u64
    } else {
        0u64
    }
}

pub fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
