use crate::prelude::*;
use reqwest::blocking::Client as ReqwestClient;
use sha2::{Digest, Sha256};

pub(crate) struct Client {
    pub cache_dir: PathBuf,
}

impl Client {
    pub fn get(&self, url: &str) -> Result<String, ClientError> {
        match self.get_from_cache(url) {
            Ok(Some(content)) => {
                return Ok(content);
            }
            Ok(None) => {}
            Err(e) => {
                error!("Error fetching from cache: {e:?}");
            }
        }
        self.fetch_and_cache(url)
    }

    fn get_from_cache(&self, url: &str) -> Result<Option<String>, ClientError> {
        let cache_path = self.get_cache_path(url);
        if !cache_path.exists() {
            return Ok(None);
        }
        let mut file = File::open(cache_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Some(contents))
    }

    fn fetch_and_cache(&self, url: &str) -> Result<String, ClientError> {
        let client = ReqwestClient::new();
        let response = client.get(url).send()?;
        if !response.status().is_success() {
            return Err(ClientError::Status(response.status().as_u16()));
        }
        let content = response.text()?;
        self.cache_content(url, &content)?;
        Ok(content)
    }

    fn cache_content(&self, url: &str, content: &str) -> Result<(), ClientError> {
        let cache_path = self.get_cache_path(url);
        let mut file = File::create(cache_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    fn get_cache_path(&self, url: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        self.cache_dir.join(format!("{hash}.html"))
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
