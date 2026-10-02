use serde::Deserialize;
use sha1::{Digest, Sha1};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileHash([u8; 20]);

impl FileHash {
    pub fn new(hash: [u8; 20]) -> Self {
        FileHash(hash)
    }

    pub fn from_str(hash_str: &str) -> Result<Self, hex::FromHexError> {
        let mut hash = [0u8; 20];
        hex::decode_to_slice(hash_str, &mut hash)?;
        Ok(FileHash(hash))
    }

    pub fn digest(bytes: &[u8]) -> FileHash {
        let hash = *Sha1::digest(bytes)
            .as_array::<20>()
            .expect("sha1 is always 20 bytes. so this should never fail! right?");
        FileHash(hash)
    }

    pub fn digest_chunks() -> FileHashChunks {
        FileHashChunks(Sha1::new())
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl std::fmt::Display for FileHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for FileHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hash_str = String::deserialize(deserializer)?;
        let hash = FileHash::from_str(&hash_str).map_err(serde::de::Error::custom)?;
        Ok(hash)
    }
}

pub struct FileHashChunks(Sha1);

impl FileHashChunks {
    pub fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    pub fn finalize(self) -> FileHash {
        let hash = *self
            .0
            .finalize()
            .as_array::<20>()
            .expect("sha1 is always 20 bytes. so this should never fail! right?");
        FileHash(hash)
    }
}
