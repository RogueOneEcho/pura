use crate::prelude::*;
use serde_json::Value;

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
        get_ip(&http).await.map_err(ServiceError::Ip)?;
        let podcasts = PodcastProvider::new(paths.get_podcast_dir());
        Ok(Self {
            options,
            paths,
            http,
            podcasts,
        })
    }
}

async fn get_ip(http: &HttpClient) -> Result<(), HttpError> {
    let ip_url = Url::parse("https://ipinfo.io").expect("URL should be valid");
    http.remove(&ip_url, Some("json")).await;
    let ip: Value = http.get_json(&ip_url).await?;
    debug!(
        "External IP: {} ({}, {})",
        ip.get("ip").unwrap_or_default(),
        ip.get("city").unwrap_or_default(),
        ip.get("country").unwrap_or_default()
    );
    Ok(())
}

#[derive(Debug)]
pub enum ServiceError {
    GetConfig(ConfigError),
    ValidateConfig(Vec<ValidationError>),
    Ip(HttpError),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let (action, reason) = match self {
            ServiceError::GetConfig(e) => ("read config", e.to_string()),
            ServiceError::ValidateConfig(errors) => {
                let reason = errors.iter().fold(String::new(), |mut acc, err| {
                    acc.push('\n');
                    acc.push_str(&err.to_string());
                    acc
                });
                ("validate config", reason)
            }
            ServiceError::Ip(e) => ("determine IP", e.to_string()),
        };
        write!(f, "{} to {action}\n{reason}", "Failed".bold())
    }
}
