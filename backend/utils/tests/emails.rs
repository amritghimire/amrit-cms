use sqlx::PgPool;
use utils::state::AppState;

#[sqlx::test]
async fn send_email_saved_in_memory(pool: PgPool) {
    let recipient_mail = "mail@example.com".to_string();
    let mail_subject = "New subject".to_string();
    let mail_body = "Body of email".to_string();

    let state = AppState::test_state(pool, None);
    let mut email_client = state.email_client.unwrap();

    email_client
        .send_email(
            recipient_mail.clone(),
            mail_subject.clone(),
            mail_body.clone(),
        )
        .expect("Unable to send email");

    let emails = email_client.get_mails();

    assert_eq!(emails.len(), 1);
    assert_eq!(emails[0].sender, state.settings.email.sender);
    assert_eq!(emails[0].to, recipient_mail);
    assert_eq!(emails[0].subject, mail_subject);
    assert_eq!(emails[0].body, mail_body);
}
