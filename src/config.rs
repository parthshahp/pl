use serde::Deserialize;
use std::{fs, io};

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct UserConfig {
    pub project_dirs: Vec<String>,
    pub editor_command: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            project_dirs: vec!["~/Projects".to_string()],
            editor_command: "code".to_string(),
        }
    }
}

pub fn load_user_config() -> io::Result<UserConfig> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "config directory not found"))?;

    let config_path = config_dir.join("pl").join("config.toml");

    match fs::read_to_string(&config_path) {
        Ok(raw) => toml::from_str(&raw).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid config at {}: {e}", config_path.display()),
            )
        }),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(UserConfig::default()),
        Err(err) => Err(err),
    }
}
