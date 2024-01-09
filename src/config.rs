use anyhow::Result;
use config::{Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// path to the Kobo sqlite DB
    pub db_path: Option<PathBuf>,
}

#[derive(thiserror::Error, Debug)]
enum ConfigError {
    #[error("No DB path provided")]
    NoDBProvidedError,
}

impl Config {
    pub fn new(path: PathBuf) -> Result<Self, config::ConfigError> {
        // TODO: figure out how to propagate error up
        let s = config::Config::builder()
            .add_source(
                File::with_name(path.into_os_string().into_string().unwrap().as_str())
                    .required(false),
            )
            .add_source(Environment::with_prefix("koldpress"))
            .build()?;
        s.try_deserialize()
    }

    pub fn validate(&self) -> Result<()> {
        if self.db_path.is_none() {
            return Err(ConfigError::NoDBProvidedError)?;
        }
        Ok(())
    }
}
