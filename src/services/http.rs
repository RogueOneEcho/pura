use crate::prelude::*;
use std::ffi::OsString;
use tokio::fs::{create_dir_all, read_to_string, remove_file};
use tokio::io::AsyncWriteExt;
use urlencoding::encode;

/// A client for making HTTP requests and caching responses
pub struct HttpClient {
    dir: PathBuf,
}

impl HttpClient {
    pub(crate) fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub(crate) async fn get_html(&self, url: &Url) -> Result<Html, HttpError> {
        let path = self.get(url, Some("html")).await?;
        let contents = read_to_string(&path)
            .await
            .map_err(|e| HttpError::Io(path, e))?;
        Ok(Html::parse_document(&contents))
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(&self, url: &Url) -> Result<T, HttpError> {
        let path = self.get(url, Some("json")).await?;
        let file = File::open(&path).map_err(|e| HttpError::Io(path.clone(), e))?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| HttpError::InvalidJson(path, e))
    }

    pub(crate) async fn get(
        &self,
        url: &Url,
        extension: Option<&str>,
    ) -> Result<PathBuf, HttpError> {
        let path = self.get_cache_path(url, extension);
        if path.exists() {
            trace!("Cache HIT: {url}");
        } else {
            trace!("Cache MISS: {url}");
            self.download_to_cache(url, &path).await?;
        }
        Ok(path)
    }

    pub(crate) async fn remove(&self, url: &Url, extension: Option<&str>) -> bool {
        let path = self.get_cache_path(url, extension);
        let exists = path.exists();
        if exists {
            trace!("Removing: {}", path.display());
            if let Err(e) = remove_file(&path).await {
                debug!("Failed to remove: {}", path.display());
                debug!("{e}");
                return false;
            };
        }
        exists
    }

    fn get_cache_path(&self, url: &Url, extension: Option<&str>) -> PathBuf {
        let domain = url.domain().unwrap_or("__unknown");
        let mut segments: PathBuf = url
            .path_segments()
            .expect("url should have path segments")
            .collect();
        if segments == PathBuf::new() {
            segments = PathBuf::from("__root");
        }
        let mut path = self.dir.join(domain).join(segments);
        if let Some(query) = url.query() {
            let mut file_name = path
                .file_name()
                .expect("path should have a filename")
                .to_owned();
            file_name.push(OsString::from("-"));
            file_name.push(OsString::from(encode(query).as_ref()));
            path.set_file_name(file_name);
        }
        if let Some(extension) = extension {
            path.set_extension(extension);
        }
        path
    }

    #[allow(clippy::unused_self)]
    async fn download_to_cache(&self, url: &Url, path: &PathBuf) -> Result<(), HttpError> {
        let dir = path
            .parent()
            .expect("cache path should have a parent directory");
        if !dir.exists() {
            trace!("Creating cache directory: {}", dir.display());
            create_dir_all(dir)
                .await
                .map_err(|e| HttpError::Io(dir.into(), e))?;
        }
        let client = ReqwestClient::new();
        trace!("Downloading {url} to {}", path.display());
        let mut response = client
            .get(url.as_str())
            .send()
            .await
            .map_err(|e| HttpError::Request(url.clone(), e))?;
        if !response.status().is_success() {
            return Err(HttpError::Response(
                url.clone(),
                response.status().as_u16(),
            ));
        }
        let mut file = AsyncFile::create(path)
            .await
            .map_err(|e| HttpError::Io(path.clone(), e))?;
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| HttpError::ResponseIo(url.clone(), e))?
        {
            file.write_all(&chunk)
                .await
                .map_err(|e| HttpError::Io(path.clone(), e))?;
        }
        Ok(())
    }
}

#[allow(clippy::absolute_paths)]
#[derive(Debug)]
pub enum HttpError {
    Response(Url, u16),
    Request(Url, reqwest::Error),
    Io(PathBuf, std::io::Error),
    ResponseIo(Url, reqwest::Error),
    InvalidJson(PathBuf, serde_json::Error),
}

impl Display for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let message = match self {
            HttpError::Response(url, number) => {
                let reason = StatusCode::from_u16(*number)
                    .map(|e| e.canonical_reason())
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                format!("Unexpected response status: {number} {reason}\nURL: {url}")
            }
            HttpError::Request(url, e) => format!("A request error occurred.\nURL:{url}\n{e}"),
            HttpError::Io(path, e) => {
                format!("An I/O error occurred.\nPath: {}\n{e}", path.display())
            }
            HttpError::InvalidJson(path, e) => {
                format!(
                    "A deserialization error occurred.\nPath: {}\n{e}",
                    path.display()
                )
            }
            HttpError::ResponseIo(url, e) => {
                format!("A response I/O error occurred.\nURL: {url}\n{e}",)
            }
        };
        write!(f, "{message}")
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            dir: PathProvider::default().get_http_dir(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use serde_json::Value;

    #[tokio::test]
    pub async fn get() -> Result<(), HttpError> {
        // Arrange
        let _ = init_logging();
        let http = HttpClient::default();
        let url = Url::parse("https://example.com/?abc=123&def=456").expect("url should be valid");
        let expected = http.get_cache_path(&url, Some("html"));
        http.remove(&url, Some("html")).await;

        // Act
        let path = http.get(&url, Some("html")).await?;

        // Assert
        assert_eq!(path, expected);
        assert!(path.exists());
        Ok(())
    }

    #[tokio::test]
    pub async fn get_html() -> Result<(), HttpError> {
        // Arrange
        let _ = init_logging();
        let http = HttpClient::default();
        let url = Url::parse("https://example.com").expect("url should be valid");
        http.remove(&url, Some("html")).await;

        // Act
        let _html = http.get_html(&url).await?;

        // Assert
        Ok(())
    }

    #[tokio::test]
    pub async fn get_json() -> Result<(), HttpError> {
        // Arrange
        let _ = init_logging();
        let http = HttpClient::default();
        let url = Url::parse("https://ipinfo.io").expect("url should be valid");
        http.remove(&url, Some("json")).await;

        // Act
        let _json: Value = http.get_json(&url).await?;

        // Assert
        Ok(())
    }
}
