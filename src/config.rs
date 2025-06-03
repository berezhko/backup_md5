use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "deserialize_lowercase_hashset")]
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

fn deserialize_lowercase_hashset<'de, D>(deserializer: D) -> Result<HashSet<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let original: HashSet<String> = HashSet::deserialize(deserializer)?;
    let lowercase = original.into_iter().map(|s| s.to_lowercase()).collect();
    Ok(lowercase)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_valid_config() {
        let mut config_file = NamedTempFile::new().unwrap();
        write!(
            config_file,
            r#"
            extensions = ["txt", "jpg", "pdf"]
        "#
        )
        .unwrap();

        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();

        assert_eq!(config.extensions.len(), 3);
        assert!(config.extensions.contains("txt"));
        assert!(config.extensions.contains("jpg"));
        assert!(config.extensions.contains("pdf"));
    }

    #[test]
    fn test_case_insensitive_extensions() {
        let mut config_file = NamedTempFile::new().unwrap();
        write!(
            config_file,
            r#"
            extensions = ["TXT", "JPG", "PDF"]
        "#
        )
        .unwrap();

        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();

        assert!(config.extensions.contains("txt"));
        assert!(config.extensions.contains("jpg"));
        assert!(config.extensions.contains("pdf"));
    }

    #[test]
    fn test_empty_extensions() {
        let mut config_file = NamedTempFile::new().unwrap();
        write!(
            config_file,
            r#"
            extensions = []
        "#
        )
        .unwrap();

        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        assert!(config.extensions.is_empty());
    }

    #[test]
    fn test_duplicate_extensions() {
        let mut config_file = NamedTempFile::new().unwrap();
        write!(
            config_file,
            r#"
            extensions = ["txt", "TXT", "txt"]
        "#
        )
        .unwrap();

        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.extensions.len(), 1);
        assert!(config.extensions.contains("txt"));
    }

    #[test]
    fn test_missing_config_file() {
        let result = Config::from_file("nonexistent_file.toml");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read config file"));
    }

    #[test]
    fn test_invalid_toml_syntax() {
        let mut config_file = NamedTempFile::new().unwrap();
        write!(config_file, "invalid toml content").unwrap();

        let result = Config::from_file(config_file.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to parse config file"));
    }

    #[test]
    fn test_missing_extensions_field() {
        let mut config_file = NamedTempFile::new().unwrap();
        write!(
            config_file,
            r#"
            [other_section]
            key = "value"
        "#
        )
        .unwrap();

        let result = Config::from_file(config_file.path().to_str().unwrap());
        assert!(result.is_err());
    }

    #[ignore]
    #[test]
    fn test_whitespace_in_extensions() {
        let mut config_file = NamedTempFile::new().unwrap();
        write!(
            config_file,
            r#"
            extensions = [" txt ", " jpg ", " pdf "]
        "#
        )
        .unwrap();

        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        assert!(config.extensions.contains("txt"));
        assert!(config.extensions.contains("jpg"));
        assert!(config.extensions.contains("pdf"));
    }

    #[test]
    fn test_commented_extensions() {
        let mut config_file = NamedTempFile::new().unwrap();
        write!(
            config_file,
            r#"
            extensions = [
                "txt",  # Text files
                "jpg",  # JPEG images
                # "tmp",  # Temporary files (commented out)
                "pdf"   # PDF documents
            ]
        "#
        )
        .unwrap();

        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.extensions.len(), 3);
        assert!(!config.extensions.contains("tmp"));
    }

    #[test]
    fn test_multiline_config() {
        let mut config_file = NamedTempFile::new().unwrap();
        write!(
            config_file,
            r#"
            extensions = [
                "txt",
                "jpg",
                "pdf",
                "docx",
                "xlsx"
            ]
        "#
        )
        .unwrap();

        let config = Config::from_file(config_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.extensions.len(), 5);
    }
}
