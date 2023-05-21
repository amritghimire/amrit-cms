use crate::configuration::{EmailMode, EmailSettings};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use secrecy::ExposeSecret;

pub trait EmailTrait {
    fn send_email(&self, to: String, subject: String, body: String) -> anyhow::Result<()>;
    fn get_sender(&self) -> &str;
}

#[derive(Clone)]
pub enum EmailClient {
    SmtpClient(SmtpClient),
    TerminalClient(TerminalClient),
}

#[derive(Clone)]
pub struct SmtpClient {
    sender: String,
    transport: SmtpTransport,
}

impl SmtpClient {
    pub fn new(settings: EmailSettings) -> Self {
        let config = settings;
        let creds = Credentials::new(
            config.username.unwrap().expose_secret().to_owned(),
            config.password.unwrap().expose_secret().to_owned(),
        );
        let transport = SmtpTransport::starttls_relay(&config.relay.unwrap())
            .unwrap()
            .credentials(creds)
            .build();

        Self {
            sender: config.sender,
            transport,
        }
    }
}

impl EmailTrait for SmtpClient {
    fn send_email(&self, to: String, subject: String, body: String) -> anyhow::Result<()> {
        let email = Message::builder()
            .from(self.sender.parse().unwrap())
            .reply_to(self.sender.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(&subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body)?;
        self.transport.clone().send(&email)?;
        Ok(())
    }

    fn get_sender(&self) -> &str {
        &self.sender
    }
}

#[derive(Clone)]
pub struct TerminalClient {
    sender: String,
}

impl TerminalClient {
    pub fn new(settings: EmailSettings) -> Self {
        let sender = settings.sender;
        Self { sender }
    }
}

impl EmailTrait for TerminalClient {
    fn send_email(&self, to: String, subject: String, body: String) -> anyhow::Result<()> {
        println!("From: {}", self.sender);
        println!("To: {to}");
        println!("Subject: {subject}\n\n");
        println!("{body}");
        Ok(())
    }

    fn get_sender(&self) -> &str {
        &self.sender
    }
}

pub fn get_email_client(settings: EmailSettings) -> EmailClient {
    match settings.mode {
        EmailMode::Terminal => EmailClient::TerminalClient(TerminalClient::new(settings)),
        EmailMode::SMTP => EmailClient::SmtpClient(SmtpClient::new(settings)),
    }
}

impl EmailClient {
    pub fn unwrap(self) -> Box<dyn EmailTrait> {
        match self {
            EmailClient::SmtpClient(c) => Box::new(c),
            EmailClient::TerminalClient(c) => Box::new(c),
        }
    }
}
