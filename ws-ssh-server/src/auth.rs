use std::collections::HashMap;
use russh_keys::key::PublicKey;

pub struct AuthStore {
    users: HashMap<String, String>,
}

impl AuthStore {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        users.insert("admin".to_string(), "secret123".to_string());
        users.insert("guest".to_string(), "guest".to_string());
        Self { users }
    }

    pub fn verify_password(&self, user: &str, pass: &str) -> bool {
        match self.users.get(user) {
            Some(stored) => stored == pass,
            None => false,
        }
    }

    pub fn verify_pubkey(&self, _user: &str, _key: &PublicKey) -> bool {
        true
    }
}
