use std::sync::mpsc;
use utils::configuration::{RunMode, Settings};
use utils::email::{EmailClient, MessagePassingClient};

#[sqlx::test]
async fn send_email_saved_in_memory() {
    let recipient_mail = "mail@example.com".to_string();
    let mail_subject = "New subject".to_string();
    let mail_body = "Body of email".to_string();

    let (tx, rx) = mpsc::sync_channel(2);
    let settings = Settings::get_config(RunMode::Test).expect("Unable to fetch test config");

    let email_client = EmailClient::MessagePassingClient(
        MessagePassingClient::with_tx(settings.email.clone(), tx)
    );


    email_client.unwrap()
        .send_email(
            recipient_mail.clone(),
            mail_subject.clone(),
            mail_body.clone(),
        )
        .expect("Unable to send email");

    let email = rx.recv().unwrap();

    assert_eq!(email.sender, settings.email.sender);
    assert_eq!(email.to, recipient_mail);
    assert_eq!(email.subject, mail_subject);
    assert_eq!(email.body, mail_body);
}
