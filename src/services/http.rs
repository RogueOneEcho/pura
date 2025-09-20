use crate::prelude::*;
use reqwest::header::CONTENT_TYPE;
use reqwest::Response;
use std::ffi::OsString;
use tokio::fs::{create_dir_all, read_to_string, remove_file};
use tokio::io::AsyncWriteExt;
use urlencoding::encode;

/// A client for making HTTP requests and caching responses
#[derive(Clone, Debug)]
pub struct HttpClient {
    dir: PathBuf,
}

impl HttpClient {
    pub(crate) fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub(crate) async fn get_html(&self, url: &Url) -> Result<Html, HttpError> {
        let path = self.get(url, Some(HTML_EXTENSION)).await?;
        let contents = read_to_string(&path)
            .await
            .map_err(|e| HttpError::Io(path, e))?;
        Ok(Html::parse_document(&contents))
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(&self, url: &Url) -> Result<T, HttpError> {
        let path = self.get(url, Some(JSON_EXTENSION)).await?;
        let file = File::open(&path).map_err(|e| HttpError::Io(path.clone(), e))?;
        let reader = BufReader::new(file);
        match serde_json::from_reader(reader) {
            Ok(json) => Ok(json),
            Err(e) => {
                self.remove(url, Some(JSON_EXTENSION)).await;
                Err(HttpError::InvalidJson(path, e))
            }
        }
    }

    pub(crate) async fn head(&self, url: &Url) -> Result<String, HttpError> {
        let path = self.get_cache_path(url, Some(HEAD_EXTENSION));
        if path.exists() {
            trace!("HEAD cache HIT: {url}");
            read_to_string(&path)
                .await
                .map_err(|e| HttpError::Io(path, e))
        } else {
            trace!("HEAD cache MISS: {url}");
            self.head_to_cache(url, &path).await
        }
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
                trace!("Failed to remove: {}", path.display());
                trace!("{e}");
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
    async fn head_to_cache(&self, url: &Url, path: &PathBuf) -> Result<String, HttpError> {
        create_dir(path).await?;
        let client = ReqwestClient::new();
        trace!("HEAD {url} to {}", path.display());
        let response = client
            .head(url.as_str())
            .send()
            .await
            .map_err(|e| HttpError::Request(url.clone(), e))?;
        let content_type = get_content_type(response).unwrap_or_default();
        let mut file = AsyncFile::create(path)
            .await
            .map_err(|e| HttpError::Io(path.clone(), e))?;
        file.write_all(content_type.as_bytes())
            .await
            .map_err(|e| HttpError::Io(path.clone(), e))?;
        Ok(content_type)
    }

    #[allow(clippy::unused_self)]
    async fn download_to_cache(&self, url: &Url, path: &PathBuf) -> Result<(), HttpError> {
        create_dir(path).await?;
        let client = ReqwestClient::new();
        trace!("Downloading {url} to {}", path.display());
        let mut response = client
            .get(url.as_str())
            .send()
            .await
            .map_err(|e| HttpError::Request(url.clone(), e))?;
        if !response.status().is_success() {
            return Err(HttpError::Response(url.clone(), response.status().as_u16()));
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

async fn create_dir(path: &Path) -> Result<(), HttpError> {
    let dir = path
        .parent()
        .expect("cache path should have a parent directory");
    if !dir.exists() {
        trace!("Creating cache directory: {}", dir.display());
        create_dir_all(dir)
            .await
            .map_err(|e| HttpError::Io(dir.into(), e))?;
    }
    Ok(())
}

fn get_content_type(response: Response) -> Option<String> {
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)?
        .to_str()
        .ok()?
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    Some(content_type)
}

#[allow(clippy::absolute_paths)]
#[derive(Debug)]
pub enum HttpError {
    Response(Url, u16),
    Request(Url, reqwest::Error),
    Io(PathBuf, std::io::Error),
    ResponseIo(Url, reqwest::Error),
    InvalidJson(PathBuf, serde_json::Error),
    NoContentType(Url),
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
            HttpError::NoContentType(url) => {
                format!("Response did not contain a Content-Type header:\nURL: {url}",)
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
    pub async fn head() {
        // Arrange
        let _ = init_logging();
        let http = HttpClient::default();
        let url = Url::parse("https://example.com/?abc=123&def=456").expect("url should be valid");
        http.remove(&url, Some(HEAD_EXTENSION)).await;

        // Act
        let result = http.head(&url).await;

        // Assert
        let content_type = result.assert_ok();
        assert_eq!(content_type, "text/html");
    }

    #[tokio::test]
    #[ignore = "uses simplecast.com"]
    pub async fn head_xml() {
        // Arrange
        let _ = init_logging();
        let http = HttpClient::default();
        let url = Url::parse("https://feeds.simplecast.com/lP7owBq8").expect("url should be valid");
        http.remove(&url, Some(HEAD_EXTENSION)).await;

        // Act
        let result = http.head(&url).await;

        // Assert
        let content_type = result.assert_ok();
        assert_eq!(content_type, "application/xml");
    }

    #[tokio::test]
    pub async fn get() {
        // Arrange
        let _ = init_logging();
        let http = HttpClient::default();
        let url = Url::parse("https://example.com/?abc=123&def=456").expect("url should be valid");
        let expected = http.get_cache_path(&url, Some(HTML_EXTENSION));
        http.remove(&url, Some(HTML_EXTENSION)).await;

        // Act
        let result = http.get(&url, Some(HTML_EXTENSION)).await;

        // Assert
        let path = result.assert_ok();
        assert_eq!(path, expected);
        assert!(path.exists());
    }

    #[tokio::test]
    pub async fn get_html() {
        // Arrange
        let _ = init_logging();
        let http = HttpClient::default();
        let url = Url::parse("https://example.com").expect("url should be valid");
        http.remove(&url, Some(HTML_EXTENSION)).await;

        // Act
        let result = http.get_html(&url).await;

        // Assert
        let _html = result.assert_ok();
    }

    #[tokio::test]
    #[ignore = "uses ipinfo.io"]
    pub async fn get_json() {
        // Arrange
        let _ = init_logging();
        let http = HttpClient::default();
        let url = Url::parse("https://ipinfo.io").expect("url should be valid");
        http.remove(&url, Some(JSON_EXTENSION)).await;

        // Act
        let result = http.get_json::<Value>(&url).await;

        // Assert
        let _json = result.assert_ok();
    }
}
