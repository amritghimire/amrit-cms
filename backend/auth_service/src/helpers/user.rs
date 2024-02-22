use crate::errors::auth::{
    EmailCheckError, FetchUserError, UserRegistrationError, UsernameCheckError,
};
use crate::extractors::user::User;
use secrecy::ExposeSecret;
use sqlx::PgConnection;

#[tracing::instrument(name = "Checking for existing username")]
pub async fn is_username_used(
    transaction: &mut PgConnection,
    username: &str,
) -> Result<bool, UsernameCheckError> {
    sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM users WHERE normalized_username = $1)
        "#,
        username
    )
    .fetch_one(transaction)
    .await
    .map_err(UsernameCheckError::UsernameCheck)?
    .exists
    .ok_or(UsernameCheckError::Unexpected)
}

#[tracing::instrument(name = "Checking for existing email")]
pub async fn is_email_used(
    transaction: &mut PgConnection,
    email: &str,
) -> Result<bool, EmailCheckError> {
    sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)
        "#,
        email
    )
    .fetch_one(transaction)
    .await
    .map_err(EmailCheckError::EmailCheck)?
    .exists
    .ok_or(EmailCheckError::Unexpected)
}

#[tracing::instrument(name = "Inserting subscriber to database", skip(transaction, user))]
pub async fn insert_user(
    transaction: &mut PgConnection,
    user: &User,
) -> Result<i32, UserRegistrationError> {
    let password_hash = user.password_hash.clone();
    let output = sqlx::query!(
        r#"
        INSERT INTO users (name, email, username, normalized_username, password_hash, created_at, updated_at,
                                  is_active, is_confirmed)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) returning id;
        "#,
        user.name,
        user.email,
        user.username,
        user.normalized_username,
        password_hash.expose_secret(),
        user.created_at,
        user.updated_at,
        user.is_active,
        user.is_confirmed
    ).fetch_one(transaction).await.map_err(UserRegistrationError::InsertUserFailed)?;
    Ok(output.id)
}

#[tracing::instrument(name = "Fetching user form user id", skip(transaction))]
pub async fn fetch_user(
    transaction: &mut PgConnection,
    user_id: i32,
) -> Result<User, FetchUserError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT * from users where id = $1
        "#,
        user_id
    )
    .fetch_one(transaction)
    .await
    .map_err(FetchUserError::UserFetch)?;

    Ok(user)
}
