use axum::http::StatusCode;
use axum::response::Response;
use axum::{http, Router};
use fake::faker::internet::en::Password;
use fake::faker::internet::en::SafeEmail;
use fake::faker::internet::en::Username;
use fake::faker::name::en::Name;
use fake::{Dummy, Fake, Faker};
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::mpsc;

use auth_service::router::create_router;
use tower::util::ServiceExt;

use auth_service::extractor::{ConfirmationActionType, User};
use sqlx::PgPool;
use url::Url;
use utils::configuration::{RunMode, Settings};
use utils::email::get_link;
use utils::test;

static STRONG_PASSWORD: &str = "r0sebudmaelstrom11/20/91aaaa";

#[derive(Debug, Dummy, Serialize, Clone)]
pub struct RegistrationPayload {
    #[dummy(faker = "Username()")]
    pub username: String,
    #[dummy(faker = "Password(8..72)")]
    pub password: String,
    #[dummy(faker = "SafeEmail()")]
    pub email: String,
    pub confirm_password: String,
    #[dummy(faker = "Name()")]
    pub name: String,
}

#[sqlx::test]
async fn registration_200_valid_form_data(pool: PgPool) {
    let (tx, _rx) = mpsc::sync_channel(5);
    let state = test::test_state_for_email(pool, tx);

    let app = create_router().with_state(state);
    let payload: RegistrationPayload = Faker.fake();
    let data = serde_json::to_value(payload).unwrap();
    let data = replace_password(&data, STRONG_PASSWORD);
    let data = replace_key(&data, "username", "safe_username");
    let response = send_request(&app, &data).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test]
async fn registration_valid_form_data_is_inserted(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");
    let (tx, _rx) = mpsc::sync_channel(5);
    let state = test::test_state_for_email(pool, tx);

    let app = create_router().with_state(state);
    let payload: RegistrationPayload = Faker.fake();
    let data = serde_json::to_value(payload.clone()).unwrap();
    let data = replace_password(&data, STRONG_PASSWORD);
    let data = replace_key(&data, "username", "safe_username");
    let response = send_request(&app, &data).await;

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "The request didn't get 200 response for {:?}",
        payload
    );

    let saved = sqlx::query_as!(
        User,
        "SELECT id, name, email, username, normalized_username, is_active, is_confirmed, created_at, updated_at, password_hash from users"
    )
        .fetch_one(&mut *conn)
        .await
        .expect("Unable to fetch the table");

    assert_eq!(saved.email, payload.email);
    assert_eq!(saved.name, payload.name);
    assert_eq!(saved.username, "safe_username");
    assert_eq!(
        saved.normalized_username,
        User::normalize_username(&saved.username).unwrap()
    );
    assert!(saved.is_active);
    assert!(!saved.is_confirmed);
    assert!(saved.check_password(STRONG_PASSWORD));
    assert!(!saved.check_password("wrong"));

    let confirmation = sqlx::query!("SELECT * from confirmations")
        .fetch_one(&mut *conn)
        .await
        .expect("unable to fetch the confirmation");

    assert_eq!(
        confirmation.action_type,
        String::from(ConfirmationActionType::UserVerification)
    );
    assert_eq!(confirmation.details, Some(json!({"email": payload.email})));
    assert_eq!(confirmation.user_id, saved.id);
}

#[sqlx::test]
async fn registration_sends_confirmation_email(pool: PgPool) {
    let (tx, rx) = mpsc::sync_channel(5);
    let settings = Settings::get_config(RunMode::Test).expect("Unable to fetch test config");
    let state = test::test_state_for_email(pool, tx);
    let app = create_router().with_state(state);
    let payload: RegistrationPayload = Faker.fake();
    let data = serde_json::to_value(payload.clone()).unwrap();
    let data = replace_password(&data, STRONG_PASSWORD);
    let data = replace_key(&data, "username", "safe_username");
    send_request(&app, &data).await;

    let email_object = rx
        .try_recv()
        .expect("Email not sent during the subscription");
    assert_eq!(email_object.sender, settings.email.sender);
    assert_eq!(email_object.to, payload.email);
    assert_eq!(
        email_object.subject,
        "Please verify your account to proceed"
    );
    let raw_link = get_link(&email_object.plain);
    let confirmation_link = Url::parse(&raw_link).unwrap();
    let application_link = Url::parse(&settings.application.full_url()).unwrap();
    assert_eq!(
        confirmation_link.host_str().unwrap(),
        application_link.host_str().unwrap()
    )
}

#[sqlx::test]
async fn registration_already_exists(pool: PgPool) {
    let (tx, _rx) = mpsc::sync_channel(5);
    let state = test::test_state_for_email(pool, tx);

    let app = create_router().with_state(state);
    let payload: RegistrationPayload = Faker.fake();
    let data = serde_json::to_value(payload.clone()).unwrap();
    let data = replace_password(&data, STRONG_PASSWORD);
    let data = replace_key(&data, "username", "safe_username");
    send_request(&app, &data).await;

    // Send with same username
    let new_email: String = SafeEmail().fake();
    let data = replace_key(&data, "email", &new_email);
    let response = send_request(&app, &data).await;
    test::assert_response(response, StatusCode::BAD_REQUEST, "Username not available").await;

    // Change the username
    let new_username: String = Username().fake();
    let data = replace_key(&data, "username", &new_username);
    let data = replace_key(&data, "email", &payload.email);
    let response = send_request(&app, &data).await;
    test::assert_response(
        response,
        StatusCode::BAD_REQUEST,
        "Email already registered",
    )
    .await;
}

#[sqlx::test]
async fn register_returns_a_400_for_invalid_form_data(pool: PgPool) {
    let (tx, _rx) = mpsc::sync_channel(5);
    let state = test::test_state_for_email(pool, tx);
    let app = create_router().with_state(state);

    let payload: RegistrationPayload = Faker.fake();
    let data = serde_json::to_value(payload).unwrap();
    let data = replace_password(&data, STRONG_PASSWORD);

    let test_cases = vec![
        (json!({}), "empty payload"),
        (filter_key(&data, "name"), "missing the name"),
        (filter_key(&data, "email"), "missing the email"),
        (filter_key(&data, "username"), "missing the username"),
        (filter_key(&data, "password"), "missing the password"),
        (
            filter_key(&data, "confirm_password"),
            "missing the confirm_password",
        ),
        (replace_key(&data, "username", ""), "empty username"),
        (
            replace_key(&data, "username", "नेपाली"),
            "non control character in username",
        ),
        (
            replace_key(&data, "username", "crap"),
            "profanity in username",
        ),
        (replace_key(&data, "password", "short"), "short password"),
        (
            replace_key(&data, "password", &"*".repeat(74)),
            "long password",
        ),
        (replace_key(&data, "email", ""), "empty email"),
        (
            replace_key(&data, "confirm_password", "different"),
            "different password and confirm password",
        ),
        (replace_password(&data, "1234567890"), "weak password"),
    ];

    for (payload, error_message) in test_cases {
        let response = app
            .clone()
            .oneshot(test::build_request(
                "/register",
                http::Method::POST,
                &payload,
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "The request didn't throw 400 request for the case when {}",
            error_message
        );
    }
}

async fn send_request(app: &Router, data: &Value) -> Response {
    let request = test::build_request("/register", http::Method::POST, data);
    app.clone().oneshot(request).await.unwrap()
}

fn filter_key(source: &Value, key: &str) -> Value {
    let mut output = source.clone();
    output
        .as_object_mut()
        .expect("Cannot unwrap as mutable object")
        .remove(key);
    output
}

fn replace_key(source: &Value, key: &str, value: &str) -> Value {
    let mut output = source.clone();
    output[key] = value.into();
    output
}

fn replace_password(source: &Value, password: &str) -> Value {
    let mut output = source.clone();
    output["password"] = password.into();
    output["confirm_password"] = password.into();
    output
}
