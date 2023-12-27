use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command;
use std::fs::File;
use serde::{Deserialize, Serialize};
use ring::aead::{self, Aad, BoundKey, Nonce, UnboundKey, AES_256_GCM, LessSafeKey};
use ring::rand::{SecureRandom, SystemRandom};

use crate::user::User;
use toml;

const CONFIG_FILE_PATH: &str = "config/seigrconfig.toml";
const KEY_LENGTH: usize = 32;
const NONCE_LENGTH: usize = 12;


#[derive(Debug, Deserialize, Serialize)]
pub struct SeigrConfig {
    pub username: String,
    pub password: String,
    pub email: String,
    beeid: String,
    // Add other application options here
    from_file: bool,
    get_user: User,
    add_user: User,
    pub user: Option<User>,
    users: std::collections::HashMap<String, User>,
    pub save_to_file: bool,
    key: [u8; KEY_LENGTH],
    nonce: [u8; NONCE_LENGTH],

}

impl SeigrConfig {
    pub fn new() -> io::Result<Self> {
        let config_path = Path::new(CONFIG_FILE_PATH);
        if !config_path.exists() {
            // Create a new config file if it doesn't exist
            fs::create_dir_all(config_path.parent().unwrap())?;
            let mut config_file = fs::File::create(config_path)?;
            let mut config = SeigrConfig::default();
            config.key = generate_key(); // Add this line
            config.nonce = generate_nonce(); // Add this line
            let encrypted_config = encrypt_config(&config)?;
            config_file.write_all(&encrypted_config)?;
            Ok(config)
        } else {
            // If the config file exists, read it
            let config = SeigrConfig::default(); // Create a default config
            Self::read_config(&config)
        }
    }

    pub fn read_config(seigr_config: &SeigrConfig) -> io::Result<SeigrConfig> {
        let config_path = Path::new(CONFIG_FILE_PATH);
        let mut config_file = fs::File::open(config_path)?;
        let mut encrypted_config = Vec::new();
        config_file.read_to_end(&mut encrypted_config)?;
        let decrypted_config_str = decrypt_config(seigr_config, &encrypted_config)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
        // Parse the decrypted_config_str into a SeigrConfig
        let decrypted_config: SeigrConfig = serde_json::from_str(&decrypted_config_str)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
        Ok(decrypted_config)
    }

    pub fn save_config(&self) -> io::Result<()> {
        let config_path = Path::new(CONFIG_FILE_PATH);
        let mut config_file = fs::File::create(config_path)?;
        let encrypted_config = encrypt_config(self)?;
        config_file.write_all(&encrypted_config)?;
        Ok(())
    }

    // Getters and setters for username, password, email, bee_id, and other options
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn beeid(&self) -> &str {
        &self.beeid
    }

    pub fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            email: String::new(),
            beeid: String::new(),
            // Initialize other options here
            from_file: false,
            get_user: User::default(),
            add_user: User::default(),
            user: None,
            users: std::collections::HashMap::new(),
            save_to_file: false,
            key: [0u8; KEY_LENGTH],
            nonce: [0u8; NONCE_LENGTH],
        }
    }

    pub fn save_to_file(&self) -> io::Result<()> {
        let config_path = Path::new(CONFIG_FILE_PATH);
        let mut config_file = fs::File::create(config_path)?;
        let encrypted_config = encrypt_config(self)?;
        config_file.write_all(&encrypted_config)?;
        Ok(())
    }

    pub fn from_file() -> Result<SeigrConfig, std::io::Error> {
        // Try to read the file
        let file = File::open("seigrconfig.toml");
    
        match file {
            Ok(mut file) => {
                // If the file is successfully read, read it into a string
                let mut config_string = String::new();
                file.read_to_string(&mut config_string)?;
    
                // Parse the string into a SeigrConfig and return it
                let config: SeigrConfig = toml::from_str(&config_string)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                Ok(config)
            }
            Err(e) => {
                // If the file cannot be read, return the error
                Err(e)
            }
        }
    }

    pub fn set_username(username: String) -> io::Result<()> {
        let mut config = SeigrConfig::new()?;
        config.username = username;
        config.save_config()?;
        Ok(())
    }

    pub fn set_password(password: String) -> io::Result<()> {
        let mut config = SeigrConfig::new()?;
        config.password = password;
        config.save_config()?;
        Ok(())
    }

    pub fn set_email(email: String) -> io::Result<()> {
        let mut config = SeigrConfig::new()?;
        config.email = email;
        config.save_config()?;
        Ok(())
    }

    pub fn set_beeid(beeid: String) -> io::Result<()> {
        let mut config = SeigrConfig::new()?;
        config.beeid = beeid;
        config.save_config()?;
        Ok(())
    }

    pub fn add_user(&mut self, username: String, password: String, email: String) -> Result<User, io::Error> {
        let user = User::new(username.clone(), password, email);
        self.users.insert(username, user.clone());
        Ok(user)
    }

    pub fn get_user(&self, username: String) -> Result<User, io::Error> {
        match self.users.get(&username) {
            Some(user) => Ok(user.clone()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "User not found")),
        }
    }
    
}


fn generate_key() -> [u8; KEY_LENGTH] {
    let rng = SystemRandom::new();
    let mut key = [0u8; KEY_LENGTH];
    rng.fill(&mut key).unwrap();
    key
}

fn generate_nonce() -> [u8; NONCE_LENGTH] {
    let rng = SystemRandom::new();
    let mut nonce = [0u8; NONCE_LENGTH];
    rng.fill(&mut nonce).unwrap();
    nonce
}

pub fn encrypt_config(config: &SeigrConfig) -> io::Result<Vec<u8>> {
    let key_bytes = generate_key();
    let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &config.key)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to create unbound key"))?;
    let key = aead::LessSafeKey::new(unbound_key);
    let additional_data: aead::Aad<[u8; 0]> = aead::Aad::empty();

    let nonce = aead::Nonce::assume_unique_for_key(generate_nonce());
    let config_str = toml::to_string(config)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
    let mut config_bytes = config_str.into_bytes();

    let tag = key.seal_in_place_separate_tag(nonce, additional_data, &mut config_bytes)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Encryption failed"))?;

    // Append the authentication tag to the encrypted config
    config_bytes.extend_from_slice(tag.as_ref());

    Ok(config_bytes)
}

pub fn decrypt_config(config: &SeigrConfig, encrypted_config: &[u8]) -> io::Result<String> {
    let config = SeigrConfig::new()?;
    let key_bytes = generate_key(); // You need to use the same key that was used for encryption
    let unbound_key = UnboundKey::new(&aead::AES_256_GCM, &config.key)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to create unbound key"))?;
    let mut key = LessSafeKey::new(unbound_key);

    let nonce = aead::Nonce::assume_unique_for_key(generate_nonce());
    let additional_data: Aad<[u8; 0]> = Aad::empty();

    let mut encrypted_config = encrypted_config.to_vec();

    let decrypted_config = key.open_in_place(nonce, additional_data, &mut encrypted_config)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Decryption failed"))?;

    let decrypted_config_str = String::from_utf8(decrypted_config.to_vec())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    Ok(decrypted_config_str)
}