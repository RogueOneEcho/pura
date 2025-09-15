use crate::prelude::*;
use reqwest::blocking::Client as ReqwestClient;
use sha2::{Digest, Sha256};
use std::fs::{create_dir_all, read_to_string};

pub(crate) struct Client {
    pub cache_dir: PathBuf,
}

impl Client {
    pub fn get_html(&self, url: &Url) -> Result<Html, ClientError> {
        let path = self.get(url)?;
        let contents = read_to_string(path)?;
        Ok(Html::parse_document(&contents))
    }

    pub fn get(&self, url: &Url) -> Result<PathBuf, ClientError> {
        let path = self.get_cache_path(url);
        if path.exists() {
            trace!("Cache HIT: {url}");
        } else {
            trace!("Cache MISS: {url}");
            self.download_to_cache(url, &path)?;
        }
        Ok(path)
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

    #[allow(clippy::unused_self)]
    fn download_to_cache(&self, url: &Url, path: &PathBuf) -> Result<(), ClientError> {
        let dir = path
            .parent()
            .expect("cache path should have a parent directory");
        if !dir.exists() {
            trace!("Creating cache directory: {}", dir.display());
            create_dir_all(dir)?;
        }
        let client = ReqwestClient::new();
        trace!("Downloading {url} to {}", path.display());
        let response = client.get(url.as_str()).send()?;
        if !response.status().is_success() {
            return Err(ClientError::Status(response.status().as_u16()));
        }
        let mut file = File::create(path)?;
        let content = &response.bytes()?;
        file.write_all(content)?;
        Ok(())
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
            ClientError::Status(number) => StatusCode::from_u16(*number).map_or_else(
                |_| format!("{number}"),
                |code| format!("{number} {}", code.canonical_reason().unwrap_or_default()),
            ),
            ClientError::Network(e) => format!("{e}"),
            ClientError::Io(e) => format!("{e}"),
            ClientError::InvalidUrl(e) => e.to_string(),
        };
        write!(f, "{message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scrape::options::ScrapeOptions;
    use std::fs::remove_file;

    const HTML_URL: &str = "https://httpbin.org/html";

    #[test]
    pub fn get() -> Result<(), ScrapeError> {
        // Arrange
        let _ = init_logging();
        let client = Client {
            cache_dir: ScrapeOptions::DEFAULT_CACHE_DIR.into(),
        };
        let url = Url::parse(HTML_URL).expect("url should be valid");
        let expected = client.get_cache_path(&url);
        if expected.exists() {
            remove_file(&expected).expect("cache file should be deleted");
        }

        // Act
        let path = client.get(&url)?;

        // Assert
        assert_eq!(path, expected);
        assert!(path.exists());
        Ok(())
    }

    #[test]
    pub fn get_html() -> Result<(), ScrapeError> {
        // Arrange
        let _ = init_logging();
        let client = Client {
            cache_dir: ScrapeOptions::DEFAULT_CACHE_DIR.into(),
        };
        let url = Url::parse(HTML_URL).expect("url should be valid");
        let expected = client.get_cache_path(&url);
        if expected.exists() {
            remove_file(&expected).expect("cache file should be deleted");
        }

        // Act
        let _html = client.get_html(&url)?;

        // Assert
        Ok(())
    }
}
