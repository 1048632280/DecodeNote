use std::path::PathBuf;
use crate::encoding::EncodingId;

#[derive(Clone, Debug)]
pub struct DocumentSession {
    pub path: PathBuf,
    pub original_bytes: Vec<u8>,
    pub active_encoding: EncodingId,
    pub detected_encoding: Option<EncodingId>,
    pub save_encoding: EncodingId,
    pub revision: u64,
}

impl DocumentSession {
    pub fn new(
        path: PathBuf,
        original_bytes: Vec<u8>,
        active_encoding: EncodingId,
        detected_encoding: Option<EncodingId>,
    ) -> Self {
        let save_encoding = active_encoding.clone();
        Self {
            path,
            original_bytes,
            active_encoding,
            detected_encoding,
            save_encoding,
            revision: 0,
        }
    }

    pub fn bump_revision(&mut self) -> u64 {
        self.revision += 1;
        self.revision
    }

    pub fn update_after_save(&mut self, new_bytes: Vec<u8>, new_path: PathBuf) -> u64 {
        self.original_bytes = new_bytes;
        self.path = new_path;
        self.bump_revision()
    }
}
