use std::{collections::HashMap, fs, io, path::Path};

use serde::Deserialize;
use thiserror::Error;

use crate::path::CWDPattern;

#[derive(Deserialize, Default, Debug)]
pub struct Config {
    // Aliases are applied sequentially, each alias must rely on
    // the path already having previous aliases applied
    pub aliases: Option<HashMap<String, CWDPattern>>,
}

impl Config {
    pub fn from_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

// #[derive(Error, Debug)]
// pub enum LoadConfigError {
//     #[error("failed to read file")]
//     Io(#[from] io::Error),
//     #[error("TOML deserialization failed")]
//     TomlDeserialization(#[from] toml::de::Error),
// }

// pub fn load_config(path: impl AsRef<Path>) -> Result<Config, LoadConfigError> {
//     let config_str = fs::read_to_string(path)?;
//     let config = toml::from_str(&config_str)?;
//     Ok(config)
// }
