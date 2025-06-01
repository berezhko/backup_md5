use serde::Deserialize;
use anyhow::{Context, Result};
use std::fs;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub extensions: HashSet<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))?;

        Ok(config)
    }
}