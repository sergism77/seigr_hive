use seigr_hive::app;
use seigr_hive::login;
use seigr_hive::ui;
use seigr_hive::tui;
use seigr_hive::user;
use seigr_hive::seigrconfig;
use seigr_hive::eventhandler;
/// Result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use std::path::Path;
use std::fs;
use app::{App, AppState};
use login::{ authenticate_user, config_exists};
use seigr_hive::{seigrconfig::generate_key, database::{Database, DatabaseError}};
use seigrconfig::SeigrConfig;
use ratatui::{backend::{CrosstermBackend, Backend}, Terminal, Frame};
use tokio::io;
use tui::Tui;
use eventhandler::EventHandler;
use seigr_hive::ui::{render_login, render_register};
use std::sync::{Arc, Mutex};
use rayon::ThreadPoolBuilder;

// Show a welcoming message when the application starts.
pub fn welcome() {
    println!("Welcome to Seigr Hive!");
}

// Show a goodbye message when the application exits.
pub fn goodbye() {
    println!("Goodbye!");
}
use tokio::io::{BufReader, AsyncBufReadExt};

use crate::user::User;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a thread pool
    let pool = ThreadPoolBuilder::new().num_threads(4).build().expect("Failed to create thread pool");

    // Wrap the Database in an Arc<Mutex<>> so it can be shared between threads
    let database = Arc::new(Mutex::new(Database::new()));

    // Handle a request
    pool.install(|| {
        // Lock the database
        let mut database = match database.lock() {
            Ok(database) => database,
            Err(_) => return, // Handle the error appropriately
        };

        // Use the database
        let data = match database.retrieve_file("example.txt") {
            Ok(data) => data,
            Err(_) => return, // Handle the error appropriately
        };
    });
    
    // Load or generate a key
    let key = if Path::new("keyfile").exists() {
        let mut array = [0; 32];
        let vec = fs::read("keyfile").expect("Failed to read key");
        array.copy_from_slice(&vec);
        array
    } else {
        let key = generate_key().expect("Failed to generate key");
        fs::write("keyfile", &key).expect("Failed to write key");
        key
    };

    // Show a welcoming message
    welcome();

    println!("Before creating SeigrConfig");
    
    // Load or create the SeigrConfig
    let mut config = SeigrConfig::new(&key).expect("Failed to load or create config");
    config.load_users()?;

    println!("After creating SeigrConfig");

    // Create a new Tui instance
    let mut tui = Tui::new(Terminal::new(CrosstermBackend::new(std::io::stderr()))?, EventHandler::new(), config.clone())?;

    // Create a new App instance
    let mut app = App::default();

    // Define the reader before the if statement
    let mut reader = BufReader::new(io::stdin()).lines();

    let (username, email, password) = if !config.has_users() {
        // If the config does not have any users, prompt for registration
        println!("No user found. Please register.");
        let user_input = get_user_input()?;
        // Add the new user to the database
        let user = User::new(user_input.0.clone(), user_input.1.clone(), user_input.2.clone())?;
        let mut database_guard = database.lock().map_err(|_| DatabaseError::LockFailed)?;
        database_guard.add_user(user)?;
        user_input
    } else {
        // If the config has users, get the username from the user input
        println!("Please enter your username:");
        let username = reader.next_line().await?.ok_or(io::Error::new(io::ErrorKind::InvalidInput, "No input provided"))?;
    
        // Retrieve the user from the database
        let database_guard = database.lock().map_err(|_| DatabaseError::LockFailed)?;
        let user = (*database_guard).get_user(username.clone())?;
        (username, user.email.clone(), user.password.clone())
    };
    
    app.set_state(AppState::Login);
    app.set_user(User::new(username.clone(), password.to_string(), email.to_string())?);

    // Check if the user exists in the database
    let database_guard = database.lock().map_err(|_| DatabaseError::LockFailed)?;
    let user_exists = (*database_guard).user_exists(&username)?;
    if !user_exists {
        // Now you can call draw_register on your Tui instance
        tui.draw_register(&mut app)?;
    } else {
        // If the user exists, render the login form
        tui.draw_login(&mut app)?;
    }

    // Save the config to file
    config.save_config()?;

    Ok(())
}

pub fn get_user_input() -> Result<(String, String, String)> {
    // Get the username, email, and password from the user
    println!("Please enter your username:");
    let mut username = String::new();
    std::io::stdin().read_line(&mut username)?;
    println!("Please enter your email:");
    let mut email = String::new();
    std::io::stdin().read_line(&mut email)?;
    println!("Please enter your password:");
    let mut password = String::new();
    std::io::stdin().read_line(&mut password)?;
    Ok((username, email, password))
}