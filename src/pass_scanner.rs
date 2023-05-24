use std::{
    path::{Path, PathBuf},
    vec::Vec,
};

use crate::errors::Error;
use log::debug;

/** Recursively collect files from the provided base path.
*/
pub fn collect_files(base_dir: &Path) -> Result<Vec<PathBuf>, Error> {
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
        debug!("Searching {}", cwd.to_string_lossy());

        for entry in std::fs::read_dir(cwd)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                //debug!(
                //    "Found directory to recurse into: {}",
                //    path.to_string_lossy()
                //);
                targets.push(path);
                continue;
            }
            if path.is_file() {
                //debug!("Found file: {}", path.to_string_lossy());
                results.push(path)
            }
        }
    }

    debug!("Found {} files", results.len());
    Ok(results)
}

pub fn collect_pass_files(base_dir: &Path) -> Result<Vec<PathBuf>, Error> {
    debug!(
        "Searching '{}' for password entries",
        base_dir.to_string_lossy()
    );

    let pass_files: Vec<PathBuf> = collect_files(base_dir)?
        .into_iter()
        .filter(|i| match i.extension() {
            Some(ext) => return ext.to_string_lossy() == "gpg",
            None => return false,
        })
        .collect();

    debug!("Found {} entries with .gpg extension", pass_files.len());

    Ok(pass_files)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;

    use super::*;
    use crate::test_util::TmpTree;

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
            Err(err) => {
                assert!(false, "file collection failed: {:?}", err);
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
                    let entry_str = entry.to_string_lossy();
                    assert!(
                        entry_str.ends_with(".gpg"),
                        "entry was meant to end with .gpg: {:?} ({})",
                        entry_str,
                        entry_str.ends_with(".gpg")
                    );
                }
            }
            Err(err) => {
                assert!(false, "pass file collection failed: {:?}", err);
            }
        }

        Ok(())
    }
}
