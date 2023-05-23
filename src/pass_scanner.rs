use std::{
    path::{Path, PathBuf},
    vec::Vec,
};

use crate::errors::Error;

/** Recursively collect files from the provided base path.
*/
fn collect_files(base_dir: &Path) -> Result<Vec<PathBuf>, Error> {
    if !base_dir.is_dir() {
        return Err(Error::InvalidPath {
            path: base_dir.to_path_buf(),
        });
    }

    let mut targets = vec![base_dir.to_owned()];
    let mut results = vec![];

    while !targets.is_empty() {
        let cwd = targets.pop().ok_or(Error::InvalidPath {
            path: "unexpected_path".into(),
        })?;
        for entry in std::fs::read_dir(cwd)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                targets.push(path);
                continue;
            }
            if path.is_file() {
                results.push(path)
            }
        }
    }

    Ok(results)
}

fn collect_pass_files(base_dir: &Path) -> Result<Vec<PathBuf>, Error> {
    let pass_files: Vec<PathBuf> = collect_files(base_dir)?
        .into_iter()
        .filter(|i| i.ends_with(".gpg"))
        .collect();

    Ok(pass_files)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;

    use super::*;

    struct TmpTree {
        base_path: PathBuf,
        expected_files: Vec<PathBuf>,
    }

    impl TmpTree {
        fn new() -> Self {
            let mut base_dir = env::temp_dir();
            base_dir.push(uuid::Uuid::new_v4().hyphenated().to_string());
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

            Self {
                base_path: base_dir,
                expected_files: expected,
            }
        }
    }
    impl Drop for TmpTree {
        fn drop(&mut self) {
            if self.base_path.is_dir() {
                std::fs::remove_dir_all(&self.base_path).unwrap();
            }
        }
    }

    #[test]
    fn test_collecting_files() {
        let tmp_tree = TmpTree::new();

        let collected_files = collect_files(&tmp_tree.base_path);

        match collected_files {
            Ok(collected) => {
                let expected = tmp_tree.expected_files.to_owned().sort();
                let collected = collected.to_owned().sort();
                assert_eq!(collected, expected);
            }
            Err(_) => {
                assert!(false);
            }
        }
    }

    #[test]
    fn test_filter_to_pass_entries() -> Result<(), String> {
        let tmp_tree = TmpTree::new();

        let pass_entries = collect_pass_files(&tmp_tree.base_path);
        match pass_entries {
            Ok(entries) => {
                for entry in entries {
                    assert!(entry.ends_with(".gpg"));
                }
            }
            Err(_err) => {
                assert!(false);
            }
        }

        Ok(())
    }
}
