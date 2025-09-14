use rogue_config::{ConfigError, OptionsProvider, YamlOptionsProvider};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ScrapeOptions {
    pub name: String,
    pub url: String,
}

impl ScrapeOptions {
    pub(crate) fn get() -> Result<Self, ConfigError> {
        YamlOptionsProvider::get()
    }
}
