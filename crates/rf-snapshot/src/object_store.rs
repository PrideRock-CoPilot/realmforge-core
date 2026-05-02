use crate::SnapshotObjectRef;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ObjectStoreError {
    #[error("object io failed")]
    Io(#[from] io::Error),
}

#[derive(Clone, Debug)]
pub struct FileObjectStore {
    root: PathBuf,
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
        Ok(fs::read(self.root.join(Path::new(object_uri)))?)
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

    #[test]
    fn object_uri_is_content_addressed() {
        let uri = object_uri("abcdef");
        assert_eq!(uri, "sha256/ab/cd/abcdef");
    }
}
