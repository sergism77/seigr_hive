use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};
use std::path::Path;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct User {
    username: String,
    email: String,
    password: String,
    beeid: String,
    authenticate: bool,
}

impl User {
    pub fn new(username: String, email: String, password: String) -> Self {
        let beeid = Self::generate_beeid(&username, &email); // Implement your logic to generate a unique beeid
        User {
            username,
            email,
            password,
            beeid,
            authenticate: false,
        }
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
}