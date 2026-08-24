use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const CONFIG_PATH: &str = ".hblank/config.toml";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub fixtures: Vec<String>,
    pub ignore: Vec<String>,
    pub window: WindowConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fixtures: vec!["src/**/*.hblank.rs".to_owned()],
            ignore: vec!["target/**".to_owned(), ".hblank/**".to_owned()],
            window: WindowConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Hblank".to_owned(),
            width: 1440,
            height: 900,
        }
    }
}

impl Config {
    /// Creates the conventional configuration for a Rust package.
    #[must_use]
    pub fn for_project(package_name: &str) -> Self {
        let mut config = Self::default();
        config.window.title = format!("{package_name} · Hblank");
        config
    }

    /// Loads and validates configuration from the project's Hblank directory.
    ///
    /// # Errors
    /// Returns an error when the file cannot be read, parsed, or validated.
    pub fn load(project_root: &Path) -> Result<Self, ConfigError> {
        let path = project_root.join(CONFIG_PATH);
        let source = fs::read_to_string(&path).map_err(|source| ConfigError::Read {
            path: path.clone(),
            source,
        })?;
        let config = toml::from_str::<Self>(&source).map_err(|source| ConfigError::Parse {
            path: path.clone(),
            source,
        })?;
        config.validate()?;
        Ok(config)
    }

    /// Serializes validated configuration as TOML.
    ///
    /// # Errors
    /// Returns an error when configuration is invalid or serialization fails.
    pub fn to_toml(&self) -> Result<String, ConfigError> {
        self.validate()?;
        toml::to_string_pretty(self).map_err(ConfigError::Serialize)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.fixtures.is_empty() || self.fixtures.iter().any(String::is_empty) {
            return Err(ConfigError::NoFixtureFilePatterns);
        }
        if self.window.title.trim().is_empty() {
            return Err(ConfigError::EmptyWindowTitle);
        }
        if self.window.width == 0 || self.window.height == 0 {
            return Err(ConfigError::InvalidWindowSize {
                width: self.window.width,
                height: self.window.height,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not read Hblank config at {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not parse Hblank config at {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("could not serialize Hblank config: {0}")]
    Serialize(toml::ser::Error),
    #[error("Hblank config must include at least one non-empty fixture file pattern")]
    NoFixtureFilePatterns,
    #[error("Hblank window title cannot be empty")]
    EmptyWindowTitle,
    #[error("Hblank window size must be positive, received {width}x{height}")]
    InvalidWindowSize { width: u32, height: u32 },
}
