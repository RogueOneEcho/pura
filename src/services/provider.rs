use crate::prelude::*;

pub struct ServiceProvider {
    pub options: AppOptions,
    pub paths: PathProvider,
    pub http: HttpClient,
    pub podcasts: PodcastProvider,
}

impl ServiceProvider {
    pub async fn create() -> Result<ServiceProvider, ServiceError> {
        let options = AppOptions::get().map_err(ServiceError::GetConfig)?;
        options.validate().map_err(ServiceError::ValidateConfig)?;
        let paths = PathProvider::new(options.clone());
        let http = HttpClient::new(paths.get_http_dir());
        let ip = IpInfoProvider::new(options.clone(), http.clone());
        ip.validate().await.map_err(ServiceError::ValidateIp)?;
        let podcasts = PodcastProvider::new(paths.get_podcast_dir());
        Ok(Self {
            options,
            paths,
            http,
            podcasts,
        })
    }
}

#[derive(Debug)]
pub enum ServiceError {
    GetConfig(ConfigError),
    ValidateConfig(Vec<ValidationError>),
    ValidateIp(Vec<ValidationError>),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let action = match self {
            ServiceError::GetConfig(_) => "read config",
            ServiceError::ValidateConfig(_) => "validate config",
            ServiceError::ValidateIp(_) => "validate IP",
        };
        let reason = match self {
            ServiceError::GetConfig(e) => e.to_string(),
            ServiceError::ValidateConfig(errors) | ServiceError::ValidateIp(errors) => errors.log(),
        };
        write!(f, "{} to {action}\n{reason}", "Failed".bold())
    }
}
