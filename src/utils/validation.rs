use crate::prelude::*;

pub struct Validate;

impl Validate {
    pub(crate) fn directory(dir: PathBuf) -> Result<(), PathValidationError> {
        if dir == PathBuf::new() {
            return Err(PathValidationError::Required);
        }
        if !dir.is_dir() {
            return Err(PathValidationError::NotDirectory(dir.clone()));
        }
        Ok(())
    }

    pub(crate) fn expect(expected: &str, actual: &str) -> Result<(), StringValidationError> {
        if expected != actual {
            return Err(StringValidationError::Expected(
                expected.to_owned(),
                actual.to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ValidationError {
    String(String, StringValidationError),
    Path(String, PathValidationError),
    Http(HttpError),
}

#[derive(Debug)]
pub enum StringValidationError {
    Required,
    Expected(String, String),
}

#[derive(Debug)]
pub enum PathValidationError {
    Required,
    PathNotExist(PathBuf),
    NotDirectory(PathBuf),
    NotFile(PathBuf),
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ValidationError::Path(name, error) => {
                write!(f, "{name} {}", error.log())
            }
            ValidationError::String(name, error) => {
                write!(f, "{name} {}", error.log())
            }
            ValidationError::Http(e) => {
                write!(f, "{e}")
            }
        }
    }
}

impl StringValidationError {
    fn log(&self) -> String {
        match self {
            StringValidationError::Required => "is required".to_owned(),
            StringValidationError::Expected(expected, actual) => {
                format!("did not match.\nexpected: {expected}\nactual: {actual}")
            }
        }
    }
}

impl PathValidationError {
    fn log(&self) -> String {
        match self {
            PathValidationError::Required => "is required".to_owned(),
            PathValidationError::PathNotExist(path) => {
                format!("does not exist:\n{}", path.display())
            }
            PathValidationError::NotDirectory(path) => {
                format!("is not a directory:\n{}", path.display())
            }
            PathValidationError::NotFile(path) => {
                format!("is not a file:\n{}", path.display())
            }
        }
    }
}

pub trait ValidationErrorExt {
    fn log(&self) -> String;
    fn to_result(self) -> Result<(), Vec<ValidationError>>;
}

impl ValidationErrorExt for Vec<ValidationError> {
    fn log(&self) -> String {
        self.iter().fold(String::new(), |mut acc, err| {
            acc.push('\n');
            acc.push_str(&err.to_string());
            acc
        })
    }

    fn to_result(self) -> Result<(), Vec<ValidationError>> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}
