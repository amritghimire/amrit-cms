use crate::configuration::{EmailMode, EmailSettings, TlsMode};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::{Message, SmtpTransport, Transport};
use secrecy::ExposeSecret;

pub trait EmailTrait {
    fn send_email(&mut self, to: String, subject: String, body: String) -> anyhow::Result<()>;
    fn get_sender(&self) -> &str;
    fn get_mails(&self) -> Vec<EmailObject> {
        vec![]
    }
}

#[derive(Clone)]
pub enum EmailClient {
    SmtpClient(SmtpClient),
    TerminalClient(TerminalClient),
    InMemoryClient(InMemoryClient),
}

#[derive(Clone)]
pub struct SmtpClient {
    sender: String,
    transport: SmtpTransport,
}

impl SmtpClient {
    pub fn new(config: EmailSettings) -> Self {
        let transport = Self::get_transport(&config);

        Self {
            sender: config.sender,
            transport,
        }
    }

    fn get_transport(settings: &EmailSettings) -> SmtpTransport {
        let creds = Credentials::new(
            settings
                .username
                .as_ref()
                .unwrap()
                .expose_secret()
                .to_owned(),
            settings
                .password
                .as_ref()
                .unwrap()
                .expose_secret()
                .to_owned(),
        );

        let port = match settings.tls.as_ref().unwrap_or(&TlsMode::Local) {
            TlsMode::Local => 25,
            _ => 465,
        };

        match settings.tls.as_ref().unwrap_or(&TlsMode::Local) {
            TlsMode::Local => SmtpTransport::builder_dangerous(settings.relay.as_ref().unwrap())
                .tls(Tls::None)
                .port(settings.port.unwrap_or(port))
                .timeout(Some(std::time::Duration::from_secs(10)))
                .build(),
            TlsMode::Tls => SmtpTransport::relay(settings.relay.as_ref().unwrap())
                .unwrap()
                .credentials(creds)
                .port(settings.port.unwrap_or(port))
                .build(),
            TlsMode::StartTls => SmtpTransport::starttls_relay(settings.relay.as_ref().unwrap())
                .unwrap()
                .credentials(creds)
                .port(settings.port.unwrap_or(port))
                .build(),
        }
    }
}

impl EmailTrait for SmtpClient {
    fn send_email(&mut self, to: String, subject: String, body: String) -> anyhow::Result<()> {
        let email = Message::builder()
            .from(self.sender.parse()?)
            .reply_to(self.sender.parse()?)
            .to(to.parse()?)
            .subject(subject)
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
    fn send_email(&mut self, to: String, subject: String, body: String) -> anyhow::Result<()> {
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

#[derive(Clone)]
pub struct EmailObject {
    pub sender: String,
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Clone)]
pub struct InMemoryClient {
    sender: String,
    mails: Vec<EmailObject>,
}

impl InMemoryClient {
    pub fn new(settings: EmailSettings) -> Self {
        Self {
            sender: settings.sender,
            mails: Vec::new(),
        }
    }
}

impl EmailTrait for InMemoryClient {
    fn send_email(&mut self, to: String, subject: String, body: String) -> anyhow::Result<()> {
        self.mails.push(EmailObject {
            sender: self.sender.clone(),
            to,
            subject,
            body,
        });
        Ok(())
    }

    fn get_sender(&self) -> &str {
        &self.sender
    }

    fn get_mails(&self) -> Vec<EmailObject> {
        self.mails.clone()
    }
}

pub fn get_email_client(settings: EmailSettings) -> EmailClient {
    match settings.mode {
        EmailMode::Terminal => EmailClient::TerminalClient(TerminalClient::new(settings)),
        EmailMode::SMTP => EmailClient::SmtpClient(SmtpClient::new(settings)),
        EmailMode::InMemory => EmailClient::InMemoryClient(InMemoryClient::new(settings)),
    }
}

impl EmailClient {
    pub fn unwrap(self) -> Box<dyn EmailTrait> {
        match self {
            EmailClient::SmtpClient(c) => Box::new(c),
            EmailClient::TerminalClient(c) => Box::new(c),
            EmailClient::InMemoryClient(c) => Box::new(c),
        }
    }
}
