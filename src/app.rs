use crate::login;
use crate::user::User;

#[derive(Debug, Clone)]
pub enum AppState {
    Login,
    Register,
    // Add more states as needed
}

impl Default for AppState {
    fn default() -> Self {
        Self::Login
    }
}

/// Application.
#[derive(Debug, Default, Clone)]
pub struct App {
    /// should the application exit?
    pub should_quit: bool,
    /// the current user's username
    pub username: Option<String>,
    /// the current user's password
    pub password: Option<String>,
    /// the current user
    pub user: Option<User>,
    /// is the user authenticated?
    pub authenticated: bool,
    /// the current state of the application
    pub state: AppState,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            should_quit: false,
            username: None,
            password: None,
            user: None,
            authenticated: false,
            state: AppState::Login, // Set the initial state to Login
        }
    }


    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set should_quit to true to quit the application.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Register a new user.
    pub fn register_user(&mut self, username: String, password: String, email: String) -> Result<(), std::io::Error> {
        let user = login::register_user(username.clone(), password.clone(), email.clone())?;
        self.user = Some(user);
        self.username = Some(username);
        self.password = Some(password);
        Ok(())
    }

    /// Authenticate a user.
    pub fn authenticate_user(&mut self, username: String, password: String) -> Result<(), std::io::Error> {
        let authenticated = login::authenticate_user(username, password)?;
        self.authenticated = authenticated;
        Ok(())
    }

    /// Get the current state of the application.
    pub fn get_state(&self) -> AppState {
        self.state.clone()
    }

    /// Set the current state of the application.
    pub fn set_state(&mut self, state: AppState) {
        self.state = state;
    }

    /// Get the current user.
    pub fn get_user(&self) -> Option<User> {
        self.user.clone()
    }

    /// Set the current user.
    pub fn set_user(&mut self, user: User) {
        self.user = Some(user);
    }

    /// Get the current user's email.
    pub fn get_email(&self) -> Option<String> {
        self.user.as_ref().map(|user| user.email.clone())
    }

    

}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_app_register_user() {
        let mut app = App::default();
        let result = app.register_user("username".to_string(), "password".to_string(), "email@example.com".to_string());
        assert!(result.is_ok());
        assert!(app.username.is_some());
    }

    #[test]
    fn test_app_authenticate_user() {
        let mut app = App::default();
        let result = app.authenticate_user("username".to_string(), "password".to_string());
        assert!(result.is_ok());
        assert!(app.authenticated);
    }
}