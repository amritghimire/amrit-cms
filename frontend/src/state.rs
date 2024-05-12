use crate::entities::toast::{ToastMessage, ToastType};
use crate::entities::user::User;

#[derive(Clone, Default)]
pub struct AppState {
    pub toast_index: u16,
    pub dark_mode: bool,
    pub user: Option<User>,
    pub toast_messages: Vec<ToastMessage>,
}

impl AppState {
    pub fn add_toast(&mut self, typ: ToastType, message: impl AsRef<str>) {
        self.toast_messages
            .push(ToastMessage::new(self.toast_index, message, typ));
        self.toast_index += 1;
    }
}
