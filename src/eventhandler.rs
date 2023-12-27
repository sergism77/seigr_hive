use crossterm::event::{Event, KeyCode};
use crate::app::{App, AppState};

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_event(&mut self, app: &mut App, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            Event::Key(event) => self.handle_key_event(app, event.code),
            Event::FocusGained => todo!(),
            Event::FocusLost => todo!(),
            Event::Mouse(_) => todo!(),
            Event::Paste(_) => todo!(),
            Event::Resize(_, _) => todo!(),
            // Handle other types of events...
        }
    }

    fn handle_key_event(&mut self, app: &mut App, key_code: KeyCode) -> Result<(), Box<dyn std::error::Error>> {
        match key_code {
            KeyCode::Char('q') => {
                app.should_quit = true;
            },
            KeyCode::Char('r') => {
                app.state = AppState::Register;
            },
            KeyCode::Char('l') => {
                app.state = AppState::Login;
            },KeyCode::Char(_) => {
                // Handle other characters
            },
            KeyCode::Backspace => todo!(),
            KeyCode::Enter => todo!(),
            KeyCode::Left => todo!(),
            KeyCode::Right => todo!(),
            KeyCode::Up => todo!(),
            KeyCode::Down => todo!(),
            KeyCode::Home => todo!(),
            KeyCode::End => todo!(),
            KeyCode::PageUp => todo!(),
            KeyCode::PageDown => todo!(),
            KeyCode::Tab => todo!(),
            KeyCode::BackTab => todo!(),
            KeyCode::Delete => todo!(),
            KeyCode::Insert => todo!(),
            KeyCode::F(_) => todo!(),
            KeyCode::Null => todo!(),
            KeyCode::Esc => todo!(),
            KeyCode::CapsLock => todo!(),
            KeyCode::ScrollLock => todo!(),
            KeyCode::NumLock => todo!(),
            KeyCode::PrintScreen => todo!(),
            KeyCode::Pause => todo!(),
            KeyCode::Menu => todo!(),
            KeyCode::KeypadBegin => todo!(),
            KeyCode::Media(_) => todo!(),
            KeyCode::Modifier(_) => todo!(),
            // Handle other key codes...
        }
        Ok(())
    }

    // Add more event handling methods as needed...
}