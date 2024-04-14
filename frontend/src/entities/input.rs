use std::collections::HashMap;

pub struct UserInput {
    entries: HashMap<String, String>,
}

impl UserInput {
    pub fn new() -> UserInput {
        UserInput {
            entries: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: String) {
        self.entries.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> String {
        String::from(self.entries.get(key).unwrap_or(&"".to_string()))
    }
}
