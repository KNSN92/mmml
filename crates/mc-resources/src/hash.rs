use serde::Deserialize;
use sha1::{Digest, Sha1};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHash([u8; 20]);

impl ResourceHash {
    pub fn new(hash: [u8; 20]) -> Self {
        ResourceHash(hash)
    }

    pub fn from_str(hash_str: &str) -> Result<Self, hex::FromHexError> {
        let mut hash = [0u8; 20];
        hex::decode_to_slice(hash_str, &mut hash)?;
        Ok(ResourceHash(hash))
    }

    pub fn digest(bytes: &[u8]) -> ResourceHash {
        let hash = *Sha1::digest(bytes)
            .as_array::<20>()
            .expect("sha1 is always 20 bytes. so this should never fail! right?");
        ResourceHash(hash)
    }

    pub fn digest_chunks() -> FileHashChunks {
        FileHashChunks(Sha1::new())
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    pub fn prefix(&self) -> String {
        hex::encode(&[self.0[0]])
    }
}

impl std::fmt::Display for ResourceHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for ResourceHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hash_str = String::deserialize(deserializer)?;
        let hash = ResourceHash::from_str(&hash_str).map_err(serde::de::Error::custom)?;
        Ok(hash)
    }
}

pub struct FileHashChunks(Sha1);

impl FileHashChunks {
    pub fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    pub fn finalize(self) -> ResourceHash {
        let hash = *self
            .0
            .finalize()
            .as_array::<20>()
            .expect("sha1 is always 20 bytes. so this should never fail! right?");
        ResourceHash(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::ResourceHash;

    #[test]
    fn hashes_known_and_empty_inputs() {
        assert_eq!(
            ResourceHash::digest(b"abc").to_string(),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(
            ResourceHash::digest(b"").to_string(),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
    }

    #[test]
    fn chunked_hash_matches_one_shot_hash() {
        let mut hash = ResourceHash::digest_chunks();
        hash.update(b"a");
        hash.update(b"b");
        hash.update(b"c");
        assert_eq!(hash.finalize(), ResourceHash::digest(b"abc"));
    }

    #[test]
    fn parses_and_formats_sha1_hashes() {
        let hash = ResourceHash::from_str("01abcdef00000000000000000000000000000000").unwrap();
        assert_eq!(hash.to_string(), "01abcdef00000000000000000000000000000000");
        assert_eq!(hash.prefix(), "01");
        assert_eq!(ResourceHash::from_str(&hash.to_string()).unwrap(), hash);
    }

    #[test]
    fn rejects_malformed_hash_strings() {
        for invalid in [
            "",
            "abc",
            "00gg000000000000000000000000000000000000",
            "000000000000000000000000000000000000000000",
        ] {
            assert!(
                ResourceHash::from_str(invalid).is_err(),
                "accepted {invalid:?}"
            );
        }
        assert!(serde_json::from_str::<ResourceHash>("\"abc\"").is_err());
    }
}
