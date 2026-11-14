//! Application data layout. Everything lives under the OS app-data directory
//! (never the install directory or a temp folder).

use crate::error::AppResult;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub database: PathBuf,
    pub logs: PathBuf,
    pub backups: PathBuf,
    pub covers: PathBuf,
    pub documents: PathBuf,
    pub exports: PathBuf,
}

impl AppPaths {
    /// Builds the layout under `root` and creates every directory.
    pub fn ensure(root: &Path) -> AppResult<Self> {
        let paths = AppPaths {
            data_dir: root.to_path_buf(),
            database: root.join("pusthakam.db"),
            logs: root.join("logs"),
            backups: root.join("backups"),
            covers: root.join("covers"),
            documents: root.join("documents"),
            exports: root.join("exports"),
        };
        for dir in [
            &paths.data_dir,
            &paths.logs,
            &paths.backups,
            &paths.covers,
            &paths.documents,
            &paths.exports,
        ] {
            std::fs::create_dir_all(dir)?;
        }
        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_all_directories_and_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("Pusthakam");
        let p = AppPaths::ensure(&root).unwrap();
        for d in [&p.logs, &p.backups, &p.covers, &p.documents, &p.exports] {
            assert!(d.is_dir(), "{d:?} should exist");
        }
        assert_eq!(p.database, root.join("pusthakam.db"));
        AppPaths::ensure(&root).unwrap(); // second call must not fail
    }
}
