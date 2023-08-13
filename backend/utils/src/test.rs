use sqlx::PgPool;
use std::sync::mpsc::SyncSender;
use crate::configuration::{RunMode, Settings};
use crate::email::{EmailClient, EmailObject, MessagePassingClient};
use crate::state::AppState;

pub fn test_state_for_email(pool: PgPool, tx: SyncSender<EmailObject>) -> AppState {
    let settings = Settings::get_config(RunMode::Test).expect("Unable to fetch test config");

    let email_client = EmailClient::MessagePassingClient(
        MessagePassingClient::with_tx(settings.email.clone(), tx)
    );
    AppState::test_email_state(pool, email_client)
}
