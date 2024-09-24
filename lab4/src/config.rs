use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub field: Field,
    pub food: usize,
    pub delay: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            field: Field {
                width: 40,
                height: 30,
            },

            food: 1,
            delay: 1000,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Field {
    pub width: usize,
    pub height: usize,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Option<Self> {
        let config = std::fs::read_to_string(path).ok()?;
        let config = toml::from_str(&config).ok()?;

        Some(config)
    }
}
