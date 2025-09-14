use rogue_config::{ConfigError, OptionsProvider, YamlOptionsProvider};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ScrapeOptions {
    pub name: String,
    pub url: String,
    pub cache_dir: PathBuf,
}

impl ScrapeOptions {
    pub(crate) fn set_defaults(&mut self) {
        if self.cache_dir == PathBuf::new() {
            self.cache_dir = "cache/html".into();
        }
    }

    pub(crate) fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = vec![];
        if self.name.is_empty() {
            errors.push(ValidationError::Required("name".to_owned()));
        }
        if self.url.is_empty() {
            errors.push(ValidationError::Required("url".to_owned()));
        }
        if self.cache_dir == PathBuf::new() {
            errors.push(ValidationError::Required("cache_dir".to_owned()));
        } else {
            let cache_dir = PathBuf::from(&self.cache_dir);
            if !cache_dir.exists() {
                errors.push(ValidationError::PathNotExist(
                    "cache_dir".to_owned(),
                    cache_dir,
                ));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub(crate) fn get() -> Result<Self, ConfigError> {
        let mut options: ScrapeOptions = YamlOptionsProvider::get()?;
        options.set_defaults();
        Ok(options)
    }
}

#[derive(Debug)]
pub enum ValidationError {
    Required(String),
    PathNotExist(String, PathBuf),
}

impl Display for ValidationError {
    #[allow(clippy::absolute_paths)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::Required(field) => {
                write!(f, "Required field `{field}` is empty")
            }
            ValidationError::PathNotExist(field, path) => {
                write!(f, "Path field `{field}` does not exist: {}", path.display())
            }
        }
    }
}
