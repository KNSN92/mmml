use std::collections::HashMap;

use serde::Deserialize;

use crate::hash::FileHash;

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "debug", serde(deny_unknown_fields))]
pub struct AssetObject {
    pub hash: FileHash,
    pub size: u64,
}
