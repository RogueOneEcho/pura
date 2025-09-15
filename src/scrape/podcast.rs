use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Podcast {
    pub title: String,
    pub description: String,
    pub date: String,
    pub episode_type: String,
    pub author: String,
    pub subtitle: String,
    pub summary: String,
    pub duration: usize,
    pub guid: String,
    pub file: PathBuf,
}

impl Display for Podcast {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let message = match serde_yaml::to_string(self) {
            Ok(yaml) => yaml,
            Err(_) => format!("{:?}", self),
        };
        write!(f, "{message}")
    }
}
