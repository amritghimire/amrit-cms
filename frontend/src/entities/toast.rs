#[derive(Clone, PartialOrd, PartialEq)]
pub enum ToastType {
    Secondary,
    Success,
    Error,
    Info,
    Dark,
    Warning,
}

impl From<&str> for ToastType {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "secondary" => ToastType::Secondary,
            "success" => ToastType::Success,
            "info" => ToastType::Info,
            "dark" => ToastType::Dark,
            "warning" => ToastType::Warning,
            _ => ToastType::Error,
        }
    }
}

#[derive(Clone, PartialOrd, PartialEq)]
pub struct ToastMessage {
    pub id: u16,
    pub message: String,
    pub typ: ToastType,
}

impl ToastMessage {
    pub fn new(id: u16, message: impl AsRef<str>, typ: ToastType) -> Self {
        Self {
            id,
            message: message.as_ref().to_string(),
            typ,
        }
    }

    pub fn class(&self) -> &str {
        match self.typ {
            ToastType::Info => "bg-blue-500",
            ToastType::Secondary => "bg-gray-500",
            ToastType::Success => "bg-teal-500",
            ToastType::Error => "bg-red-500",
            ToastType::Warning => "bg-yellow-500",
            ToastType::Dark => "bg-gray-800",
        }
    }
}
