use crate::configuration::{EmailMode, EmailSettings, TlsMode};
use async_trait::async_trait;
use lettre::address::AddressError;
use lettre::message::MultiPart;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::transport::smtp::Error;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use secrecy::ExposeSecret;
use std::sync::mpsc::SyncSender;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

#[derive(Debug)]
pub struct EmailError {
    pub message: String,
}

impl std::fmt::Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to send an email {}.", self.message)
    }
}

impl std::error::Error for EmailError {}

impl From<AddressError> for EmailError {
    fn from(value: AddressError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<lettre::error::Error> for EmailError {
    fn from(value: lettre::error::Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<Error> for EmailError {
    fn from(value: Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

pub trait EmailTrait {
    fn get_sender(&self) -> &str;
}

#[derive(Clone)]
pub enum EmailClient {
    SmtpClient(SmtpClient),
    TerminalClient(TerminalClient),
    MessagePassingClient(MessagePassingClient),
}

#[derive(Clone)]
pub struct SmtpClient {
    sender: String,
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl SmtpClient {
    fn get_transport(settings: &EmailSettings) -> AsyncSmtpTransport<Tokio1Executor> {
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
            TlsMode::Local => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
                settings.relay.as_ref().unwrap(),
            )
            .tls(Tls::None)
            .port(settings.port.unwrap_or(port))
            .timeout(Some(std::time::Duration::from_secs(10)))
            .build(),
            TlsMode::Tls => {
                AsyncSmtpTransport::<Tokio1Executor>::relay(settings.relay.as_ref().unwrap())
                    .unwrap()
                    .credentials(creds)
                    .port(settings.port.unwrap_or(port))
                    .build()
            }
            TlsMode::StartTls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(
                settings.relay.as_ref().unwrap(),
            )
            .unwrap()
            .credentials(creds)
            .port(settings.port.unwrap_or(port))
            .build(),
        }
    }

    pub fn new(config: EmailSettings) -> Self {
        let transport = SmtpClient::get_transport(&config);
        Self {
            sender: config.sender.clone(),
            transport,
        }
    }

    async fn send_email(
        &self,
        to: String,
        subject: String,
        plain: String,
        html: String,
    ) -> Result<(), EmailError> {
        let email_body = MultiPart::alternative_plain_html(plain, html);

        let email = Message::builder()
            .from(self.sender.parse()?)
            .reply_to(self.sender.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .multipart(email_body)?;
        self.transport.send(email).await?;
        Ok(())
    }
}

impl EmailTrait for SmtpClient {
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

    fn send_email(
        &self,
        to: String,
        subject: String,
        plain: String,
        html: String,
    ) -> Result<(), EmailError> {
        println!("From: {}", self.sender);
        println!("To: {to}");
        println!("Subject: {subject}\n\n");
        println!("{plain}");
        println!("----------");
        println!("{html}");
        Ok(())
    }
}

impl EmailTrait for TerminalClient {
    fn get_sender(&self) -> &str {
        &self.sender
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct EmailObject {
    pub sender: String,
    pub to: String,
    pub subject: String,
    pub plain: String,
    pub html: String,
}

#[derive(Clone)]
pub struct MessagePassingClient {
    sender: String,
    tx: SyncSender<EmailObject>,
}

impl MessagePassingClient {
    pub fn new(settings: EmailSettings) -> Self {
        let (tx, _) = mpsc::sync_channel(5 /* usize */);

        Self {
            sender: settings.sender,
            tx,
        }
    }

    pub fn with_tx(settings: EmailSettings, tx: SyncSender<EmailObject>) -> Self {
        Self {
            sender: settings.sender,
            tx,
        }
    }

    fn send_email(
        &self,
        to: String,
        subject: String,
        plain: String,
        html: String,
    ) -> Result<(), EmailError> {
        self.tx
            .send(EmailObject {
                sender: self.sender.clone(),
                to,
                subject,
                plain,
                html,
            })
            .unwrap();
        Ok(())
    }
}

#[async_trait(?Send)]
impl EmailTrait for MessagePassingClient {
    fn get_sender(&self) -> &str {
        &self.sender
    }
}

pub fn get_email_client(settings: EmailSettings) -> EmailClient {
    match settings.mode {
        EmailMode::Terminal => EmailClient::TerminalClient(TerminalClient::new(settings)),
        EmailMode::SMTP => EmailClient::SmtpClient(SmtpClient::new(settings)),
        EmailMode::MessagePassing => {
            EmailClient::MessagePassingClient(MessagePassingClient::new(settings))
        }
    }
}

impl EmailClient {
    pub fn unwrap(self) -> Box<dyn EmailTrait> {
        match self {
            EmailClient::SmtpClient(c) => Box::new(c),
            EmailClient::TerminalClient(c) => Box::new(c),
            EmailClient::MessagePassingClient(c) => Box::new(c),
        }
    }
}

pub async fn send_email(
    client: &EmailClient,
    to: String,
    subject: String,
    plain: String,
    html: String,
) -> Result<(), EmailError> {
    match client {
        EmailClient::SmtpClient(c) => c.send_email(to, subject, plain, html).await,
        EmailClient::TerminalClient(c) => c.send_email(to, subject, plain, html),
        EmailClient::MessagePassingClient(c) => c.send_email(to, subject, plain, html),
    }
}

fn get_successful_email_count(spawned_threads: Vec<JoinHandle<Result<(), EmailError>>>) -> u64 {
    spawned_threads
        .into_iter()
        .map(|c| c.join().unwrap().is_ok() as u64)
        .sum::<u64>()
}

pub async fn send_emails(client: EmailClient, emails: Vec<EmailObject>) -> u64 {
    match client {
        EmailClient::SmtpClient(c) => {
            let mut spawned_threads = vec![];
            let client = Arc::new(Mutex::new(c));

            for email in emails {
                let client = Arc::clone(&client);
                spawned_threads.push(thread::spawn(move || {
                    let runtime = tokio::runtime::Runtime::new().unwrap();
                    let client = client.lock().unwrap();

                    runtime.block_on(async {
                        client
                            .send_email(email.to, email.subject, email.plain, email.html)
                            .await
                    })
                }));
            }
            get_successful_email_count(spawned_threads)
        }
        EmailClient::TerminalClient(c) => {
            let mut counts = 0;
            for email in emails {
                counts += c
                    .send_email(email.to, email.subject, email.plain, email.html)
                    .is_ok() as u64;
            }
            counts
        }
        EmailClient::MessagePassingClient(c) => {
            let mut counts = 0;
            for email in emails {
                counts += c
                    .send_email(email.to, email.subject, email.plain, email.html)
                    .is_ok() as u64;
            }
            counts
        }
    }
}

pub fn get_link(s: &str) -> String {
    let links: Vec<_> = linkify::LinkFinder::new()
        .links(s)
        .filter(|l| *l.kind() == linkify::LinkKind::Url)
        .collect();
    assert_eq!(links.len(), 1);
    links[0].as_str().to_owned()
}
