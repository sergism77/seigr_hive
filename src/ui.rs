use ratatui::{
  prelude::{Alignment, Frame},
  style::{Color, Style},
  widgets::{Block, BorderType, Borders, Paragraph},
  backend::{Backend, CrosstermBackend}, Terminal,
};

use crate::app::App;

pub fn render_login(app: &App, f: &mut Frame) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    let mut frame = terminal.get_frame();
    let username = app.username.clone().unwrap_or("No user logged in".to_string());
    let password = app.password.clone().unwrap_or("".to_string());

  f.render_widget(
      Paragraph::new(format!(
          "
          
          Username: {}\n\
          Password: {}\n\
          Press `Enter` to submit.\n\
          Press `Esc`, `Ctrl-C` or `q` to stop running.
          ",
          username, password
      ))
      .block(
          Block::default()
              .title("Login")
              .title_alignment(Alignment::Center)
              .borders(Borders::ALL)
              .border_type(BorderType::Rounded),
      )
      .style(Style::default().fg(Color::Yellow))
      .alignment(Alignment::Center),
      f.size(),
  );

  Ok((username, password))
}

pub fn render_register(app: &App, f: &mut Frame) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    let mut frame = terminal.get_frame();
    let username = app.username.clone().unwrap_or("No user logged in".to_string());
    let password = app.password.clone().unwrap_or("".to_string());
    let email = app.user.as_ref().and_then(|user| Some(user.get_email().clone())).unwrap_or("");

    f.render_widget(
        Paragraph::new(format!(
            "
            Registration Form:\n\
            Username: {}\n\
            Email: {}\n\
            Password: {}\n\
            Press `Enter` to submit.\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.
            ",
            username, email, password
        )),
        f.size(),
    );

    Ok((username, password))
}
