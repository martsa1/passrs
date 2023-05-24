use log::debug;
use std::{env, path::PathBuf};
use uuid::Uuid;

pub struct TmpTree {
    pub base_path: PathBuf,
    pub expected_files: Vec<PathBuf>,
}

impl TmpTree {
    pub fn new() -> Self {
        let mut base_dir = env::temp_dir();
        base_dir.push(Uuid::new_v4().hyphenated().to_string());
        std::fs::create_dir_all(&base_dir).unwrap();

        let tlds = ["a", "b"];
        let per_dir_files = ["foo", "bar", "target.gpg"];

        let mut expected = vec![];
        for dir in tlds {
            let target = base_dir.join(&dir);
            std::fs::create_dir_all(&target).unwrap();

            for file_ in per_dir_files {
                let f_path = target.join(&file_);
                std::fs::write(&f_path, file_).unwrap();
                expected.push(f_path.to_owned());
            }
        }

        debug!(
            "Created sample directory tree at: {}",
            base_dir.to_string_lossy()
        );
        Self {
            base_path: base_dir,
            expected_files: expected,
        }
    }
}
impl Drop for TmpTree {
    fn drop(&mut self) {
        debug!("Cleaning up {}", self.base_path.to_string_lossy());
        if self.base_path.is_dir() {
            std::fs::remove_dir_all(&self.base_path).unwrap();
        }
    }
}
