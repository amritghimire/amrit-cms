mod common;

use axum::http::header::{AUTHORIZATION, SET_COOKIE};
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
use std::time::Duration;
use tower::util::ServiceExt;

use auth_service::extractors::confirmation::ConfirmationActionType;
use auth_service::extractors::session::SESSION_TOKEN_COOKIE;
use auth_service::extractors::user::User;
use sqlx::PgPool;
use url::Url;
use utils::email::get_link;
use utils::test;

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
    let (_rx, _, app) = common::setup_app(pool);
    let (_, data) = registration_payload();

    let response = send_request(&app, &data).await;

    let authorization_header: &str = response
        .headers()
        .get(AUTHORIZATION)
        .unwrap()
        .to_str()
        .unwrap();
    let cookie_header: &str = response
        .headers()
        .get(SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(!authorization_header.is_empty());
    assert!(cookie_header.contains(SESSION_TOKEN_COOKIE))
}

#[sqlx::test]
async fn registration_valid_form_data_is_inserted(pool: PgPool) {
    let mut conn = pool.acquire().await.expect("Unable to acquire connection");
    let (_rx, _, app) = common::setup_app(pool);
    let (payload, data) = registration_payload();

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
    assert!(saved.check_password(common::STRONG_PASSWORD));
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
    let (email_rx, task_rx, settings, app) = common::setup_app_with_task_thread(pool);

    let (payload, data) = registration_payload();

    send_request(&app, &data).await;

    let task = task_rx.try_recv().expect("Task not thrown out.");
    task.handle.await.expect("Join error, task panicked");

    let email_object = email_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("Email not sent during the subscription");
    assert_eq!(email_object.sender.to_string(), "test@example.com");
    assert_eq!(email_object.to[0].email, payload.email);
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
    let (_rx, _, app) = common::setup_app(pool);
    let (payload, data) = registration_payload();

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
    let (_, _, app) = common::setup_app(pool);
    let (_, data) = registration_payload();

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

fn registration_payload() -> (RegistrationPayload, Value) {
    let payload: RegistrationPayload = Faker.fake();
    let data = serde_json::to_value(payload.clone()).unwrap();
    let data = replace_password(&data, common::STRONG_PASSWORD);
    let data = replace_key(&data, "username", "safe_username");
    (payload, data)
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
