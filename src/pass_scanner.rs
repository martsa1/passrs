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

/** Fuzzy filter the provided vec of pass entries with the filter key.
 * Returns Some vector of matching strings, or None if there are no matches left.
*/
pub fn filter_pass_entries(pass_entries: &Vec<String>, filter: &str) -> Option<Vec<String>> {
    use fuzzy_matcher::skim::SkimMatcherV2;
    use fuzzy_matcher::FuzzyMatcher;

    debug!("Filter string: {:?}", filter);
    if filter == "" {
        let res = pass_entries.to_vec();
        return Some(res);
    }

    let matcher = SkimMatcherV2::default().ignore_case();
    let mut matched_entries: Vec<String> = pass_entries
        .iter()
        .filter_map(|x| match matcher.fuzzy_match(x, &filter) {
            Some(val) => {
                return Some(format!("{}: {}", val, x));
            }
            None => {
                return None;
            }
        })
        .collect();
    matched_entries.sort();
    matched_entries.reverse();
    // TODO: is poentially more elegant to use `.unstable_sort_by(|a, b| b.cmp(a))` per
    // rust docs: https://doc.rust-lang.org/std/primitive.slice.html#method.sort_unstable_by
    return Some(matched_entries);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TmpTree;
    use anyhow::Result;

    #[test]
    fn test_collecting_files() -> Result<()> {
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

        Ok(())
    }

    #[test]
    fn test_filter_to_pass_entries() -> Result<()> {
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

    #[test]
    fn test_filter_pass_entries() -> Result<()> {
        let sample_entries: Vec<String> = vec![
            "p/foo".into(),
            "p/bar".into(),
            "w/welp".into(),
            "w/winning".into(),
        ];

        // An empty filter should return all results
        let res = filter_pass_entries(&sample_entries, "");
        match res {
            Some(res) => {
                assert_eq!(sample_entries, res);
            }
            None => {
                assert!(false, "No results from filter");
            }
        }

        // A non-empty filter should return fuzzy matches.
        let res = filter_pass_entries(&sample_entries, "wp");
        assert!(res.is_some());

        let res = res.unwrap();
        assert_eq!(res.len(), 1);

        assert!(res[0].ends_with("w/welp"));

        Ok(())
    }
}
