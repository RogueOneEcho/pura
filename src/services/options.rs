use crate::prelude::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AppOptions {
    /// Directory to cache HTTP responses and audio files.
    ///
    /// Default: `cache`
    pub cache_dir: Option<PathBuf>,
    /// Directory to output processed files.
    ///
    /// Default: `output`
    pub output_dir: Option<PathBuf>,
    /// Base URL to use for server.
    ///
    /// Default: None
    pub server_base: Option<Url>,
    /// Expected external IP address.
    ///
    /// Execution will stop if different.
    ///
    /// Default: None
    pub expect_ip: Option<String>,
    /// Expected country geolocation.
    ///
    /// Execution will stop if different.
    ///
    /// Default: None
    pub expect_country: Option<String>,
}

impl AppOptions {
    pub(crate) fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let paths = PathProvider::new(self.clone());
        paths.validate()
    }

    pub(crate) fn get() -> Result<Self, ConfigError> {
        EnvironmentOptionsProvider::get()
    }
}
