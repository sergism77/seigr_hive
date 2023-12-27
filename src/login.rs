use std::path::Path;
use crate::seigrconfig::SeigrConfig;
use crate::user::User;

use crate::{tui::Tui, app::App};

pub fn register_user(username: String, password: String, email: String) -> Result<User, std::io::Error> {
    let mut config = SeigrConfig::from_file()?;
    let user = config.add_user(username, password, email)?;
    config.save_to_file()?;
    Ok(user)
}

pub fn authenticate_user(username: String, password: String) -> Result<bool, std::io::Error> {
  let config = SeigrConfig::from_file()?;
  let user_result = config.get_user(username);

  match user_result {
      Ok(user) => {
          let authenticated = user.authenticate(&password);
          Ok(authenticated)
      },
      Err(e) => Err(e),
  }
}

pub fn config_exists() -> bool {
    Path::new("seigrconfig.toml").exists()
}