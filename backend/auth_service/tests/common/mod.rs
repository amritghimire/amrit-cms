use auth_service::extractors::confirmation::{Confirmation, ConfirmationActionType};
use auth_service::extractors::user::User;
use auth_service::helpers::confirmation::add_confirmation;
use auth_service::helpers::sessions::create_new_session;
use auth_service::helpers::user::insert_user;
use auth_service::payload::RegisterPayload;
use auth_service::router::create_router;
use axum::Router;
use email_clients::email::EmailObject;
use fake::faker::internet::en::{SafeEmail, Username};
use fake::faker::name::en::Name;
use fake::Fake;
use serde_json::json;
use sqlx::{PgConnection, PgPool};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use utils::configuration::{RunMode, Settings};
use utils::test;

pub static STRONG_PASSWORD: &str = "r0sebudmaelstrom11/20/91aaaa";

#[allow(dead_code)]
pub fn setup_app(pool: PgPool) -> (Receiver<EmailObject>, Settings, Router) {
    let (tx, rx) = mpsc::sync_channel(5);
    let settings = Settings::get_config(RunMode::Test).expect("Unable to fetch test config");
    let state = test::test_state_for_email(pool, tx);
    let app = create_router().with_state(state);
    (rx, settings, app)
}

#[allow(dead_code)]
pub async fn confirmation_fixture(transaction: &mut PgConnection) -> (Confirmation, String) {
    let user = user_fixture(transaction).await;
    let (confirmation, token) = Confirmation::new(
        user.id,
        ConfirmationActionType::UserVerification,
        json!({"email": user.email}),
    );
    add_confirmation(transaction, &confirmation)
        .await
        .expect("Cannot add confirmation");
    (confirmation, token)
}

#[allow(dead_code)]
pub async fn user_fixture(transaction: &mut PgConnection) -> User {
    let user_payload = RegisterPayload {
        username: Username().fake(),
        password: STRONG_PASSWORD.to_string(),
        email: SafeEmail().fake(),
        confirm_password: STRONG_PASSWORD.to_string(),
        name: Name().fake(),
    };

    let mut user = User::try_from(user_payload).expect("Cannot form new user");
    let id = insert_user(transaction, &user)
        .await
        .expect("Cannot insert user");
    user.id = id;
    user
}

#[allow(dead_code)]
pub async fn session_fixture(transaction: &mut PgConnection, user_id: i32) -> String {
    create_new_session(transaction, user_id, json!({}))
        .await
        .unwrap()
}
