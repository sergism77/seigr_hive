use ratatui::{
  prelude::{Alignment, Frame},
  style::{Color, Style},
  widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::app::App;

pub fn render_login(app: &mut App, f: &mut Frame) {
  let (username, password) = match &app.user {
    Some(user) => (user.username(), user.password()),
    None => ("No user logged in", ""),
};

  f.render_widget(
      Paragraph::new(format!(
          "
          Login Form:\n\
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
  )
}

pub fn render_register(app: &mut App, f: &mut Frame) {
  let (username, email, password) = match &app.user {
      Some(user) => (user.username(), user.email(), user.password()),
      None => ("No user logged in", "", ""),
  };
  
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
      ))
      .block(
          Block::default()
              .title("Register")
              .title_alignment(Alignment::Center)
              .borders(Borders::ALL)
              .border_type(BorderType::Rounded),
      )
      .style(Style::default().fg(Color::Yellow))
      .alignment(Alignment::Center),
      f.size(),
  )
}