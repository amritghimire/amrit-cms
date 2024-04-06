use crate::entities::user::User;

#[derive(Clone, Default)]
pub struct AppState {
    pub dark_mode: bool,
    pub user: Option<User>,
}
