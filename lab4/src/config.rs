use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub field: Field,
    pub food: usize,
    pub delay: usize,
}

#[derive(Deserialize)]
pub struct Field {
    pub width: usize,
    pub height: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            field: Field {
                width: 30,
                height: 40,
            },

            food: 1,
            delay: 1000,
        }
    }
}

impl Config {
    pub fn load<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let Ok(config) = std::fs::read_to_string(path) else {
            return Default::default();
        };

        let Ok(config) = toml::from_str(&config) else {
            return Default::default();
        };

        config
    }
}
