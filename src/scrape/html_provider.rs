use log::error;
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

pub(crate) struct HtmlProvider {
    pub cache_dir: PathBuf,
}

impl HtmlProvider {
    pub fn get(&self, url: &str) -> Result<String, HtmlProviderError> {
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

    fn get_from_cache(&self, url: &str) -> Result<Option<String>, HtmlProviderError> {
        let cache_path = self.get_cache_path(url);
        if !cache_path.exists() {
            return Ok(None);
        }
        let mut file = File::open(cache_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Some(contents))
    }

    #[allow(clippy::absolute_paths)]
    fn fetch_and_cache(&self, url: &str) -> Result<String, HtmlProviderError> {
        let client = Client::new();
        let response = client.get(url).send()?;
        if !response.status().is_success() {
            return Err(HtmlProviderError::Status(response.status().as_u16()));
        }
        let content = response.text()?;
        self.cache_content(url, &content)?;
        Ok(content)
    }

    fn cache_content(&self, url: &str, content: &str) -> Result<(), HtmlProviderError> {
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
pub enum HtmlProviderError {
    Status(u16),
    Network(reqwest::Error),
    Io(std::io::Error),
    InvalidUrl(String),
}

#[allow(clippy::absolute_paths)]
impl From<reqwest::Error> for HtmlProviderError {
    fn from(err: reqwest::Error) -> Self {
        HtmlProviderError::Network(err)
    }
}

#[allow(clippy::absolute_paths)]
impl From<std::io::Error> for HtmlProviderError {
    fn from(err: std::io::Error) -> Self {
        HtmlProviderError::Io(err)
    }
}
