use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password: String,
    pub beeid: String,
    pub authenticate: bool,
    pub key: [u8; 32],
    pub password_hash: String,
}

impl User {
    pub fn new(username: String, email: String, password: String) -> Result<Self, &'static str> {
        let beeid = Self::generate_beeid(&username, &email); // Implement your logic to generate a unique beeid
        // Check if the beeid is valid, if not return an error
        if beeid.is_empty() {
            return Err("Failed to generate beeid");
        }
        Ok(User {
            username,
            email,
            password,
            beeid,
            authenticate: false,
            key: [0; 32],
            password_hash: String::new(),
        })
    }
    
    pub fn generate_beeid(username: &str, email: &str) -> String {
        let mut hasher = DefaultHasher::new();
        hasher.write(username.as_bytes());
        hasher.write(email.as_bytes());
        let prefix = "seigr_bee";
        let system_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        hasher.write(&system_time.as_secs().to_be_bytes());
        // Hash the username, system time and email into a 10 digit hexadecimal number
        let beeid = format!("{:x}", hasher.finish());
        // Append the prefix to the beeid
        let beeid = format!("{}{}", prefix, beeid);
        beeid
    }

    pub fn authenticate(&self, password: &str) -> bool {
        self.password == password
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn beeid(&self) -> &str {
        &self.beeid
    }

    pub fn update_username(&mut self, new_username: String) {
        self.username = new_username;
    }

    pub fn update_password(&mut self, new_password: String) {
        self.password = new_password;
    }

    pub fn update_email(&mut self, new_email: String) {
        self.email = new_email;
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }
}