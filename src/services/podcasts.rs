use crate::prelude::*;

pub struct PodcastProvider {
    dir: PathBuf,
}

impl PodcastProvider {
    pub(crate) fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub(crate) fn get(&self, id: &str) -> Result<Podcast, DatabaseError> {
        let path = self.get_path(id);
        if !path.exists() {
            return Err(DatabaseError::NotFound(id.to_owned()));
        }
        let file = File::open(&path).map_err(|e| DatabaseError::Io(path.clone(), e))?;
        let reader = BufReader::new(file);
        serde_yaml::from_reader(reader).map_err(|e| DatabaseError::Deserialization(path.clone(), e))
    }

    pub(crate) fn put(&self, podcast: &Podcast) -> Result<(), DatabaseError> {
        let path = self.get_path(&podcast.id);
        let file = File::create(&path).map_err(|e| DatabaseError::Io(path.clone(), e))?;
        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, podcast)
            .map_err(|e| DatabaseError::Serialization(path.clone(), e))
    }

    fn get_path(&self, id: &str) -> PathBuf {
        self.dir.join(id).with_extension("yml")
    }
}

impl Default for PodcastProvider {
    fn default() -> Self {
        Self {
            dir: PathProvider::default().get_podcast_dir(),
        }
    }
}

#[allow(clippy::absolute_paths)]
#[derive(Debug)]
pub enum DatabaseError {
    NotFound(String),
    Io(PathBuf, std::io::Error),
    Serialization(PathBuf, serde_yaml::Error),
    Deserialization(PathBuf, serde_yaml::Error),
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let message = match self {
            DatabaseError::NotFound(id) => format!("Podcast not found: {id}"),
            DatabaseError::Io(path, e) => {
                format!("An I/O error occurred.\nPath: {}\n{e}", path.display())
            }
            DatabaseError::Serialization(path, e) => {
                format!(
                    "A serialization error occurred.\nPath: {}\n{e}",
                    path.display()
                )
            }
            DatabaseError::Deserialization(path, e) => {
                format!(
                    "A deserialization error occurred.\nPath: {}\n{e}",
                    path.display()
                )
            }
        };
        write!(f, "{message}")
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    pub fn put_then_get() {
        // Arrange
        let _ = init_logging();
        let podcasts = PodcastProvider::default();
        let podcast = Podcast::example();

        // Act
        podcasts.put(&podcast).assert_ok();
        let result = podcasts.get(&podcast.id);

        // Assert
        let result = result.assert_ok();
        assert_eq!(podcast, result);
    }
}
