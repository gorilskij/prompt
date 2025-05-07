use std::collections::HashMap;

use serde::Deserialize;

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
