use crate::SnapshotObjectRef;
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObjectStoreError {
    #[error("object io failed")]
    Io(#[from] io::Error),
    #[error("object not found: {0}")]
    NotFound(String),
}

#[derive(Clone, Debug)]
pub struct FileObjectStore {
    root: PathBuf,
}

#[derive(Clone, Debug)]
pub struct ObjectStoreStats {
    pub object_count: u64,
    pub total_size_bytes: u64,
}

impl FileObjectStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn put_bytes(
        &self,
        logical_path: impl Into<String>,
        bytes: &[u8],
    ) -> Result<SnapshotObjectRef, ObjectStoreError> {
        let sha256 = hex::encode(Sha256::digest(bytes));
        let object_uri = object_uri(&sha256);
        let target = self.root.join(&object_uri);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, bytes)?;
        Ok(SnapshotObjectRef {
            logical_path: logical_path.into(),
            object_uri,
            sha256,
            size_bytes: bytes.len() as u64,
        })
    }

    pub fn get_bytes(&self, object_uri: &str) -> Result<Vec<u8>, ObjectStoreError> {
        let path = self.root.join(Path::new(object_uri));
        if !path.exists() {
            return Err(ObjectStoreError::NotFound(object_uri.to_string()));
        }
        Ok(fs::read(path)?)
    }

    /// Delete an object by URI.
    pub fn delete(&self, object_uri: &str) -> Result<(), ObjectStoreError> {
        let path = self.root.join(Path::new(object_uri));
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// List all objects in the store (walks the sha256 directory structure).
    pub fn list(&self) -> Result<Vec<SnapshotObjectRef>, ObjectStoreError> {
        let mut refs = Vec::new();
        let sha256_dir = self.root.join("sha256");
        if sha256_dir.exists() {
            self.walk_dir(&sha256_dir, &sha256_dir, &mut refs)?;
        }
        Ok(refs)
    }

    /// Get storage statistics.
    pub fn stats(&self) -> Result<ObjectStoreStats, ObjectStoreError> {
        let refs = self.list()?;
        let count = refs.len() as u64;
        let total_size = refs.iter().map(|r| r.size_bytes).sum();
        Ok(ObjectStoreStats {
            object_count: count,
            total_size_bytes: total_size,
        })
    }

    /// Get metadata about a single object by URI.
    pub fn stat(&self, object_uri: &str) -> Result<SnapshotObjectRef, ObjectStoreError> {
        let path = self.root.join(Path::new(object_uri));
        if !path.exists() {
            return Err(ObjectStoreError::NotFound(object_uri.to_string()));
        }
        let metadata = fs::metadata(&path)?;
        let bytes = fs::read(&path)?;
        let sha256 = hex::encode(Sha256::digest(&bytes));
        Ok(SnapshotObjectRef {
            logical_path: object_uri.to_string(),
            object_uri: object_uri.to_string(),
            sha256,
            size_bytes: metadata.len(),
        })
    }

    fn walk_dir(
        &self,
        dir: &Path,
        base: &Path,
        refs: &mut Vec<SnapshotObjectRef>,
    ) -> Result<(), ObjectStoreError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.walk_dir(&path, base, refs)?;
            } else {
                let bytes = fs::read(&path)?;
                let sha256 = hex::encode(Sha256::digest(&bytes));
                let relative = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                let object_uri = format!("sha256/{}", relative);
                refs.push(SnapshotObjectRef {
                    logical_path: object_uri.clone(),
                    object_uri,
                    sha256,
                    size_bytes: bytes.len() as u64,
                });
            }
        }
        Ok(())
    }
}

fn object_uri(sha256: &str) -> String {
    let first = &sha256[0..2];
    let second = &sha256[2..4];
    format!("sha256/{first}/{second}/{sha256}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn object_uri_is_content_addressed() {
        let uri = object_uri("abcdef");
        assert_eq!(uri, "sha256/ab/cd/abcdef");
    }

    #[test]
    fn put_and_get_roundtrip() {
        let dir = tempdir().unwrap();
        let store = FileObjectStore::new(dir.path());
        let obj = store.put_bytes("test.txt", b"hello world").unwrap();
        let retrieved = store.get_bytes(&obj.object_uri).unwrap();
        assert_eq!(retrieved, b"hello world");
    }

    #[test]
    fn delete_removes_object() {
        let dir = tempdir().unwrap();
        let store = FileObjectStore::new(dir.path());
        let obj = store.put_bytes("delete_me.txt", b"data").unwrap();
        assert!(store.get_bytes(&obj.object_uri).is_ok());
        store.delete(&obj.object_uri).unwrap();
        assert!(store.get_bytes(&obj.object_uri).is_err());
    }

    #[test]
    fn list_and_stats_work() {
        let dir = tempdir().unwrap();
        let store = FileObjectStore::new(dir.path());
        store.put_bytes("a.txt", b"aaa").unwrap();
        store.put_bytes("b.txt", b"bbbb").unwrap();
        let refs = store.list().unwrap();
        assert_eq!(refs.len(), 2);
        let stats = store.stats().unwrap();
        assert_eq!(stats.object_count, 2);
        assert_eq!(stats.total_size_bytes, 7);
    }
}
