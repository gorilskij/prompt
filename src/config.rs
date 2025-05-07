use std::{collections::HashMap, fs, path::Path};

use serde::Deserialize;

use crate::path::CWDPattern;

#[derive(Deserialize, Default, Debug)]
pub struct Config {
    // Aliases are applied sequentially, each alias must rely on
    // the path already having previous aliases applied
    pub aliases: Option<HashMap<String, CWDPattern>>,
}

pub fn load_config(path: impl AsRef<Path>) -> Config {
    let Ok(config_str) = fs::read_to_string(path) else {
        return Default::default();
    };
    let Ok(config) = toml::from_str(&config_str) else {
        return Default::default();
    };
    config
}
