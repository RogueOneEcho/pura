use crate::prelude::*;

pub struct Validate;

impl Validate {
    pub(crate) fn directory(field: &str, dir: PathBuf) -> Result<(), ValidationError> {
        if dir == PathBuf::new() {
            return Err(ValidationError::RequiredPath(field.to_owned()));
        }
        if !dir.is_dir() {
            return Err(ValidationError::DirectoryNotExist(
                field.to_owned(),
                dir.clone(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ValidationError {
    RequiredPath(String),
    PathNotExist(String, PathBuf),
    DirectoryNotExist(String, PathBuf),
    FileNotExist(String, PathBuf),
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ValidationError::RequiredPath(name) => {
                write!(f, "{name} is required")
            }
            ValidationError::PathNotExist(name, path) => {
                write!(f, "{name} does not exist:\n{}", path.display())
            }
            ValidationError::DirectoryNotExist(name, path) => {
                write!(f, "{name} is not a directory:\n{}", path.display())
            }
            ValidationError::FileNotExist(name, path) => {
                write!(f, "{name} is not a file:\n{}", path.display())
            }
        }
    }
}
