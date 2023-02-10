use validator::Validate;

#[derive(serde::Serialize, serde::Deserialize, Validate)]
pub struct SubscriptionPayload {
    #[validate(length(min = 1, message = "Can not be empty"))]
    name: String,
    email: String,
}
