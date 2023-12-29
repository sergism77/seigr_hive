use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::hash::Hash;
use std::hash::Hasher;
use std::{fs, result, io};
use blake2::{Blake2b, Digest};
use generic_array::typenum::U64;
use crate::seigrconfig::SeigrConfig;
use crate::user::User;
use bcrypt::{hash, DEFAULT_COST};
use std::error::Error as StdError;
use std::fmt;
use crate::seigrconfig::{KEY_LENGTH, NONCE_LENGTH};

pub enum DatabaseError {
    FileNotFound(String),
    CubeNotFound(String),
    Other(String),
    IoError(io::Error),
    HashingError(String),
    UserNotFound,
    AuthenticationFailed,
    LockFailed,
}

pub struct Transaction {
    changes: Vec<Change>,
}

enum Change {
    StoreFile { filename: String, data: Vec<u8> },
    DeleteFile { filename: String },
    // Add more variants here for other types of changes
}

impl Transaction {
    pub fn new() -> Self {
        Self { changes: Vec::new() }
    }

    pub fn store_file(&mut self, filename: String, data: Vec<u8>) {
        self.changes.push(Change::StoreFile { filename, data });
    }

    pub fn delete_file(&mut self, filename: String) {
        self.changes.push(Change::DeleteFile { filename });
    }

    pub fn commit(self, database: &mut Database) -> Result<(), DatabaseError> {
        for change in self.changes {
            match change {
                Change::StoreFile { filename, data } => database.store_file(filename, data)?,
                Change::DeleteFile { filename } => database.delete_file(filename)?,
                // Handle other types of changes here
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BeeCell {
    id: String,
    data: Vec<u8>,
    hash: String,
    previous_id: String,
    next_id: String,
}

#[derive(Debug)]
pub struct Frame {
    id: String,
    beecells: Vec<BeeCell>,
    previous_id: String,
    next_id: String,
}

#[derive(Debug)]
pub struct Cube {
    id: String,
    frames: Vec<Frame>,
}

#[derive(Debug)]
pub struct Database {
    file_links: HashMap<String, Vec<String>>,
    cubes: HashMap<String, Cube>,
    users: HashMap<String, User>,
}

#[derive(Default, Debug, Clone)]
pub struct File {
    pub filename: String,
    pub beeid: String,
    pub data: Vec<u8>,
    pub previous_beecell_id: String,
    pub next_beecell_id: String,
}

impl From<std::io::Error> for DatabaseError {
    fn from(error: std::io::Error) -> Self {
        DatabaseError::IoError(error)
    }
}

impl From<bcrypt::BcryptError> for DatabaseError {
    fn from(error: bcrypt::BcryptError) -> Self {
        DatabaseError::HashingError(format!("Hashing error: {}", error))
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Add display messages for other variants...
            DatabaseError::FileNotFound(filename) => write!(f, "File not found: {}", filename),
            DatabaseError::CubeNotFound(cube_id) => write!(f, "Cube not found: {}", cube_id),
            DatabaseError::Other(message) => write!(f, "Other error: {}", message),
            DatabaseError::IoError(error) => write!(f, "IO error: {}", error),
            DatabaseError::HashingError(message) => write!(f, "Hashing error: {}", message),
            DatabaseError::UserNotFound => write!(f, "User not found"),
            DatabaseError::AuthenticationFailed => write!(f, "Authentication failed"),
            DatabaseError::LockFailed => write!(f, "Failed to acquire lock"),
        }
    }
}

impl fmt::Debug for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Add debug messages for each variant...
            DatabaseError::FileNotFound(filename) => write!(f, "File not found: {}", filename),
            DatabaseError::CubeNotFound(cube_id) => write!(f, "Cube not found: {}", cube_id),
            DatabaseError::Other(message) => write!(f, "Other error: {}", message),
            DatabaseError::IoError(error) => write!(f, "IO error: {}", error),
            DatabaseError::HashingError(message) => write!(f, "Hashing error: {}", message),
            DatabaseError::UserNotFound => write!(f, "User not found"),
            DatabaseError::AuthenticationFailed => write!(f, "Authentication failed"),
            DatabaseError::LockFailed => write!(f, "Failed to acquire lock"),

        }
    }
}

impl StdError for DatabaseError {}

impl File {
    pub fn new(filename: String, beeid: String, data: Vec<u8>, previous_beecell_id: String, next_beecell_id: String) -> Self {
        File {
            filename,
            beeid,
            data,
            previous_beecell_id,
            next_beecell_id,
        }
    }
}

impl Hash for File {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.filename.hash(state);
        self.data.hash(state);
        self.previous_beecell_id.hash(state);
        self.next_beecell_id.hash(state);
    }
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename &&
            self.data == other.data &&
            self.previous_beecell_id == other.previous_beecell_id &&
            self.next_beecell_id == other.next_beecell_id
    }
}

impl Eq for File {}

impl Database {
    pub fn new(db_path: &str, key: &[u8; KEY_LENGTH], nonce: &[u8; NONCE_LENGTH]) -> io::Result<Self> {
        let config = SeigrConfig::new(db_path, &key, &nonce)?; // assuming `key` and `nonce` are defined in this scope

        Ok(Database {
            cubes: HashMap::new(),
            file_links: HashMap::new(),
            users: config.users, // Load the users from the SeigrConfig
        })
    }

    fn calculate_hash(&self, data: &[u8]) -> String {
        let mut hasher = Blake2b::default();
        hasher.update(data);
        let result: generic_array::GenericArray<u8, U64> = hasher.finalize();

        hex::encode(result.as_slice())
    }

    fn split_into_beecells(&self, data: Vec<u8>) -> Vec<BeeCell> {
        let mut beecells = Vec::new();

        let chunk_size = 150_000_000; // 150MB in bytes
        let chunks = data.chunks(chunk_size);

        for (i, chunk) in chunks.enumerate() {
            let id = format!("beecell{}", i);
            let hash = self.calculate_hash(&chunk);
            let previous_id = if i > 0 { format!("beecell{}", i - 1) } else { String::new() };
            let next_id = format!("beecell{}", i + 1);

            let beecell = BeeCell {
                id,
                data: chunk.to_vec(),
                hash,
                previous_id,
                next_id,
            };

            beecells.push(beecell);
        }

        beecells
    }

    fn group_into_frames(&self, beecells: Vec<BeeCell>) -> Vec<Frame> {
        let mut frames = Vec::new();

        let cells_per_frame = 100;
        let chunks = beecells.chunks(cells_per_frame);

        for (i, chunk) in chunks.enumerate() {
            let id = format!("frame{}", i);
            let previous_id = if i > 0 { format!("frame{}", i - 1) } else { String::new() };
            let next_id = format!("frame{}", i + 1);

            let frame = Frame {
                id,
                beecells: chunk.to_vec(),
                previous_id,
                next_id,
            };

            frames.push(frame);
        }

        frames
    }

    fn group_into_cube(&self, frames: Vec<Frame>) -> Cube {
        let id = format!("cube{}", 0); // Assuming one cube for simplicity

        let cube = Cube {
            id,
            frames,
        };

        cube
    }

    pub fn store_file(&mut self, filename: String, data: Vec<u8>) -> std::io::Result<()> {
        let beecells = self.split_into_beecells(data);
        let frames = self.group_into_frames(beecells);
        let cube = self.group_into_cube(frames);
    
        // Store the cube in the cubes HashMap
        let cube_id = cube.id.clone();
        self.cubes.insert(cube_id.clone(), cube);
    
        // Store the file link
        self.file_links.insert(filename, vec![cube_id]);
    
        Ok(())
    }

    pub fn retrieve_file(&self, filename: &str) -> Result<Vec<u8>, DatabaseError> {
        let cube_ids = self.file_links.get(filename).ok_or(DatabaseError::FileNotFound(filename.to_string()))?;
    
        let mut data = Vec::new();
        for cube_id in cube_ids {
            let cube = self.cubes.get(cube_id).ok_or(DatabaseError::CubeNotFound(cube_id.to_string()))?;
            for frame in &cube.frames {
                for beecell in &frame.beecells {
                    data.extend(&beecell.data);
                }
            }
        }
    
        Ok(data)
    }

    pub fn get_file_links(&self, filename: &str) -> Option<&Vec<String>> {
        self.file_links.get(filename)
    }

    pub fn slice_file(&self, filename: &str, start: usize, end: usize) -> Result<Vec<u8>, DatabaseError> {
        let file_links = self.get_file_links(filename).ok_or(DatabaseError::FileNotFound(filename.to_string()))?;
    
        let mut sliced_file = Vec::new();
    
        // Add previous cells
        for link in file_links.iter().take(start) {
            let cell = self.retrieve_file(link)?;
            sliced_file.extend(cell);
        }
    
        // Add current cell
        let cell = self.retrieve_file(filename)?;
        sliced_file.extend(cell);
    
        // Add next cells
        for link in file_links.iter().skip(end) {
            let cell = self.retrieve_file(link)?;
            sliced_file.extend(cell);
        }
    
        Ok(sliced_file)
    }

    pub fn previous_beecell_id(beecell_id: &str) -> String {
        let mut hasher = DefaultHasher::new();
        beecell_id.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn next_beecell_id(beecell_id: &str) -> String {
        let mut hasher = DefaultHasher::new();
        beecell_id.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn beecell_id(data: &[u8]) -> String {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn beecell_id_from_previous(previous_beecell_id: &str, data: &[u8]) -> String {
        let mut hasher = DefaultHasher::new();
        previous_beecell_id.hash(&mut hasher);
        data.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn beecell_id_from_next(next_beecell_id: &str, data: &[u8]) -> String {
        let mut hasher = DefaultHasher::new();
        next_beecell_id.hash(&mut hasher);
        data.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn beecell_id_from_previous_and_next(previous_beecell_id: &str, next_beecell_id: &str, data: &[u8]) -> String {
        let mut hasher = DefaultHasher::new();
        previous_beecell_id.hash(&mut hasher);
        next_beecell_id.hash(&mut hasher);
        data.hash(&mut hasher);
        hasher.finish().to_string()
    }

    pub fn delete_file(&mut self, filename: String) -> Result<(), DatabaseError> {
        // Remove the file from the database
        match self.file_links.remove(&filename) {
            Some(_) => Ok(()),
            None => Err(DatabaseError::FileNotFound(format!("File {} not found", filename))),
        }
    }

    fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
        hash(password, DEFAULT_COST)
    }

    pub fn register_user(&mut self, username: String, email: String, password: String, beeid: String, key: [u8; 32]) -> Result<(), DatabaseError> {
        // Hash the password
        let password_hash = Self::hash_password(&password)?;
        // Create the User struct
        let user = User { 
            username: username.clone(), 
            email: email.clone(), 
            password: password.clone(), 
            beeid: beeid.clone(), 
            authenticate: false, 
            key: key, 
            password_hash: password_hash 
        };
        // Add the user to the users HashMap
        self.users.insert(username, user);
        Ok(())
    }

    pub fn authenticate_user(&self, username: &str, password: &str) -> Result<(), DatabaseError> {
        // Look up the user in the users HashMap
        match self.users.get(username) {
            Some(user) => {
                // Hash the entered password
                let entered_password_hash = Self::hash_password(password)?;
                // Compare the hash of the entered password to the stored hash
                if user.password_hash == entered_password_hash {
                    Ok(())
                } else {
                    Err(DatabaseError::AuthenticationFailed)
                }
            }
            None => Err(DatabaseError::UserNotFound),
        }
    }

    pub fn add_user(&mut self, user: User) -> Result<(), DatabaseError> {
        // Add the user to the users HashMap
        self.users.insert(user.username.clone(), user);
        Ok(())
    }

    pub fn get_user(&self, username: String) -> Result<User, DatabaseError> {
        // Look up the user in the users HashMap
        match self.users.get(&username) {
            Some(user) => Ok(user.clone()),
            None => Err(DatabaseError::UserNotFound),
        }
    }

    pub fn user_exists(&self, username: &str) -> Result<bool, DatabaseError> {
        // Look up the user in the users HashMap
        match self.users.get(username) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
}