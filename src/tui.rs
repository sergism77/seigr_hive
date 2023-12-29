use std::{io, panic};
use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::app::{App, AppState};
use crate::seigrconfig::SeigrConfig;
use crate::user::User;
use crate::eventhandler::EventHandler;
use crate::ui;
use crate::ui::{render_login, render_register};

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
pub struct Tui {
    /// Interface to the Terminal.
    terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>,
    /// Terminal event handler.
    pub events: EventHandler,
    /// The configuration.
    pub config: SeigrConfig,
}

impl Tui {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>, events: EventHandler, config: SeigrConfig) -> Result<Self, std::io::Error> {
        Ok(Self { terminal, events, config })
    }


    /// [`Draw`] the terminal interface by [`rendering`] the widgets.
    ///
    /// [`Draw`]: tui::Terminal::draw
    /// [`rendering`]: crate::ui::render
    pub fn draw(&mut self, app: &mut App) -> Result<(String, String), Box<dyn std::error::Error>> {
        let mut input_username = String::new();
        let mut input_password = String::new();
    
        self.terminal.draw(|frame| {
            match app.state {
                AppState::Login => {
                    if let Ok((username, password)) = ui::render_login(app, frame) {
                        input_username = username;
                        input_password = password;
                    } else if let Err(e) = ui::render_login(app, frame) {
                        // Handle error here
                        eprintln!("Error rendering login: {}", e);
                    }
                },
                AppState::Register => {
                    if let Ok((username, password)) = ui::render_register(app, frame) {
                        input_username = username;
                        input_password = password;
                    } else if let Err(e) = ui::render_register(app, frame) {
                        // Handle error here
                        eprintln!("Error rendering register: {}", e);
                    }
                },
                // Add more states as needed
            }
        }).map_err(|e| {
            // Handle error here
            eprintln!("Error drawing terminal: {}", e);
            e
        })?;
    
        Ok((input_username, input_password))
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode and sets terminal properties.
    pub fn enter(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(
            io::stderr(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    /// Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(
            io::stderr(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode and reverts back the terminal properties.
    pub fn exit(&mut self) -> Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn draw_login(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| {
            if let Err(e) = ui::render_login(app, frame) {
                // Handle error here
                eprintln!("Error rendering login: {}", e);
            }
        })?;
        Ok(())
    }
    
    pub fn draw_register(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| {
            if let Err(e) = ui::render_register(app, frame) {
                // Handle error here
                eprintln!("Error rendering register: {}", e);
            }
        })?;
        Ok(())
    }
}
