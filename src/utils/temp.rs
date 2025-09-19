#![allow(dead_code)]
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct TempDirectory;

impl TempDirectory {
    #[must_use]
    pub fn get(sub_dir_name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Duration should be valid")
            .as_millis()
            .to_string();
        temp_dir().join(sub_dir_name).join(timestamp)
    }

    #[must_use]
    pub fn create(sub_dir_name: &str) -> PathBuf {
        let dir = Self::get(sub_dir_name);
        create_dir_all(&dir).expect("Should be able to create temp dir");
        dir
    }
}
