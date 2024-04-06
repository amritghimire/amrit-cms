use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub username: String,
    pub is_confirmed: bool,
}
