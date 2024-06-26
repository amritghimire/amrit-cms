use crate::errors::auth::{
    EmailCheckError, FetchUserError, UserRegistrationError, UsernameCheckError,
};
use crate::errors::user::UserError;
use crate::extractors::user::User;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgConnection;

#[tracing::instrument(name = "Checking for existing username")]
pub async fn fetch_by_username(
    transaction: &mut PgConnection,
    username: &str,
) -> Result<Option<User>, UsernameCheckError> {
    let user = sqlx::query_as!(
        User,
        r#"
        select * from users where normalized_username = $1
        "#,
        username
    )
    .fetch_optional(transaction)
    .await
    .map_err(UsernameCheckError::UsernameCheck)?;
    Ok(user)
}

#[tracing::instrument(name = "Checking for existing email")]
pub async fn fetch_by_email(
    transaction: &mut PgConnection,
    email: &str,
) -> Result<Option<User>, EmailCheckError> {
    let user = sqlx::query_as!(
        User,
        r#"
        select * from users where email = $1
        "#,
        email.to_lowercase()
    )
    .fetch_optional(transaction)
    .await
    .map_err(EmailCheckError::EmailCheck)?;
    Ok(user)
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
        user.email.to_lowercase(),
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

pub async fn update_password(
    transaction: &mut PgConnection,
    user_id: i32,
    password: Secret<String>,
) -> Result<u64, UserError> {
    let password_hash = User::hash_password(password.expose_secret())?;
    let result = sqlx::query!(
        "UPDATE users SET password_hash = $1 WHERE id = $2",
        password_hash.to_string(),
        user_id
    )
    .execute(transaction)
    .await
    .map_err(UserError::SessionError)?;

    Ok(result.rows_affected())
}
