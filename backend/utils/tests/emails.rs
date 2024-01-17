use std::sync::mpsc;
use utils::configuration::{RunMode, Settings};
use utils::email::{send_email, EmailClient, MessagePassingClient};

#[sqlx::test]
async fn send_email_saved_in_memory() {
    let recipient_mail = "mail@example.com".to_string();
    let mail_subject = "New subject".to_string();
    let mail_body = "Body of email".to_string();
    let mail_html = "Body of email in <b>HTML</b>".to_string();

    let (tx, rx) = mpsc::sync_channel(2);
    let settings = Settings::get_config(RunMode::Test).expect("Unable to fetch test config");

    let email_client = EmailClient::MessagePassingClient(MessagePassingClient::with_tx(
        settings.email.clone(),
        tx,
    ));

    send_email(
        email_client,
        recipient_mail.clone(),
        mail_subject.clone(),
        mail_body.clone(),
        mail_html,
    )
    .await
    .expect("Unable to send email");

    let email = rx.recv().unwrap();

    assert_eq!(email.sender, settings.email.sender);
    assert_eq!(email.to, recipient_mail);
    assert_eq!(email.subject, mail_subject);
    assert_eq!(email.plain, mail_body);
}
