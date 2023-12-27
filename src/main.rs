/// Application.
pub mod app;

/// Terminal events handler.
pub mod login;

/// Widget renderer.
pub mod ui;

/// Terminal user interface.
pub mod tui;

/// User.
pub mod user;

/// SeigrConfig.
pub mod seigrconfig;

/// Event handler.
pub mod eventhandler;

/// Result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use tokio::fs;
use toml;
use app::App;
use login::{ authenticate_user, config_exists };
use ring::hmac::Key;
use seigrconfig::{SeigrConfig, generate_key};
use ratatui::{backend::{CrosstermBackend, Backend}, Terminal};
use tokio::io;
use tui::Tui;
use eventhandler::EventHandler;
use tokio::io::{BufReader, AsyncBufReadExt};

// Show a welcoming message when the application starts.
pub fn welcome() {
    println!("Welcome to Seigr Hive!");
}

// Show a goodbye message when the application exits.
pub fn goodbye() {
    println!("Goodbye!");
}


#[tokio::main]
async fn main() -> Result<()> {
    // Generate a key
    let key = generate_key(); // Replace this with your key generation function

    // Create the SeigrConfig instance
    let config = SeigrConfig::read_config(&key)?;

    let username = "username"; 
    let mut tui = Tui::new(Terminal::new(CrosstermBackend::new(std::io::stderr()))?, EventHandler::new(), config.clone(), username.to_string())?;
    let mut app = App::new();
    tui.enter()?;
    tui.draw(&mut app)?;

    if config_exists() {
        let username = match &app.user {
            Some(user) => user.username(),
            None => "",
        };

        let mut reader = BufReader::new(io::stdin()).lines();

        println!("Welcome: {}!", username);

        // Read the username and password from the user
        println!("Please enter your username:");
        let input_username = match reader.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => return Err("No input received".into()),
            Err(e) => return Err(e.into()),
        };
        println!("Please enter your password:");
        let input_password = match reader.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => return Err("No input received".into()),
            Err(e) => return Err(e.into()),
        };

        // Authenticate the user
        match authenticate_user(input_username, input_password) {
            Ok(true) => {
                println!("Welcome {}!", username);
            },
            Ok(false) => {
                println!("Incorrect username or password.");
            },
            Err(e) => {
                println!("An error occurred: {}", e);
            },
        }
    } else {
        println!("No config file found. Please register.");
    }

    Ok(())
}