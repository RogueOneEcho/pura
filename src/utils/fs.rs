use crate::prelude::*;
use std::io::Error;

pub async fn create_parent_dir_if_not_exist(path: &Path) -> Result<(), Error> {
    let dir = path.parent().expect("path should have parent directory");
    if !dir.exists() {
        trace!("Creating directory: {}", dir.display());
        create_dir_all(&dir).await?;
    }
    Ok(())
}
