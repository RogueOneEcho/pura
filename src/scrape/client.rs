use std::fs::create_dir_all;
use crate::prelude::*;
use reqwest::blocking::Client as ReqwestClient;
use sha2::{Digest, Sha256};

pub(crate) struct Client {
    pub cache_dir: PathBuf,
}

impl Client {
    pub fn get(&self, url: &Url) -> Result<String, ClientError> {
        match self.get_from_cache(url) {
            Ok(Some(content)) => {
                trace!("Cache HIT: {url}");
                return Ok(content);
            }
            Ok(None) => {
                trace!("Cache MISS: {url}");
            }
            Err(e) => {
                warn!("{} to read cache for: {url}\n{e}", "Failed".bold());
            }
        }
        self.fetch_and_cache(url)
    }

    fn get_from_cache(&self, url: &Url) -> Result<Option<String>, ClientError> {
        let cache_path = self.get_cache_path(url);
        if !cache_path.exists() {
            return Ok(None);
        }
        let mut file = File::open(cache_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Some(contents))
    }

    fn fetch_and_cache(&self, url: &Url) -> Result<String, ClientError> {
        let client = ReqwestClient::new();
        let response = client.get(url.as_str()).send()?;
        if !response.status().is_success() {
            return Err(ClientError::Status(response.status().as_u16()));
        }
        let content = response.text()?;
        self.cache_content(url, &content)?;
        Ok(content)
    }

    fn cache_content(&self, url: &Url, content: &str) -> Result<(), ClientError> {
        let path = self.get_cache_path(url);
        let dir = path.parent().expect("cache path should have a parent directory");
        if !dir.exists() {
            create_dir_all(dir)?;
        }
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn get_cache_path(&self, url: &Url) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(url.as_str());
        let hash = format!("{:x}", hasher.finalize());
        let domain = url.domain().unwrap_or("unknown");
        let extension = Path::new(url.path())
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("html");
        self.cache_dir
            .join(domain)
            .join(hash)
            .with_extension(extension)
    }
}

#[allow(clippy::absolute_paths)]
#[derive(Debug)]
pub enum ClientError {
    Status(u16),
    Network(reqwest::Error),
    Io(std::io::Error),
    InvalidUrl(String),
}

#[allow(clippy::absolute_paths)]
impl From<reqwest::Error> for ClientError {
    fn from(err: reqwest::Error) -> Self {
        ClientError::Network(err)
    }
}

#[allow(clippy::absolute_paths)]
impl From<std::io::Error> for ClientError {
    fn from(err: std::io::Error) -> Self {
        ClientError::Io(err)
    }
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let message = match self {
            ClientError::Status(number) => StatusCode::from_u16(*number)
                .and_then(|code| {
                    Ok(format!(
                        "{number} {}",
                        code.canonical_reason().unwrap_or_default()
                    ))
                })
                .unwrap_or_else(|_| format!("{number}")),
            ClientError::Network(e) => format!("{e}"),
            ClientError::Io(e) => format!("{e}"),
            ClientError::InvalidUrl(e) => format!("{e}"),
        };
        write!(f, "{message}")
    }
}
