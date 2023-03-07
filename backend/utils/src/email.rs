use crate::configuration::{EmailMode, EmailSettings};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use secrecy::ExposeSecret;

#[derive(Clone)]
pub struct EmailClient {
    sender: String,
    transport: Option<SmtpTransport>,
    settings: EmailSettings,
}

impl EmailClient {
    pub fn send_email(&self, to: String, subject: String, body: String) -> anyhow::Result<()> {
        match self.settings.mode {
            EmailMode::Terminal => {
                println!("From: {}", self.sender);
                println!("To: {to}");
                println!("Subject: {subject}\n\n");
                println!("{body}");
            }
            EmailMode::SMTP => {
                let email = Message::builder()
                    .from(self.sender.parse().unwrap())
                    .reply_to(self.sender.parse().unwrap())
                    .to(to.parse().unwrap())
                    .subject(&subject)
                    .header(ContentType::TEXT_PLAIN)
                    .body(body)?;
                self.transport.clone().unwrap().send(&email)?;
            }
        }
        Ok(())
    }

    pub fn init(settings: EmailSettings) -> Self {
        let config = settings.clone();
        let transport = match config.mode {
            EmailMode::Terminal => None,
            EmailMode::SMTP => {
                let creds = Credentials::new(
                    config.username.unwrap().expose_secret().to_owned(),
                    config.password.unwrap().expose_secret().to_owned(),
                );
                let transport = SmtpTransport::starttls_relay(&config.relay.unwrap())
                    .unwrap()
                    .credentials(creds)
                    .build();
                Some(transport)
            }
        };

        Self {
            sender: config.sender,
            transport,
            settings,
        }
    }
}
