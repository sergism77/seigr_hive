use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

use std::fs::File;
use serde::{Deserialize, Serialize};
use ring::aead::{self, Aad, BoundKey, UnboundKey, LessSafeKey};
use ring::rand::{SecureRandom, SystemRandom};

use crate::user::User;
use toml;

const CONFIG_FILE_PATH: &str = "config/seigrconfig.toml";
pub(crate) const KEY_LENGTH: usize = 32;
pub(crate) const NONCE_LENGTH: usize = 12;


#[derive(Debug, Deserialize, Serialize, Clone)]
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
    pub users: std::collections::HashMap<String, User>,
    pub save_to_file: bool,
    key: [u8; KEY_LENGTH],
    nonce: [u8; NONCE_LENGTH],
    password_hash: Option<String>,
}

impl SeigrConfig {
    pub fn new(db_path: &str, key: &[u8; KEY_LENGTH], nonce: &[u8; NONCE_LENGTH]) -> io::Result<Self> {
        let config_path = Path::new(db_path);
        if !config_path.exists() {
            // Create a new config file if it doesn't exist
            fs::create_dir_all(config_path.parent().unwrap())?;
            let mut config_file = fs::File::create(config_path)?;
            let mut config = SeigrConfig::default();
            // Set the key and other fields...
    
            // Write the config to the file
            let encrypted_config = encrypt_config(&config, key, nonce)?; // Call encrypt_config with the appropriate arguments
            config_file.write_all(&encrypted_config)?; // Write the result to the file
            Ok(config)
        } else {
            // If the config file exists, read it
            Self::read_config(key)
        }
    }
    
    pub fn read_config(key: &[u8; KEY_LENGTH]) -> io::Result<SeigrConfig> {
        let config_path = Path::new(CONFIG_FILE_PATH);
        let mut config_file = fs::File::open(config_path)?;
        let mut encrypted_config = Vec::new();
        config_file.read_to_end(&mut encrypted_config)?;
        let decrypted_config = decrypt_config(key, &encrypted_config)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
        Ok(decrypted_config)
    }

    pub fn save_config(&self) -> io::Result<()> {
        let config_path = Path::new(CONFIG_FILE_PATH);
        let mut config_file = fs::File::create(config_path)?;
    
        // Generate a new nonce for each encryption operation
        let nonce = generate_nonce().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to generate nonce"))?;
        let encrypted_config = encrypt_config(self, &self.key, &nonce)?; // Pass the nonce to encrypt_config
    
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
            password_hash: None,
        }
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

    pub fn has_users(&self) -> bool {
        !self.users.is_empty()
    }

    pub fn user_exists(&self, username: &str) -> bool {
        self.users.contains_key(username)
    }

    pub fn set_username(&mut self, username: String) -> io::Result<()> {
        self.username = username;
        self.save_config()?;
        Ok(())
    }
    
    pub fn set_password(&mut self, password: String) -> io::Result<()> {
        self.password = password;
        self.save_config()?;
        Ok(())
    }
    
    pub fn set_email(&mut self, email: String) -> io::Result<()> {
        self.email = email;
        self.save_config()?;
        Ok(())
    }

    pub fn set_beeid(&mut self, beeid: String) -> io::Result<()> {
        self.beeid = beeid;
        self.save_config()?;
        Ok(())
    }

    pub fn add_user(&mut self, username: String, password: String, email: String) -> Result<User, io::Error> {
        let user = User::new(username.clone(), password, email)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        self.users.insert(username, user.clone());
        self.save_config()?; // Save the updated config to the file
        Ok(user)
    }

    pub fn get_user(&self, username: String) -> Result<User, std::io::Error> {
        self.users.get(&username)
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "User not found"))
    }

    pub fn get_username(&self) -> Result<String, io::Error> {
        match self.users.keys().next() {
            Some(username) => Ok(username.clone()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "User's username not found")),
        }
    }

    pub fn get_password(&self) -> Result<String, io::Error> {
        match self.users.values().next() {
            Some(user) => Ok(user.password.clone()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "User's password not found")),
        }
    }

    pub fn get_email(&self) -> Result<String, io::Error> {
        match self.users.values().next() {
            Some(user) => Ok(user.email.clone()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "User's email not found")),
        }
    }

    pub fn get_beeid(&self) -> Result<String, io::Error> {
        match self.users.values().next() {
            Some(user) => Ok(user.beeid.clone()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "User's beeid not found")),
        }
    }

    pub fn get_key(&self) -> Result<[u8; KEY_LENGTH], io::Error> {
        match self.users.values().next() {
            Some(user) => Ok(user.key.clone()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "User's key not found")),
        }
    }

    // Save user data
    pub fn save_users(&self) -> std::io::Result<()> {
        let mut file = File::create("user_data")?;
        file.write_all(serde_json::to_string(&self.users)?.as_bytes())?;
        Ok(())
    }

    // Load user data
    pub fn load_users(&mut self) -> std::io::Result<()> {
        let mut file = File::open("user_data")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        self.users = serde_json::from_str(&contents)?;
        Ok(())
    }
    
}


pub fn generate_key() -> Result<[u8; KEY_LENGTH], ring::error::Unspecified> {
    let rng = SystemRandom::new();
    let mut key = [0u8; KEY_LENGTH];
    rng.fill(&mut key)?;
    println!("Key: {:?}", key);
    Ok(key)
}

fn generate_nonce() -> Result<[u8; NONCE_LENGTH], ring::error::Unspecified> {
    let rng = SystemRandom::new();
    let mut nonce = [0u8; NONCE_LENGTH];
    rng.fill(&mut nonce)?;
    println!("Nonce: {:?}", nonce);
    Ok(nonce)
}

fn encrypt_config(config: &SeigrConfig, key: &[u8; KEY_LENGTH], nonce: &[u8; NONCE_LENGTH]) -> io::Result<Vec<u8>> {
    let unbound_key = UnboundKey::new(&aead::AES_256_GCM, key)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to create unbound key"))?;
    let key = LessSafeKey::new(unbound_key);

    let config_str = serde_json::to_string(config) // Use JSON for serialization
        .map_err(|err| std::io::Error::new(io::ErrorKind::Other, err.to_string()))?;

    let additional_data: Aad<[u8; 0]> = Aad::empty();

    let nonce = aead::Nonce::assume_unique_for_key(*nonce); // Use the nonce passed as argument
    let mut config_bytes = config_str.into_bytes(); // Convert the configuration string to bytes
    let mut result = Vec::new();
    result.extend_from_slice(nonce.as_ref());

    // Encrypt the config
    key.seal_in_place_append_tag(nonce, additional_data, &mut config_bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Encryption failed"))?;

    result.extend_from_slice(&config_bytes);
    Ok(result)
}

fn decrypt_config(key: &[u8; KEY_LENGTH], encrypted_config: &[u8]) -> io::Result<SeigrConfig> {
    let unbound_key = UnboundKey::new(&aead::AES_256_GCM, key)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to create unbound key"))?;
    let key = LessSafeKey::new(unbound_key);

    // Split the nonce and the encrypted config
    let (nonce_bytes, encrypted_config) = encrypted_config.split_at(NONCE_LENGTH);
    let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to create nonce"))?;
    let additional_data: Aad<[u8; 0]> = Aad::empty();

    let mut encrypted_config = encrypted_config.to_vec();

    // Decrypt the config
    let decrypted_config = key.open_in_place(nonce, additional_data, &mut encrypted_config)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Decryption failed"))?;

    let decrypted_config_str = String::from_utf8(decrypted_config.to_vec())
        .map_err(|err| std::io::Error::new(io::ErrorKind::Other, err.to_string()))?;

    let decrypted_config: SeigrConfig = serde_json::from_str(&decrypted_config_str) // Use JSON for deserialization
        .map_err(|err| std::io::Error::new(io::ErrorKind::Other, err.to_string()))?;

    Ok(decrypted_config)
}