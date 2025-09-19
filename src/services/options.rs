use crate::prelude::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AppOptions {
    pub cache_dir: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
    pub server_base: Option<Url>,
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
