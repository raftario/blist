use crate::{
    utils,
    validation::{BeatmapDifficultyError, BeatmapError},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::cmp::Ordering;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Beatmap {
    #[serde(rename = "type")]
    pub ty: BeatmapType,
    pub date: Option<DateTime<Utc>>,
    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
    pub difficulties: Vec<BeatmapDifficulty>,
    pub key: Option<String>,
    pub hash: Option<String>,
    #[serde(rename = "levelID")]
    pub level_id: Option<String>,
    #[serde(default = "Map::new", skip_serializing_if = "Map::is_empty")]
    pub custom_data: Map<String, Value>,
}

impl Beatmap {
    pub fn new_key(key: String) -> Self {
        Self {
            ty: BeatmapType::Key,
            date: Some(Utc::now()),
            difficulties: Vec::new(),
            key: Some(key),
            hash: None,
            level_id: None,
            custom_data: Map::new(),
        }
    }
    pub fn new_hash(hash: String) -> Self {
        Self {
            ty: BeatmapType::Hash,
            date: Some(Utc::now()),
            difficulties: Vec::new(),
            key: None,
            hash: Some(hash),
            level_id: None,
            custom_data: Map::new(),
        }
    }
    pub fn new_level_id(level_id: String) -> Self {
        Self {
            ty: BeatmapType::LevelId,
            date: Some(Utc::now()),
            difficulties: Vec::new(),
            key: None,
            hash: None,
            level_id: Some(level_id),
            custom_data: Map::new(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), BeatmapError> {
        match self.ty {
            BeatmapType::Key => {
                if self.key.is_none() {
                    return Err(BeatmapError::MismatchedType {
                        ty: "key",
                        field: "key",
                    });
                }
            }
            BeatmapType::Hash => {
                if self.hash.is_none() {
                    return Err(BeatmapError::MismatchedType {
                        ty: "hash",
                        field: "hash",
                    });
                }
            }
            BeatmapType::LevelId => {
                if self.level_id.is_none() {
                    return Err(BeatmapError::MismatchedType {
                        ty: "levelID",
                        field: "levelID",
                    });
                }
            }
        }

        for (idx, d) in self.difficulties.iter().enumerate() {
            if let Err(error) = d.validate() {
                return Err(BeatmapError::InvalidDifficulty { idx, error });
            }
        }

        if let Some(k) = &self.key {
            if k.is_empty() || !utils::str_is_hex(k) {
                return Err(BeatmapError::InvalidField {
                    field: "key",
                    value: k.clone(),
                });
            }
        }
        if let Some(h) = &self.hash {
            if h.len() != 40 || !utils::str_is_hex(h) {
                return Err(BeatmapError::InvalidField {
                    field: "hash",
                    value: h.clone(),
                });
            }
        }
        if let Some(li) = &self.level_id {
            if utils::str_is_empty_or_has_newlines(li) {
                return Err(BeatmapError::InvalidField {
                    field: "levelID",
                    value: li.clone(),
                });
            }
        }

        Ok(())
    }
}

impl PartialOrd for Beatmap {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.date.partial_cmp(&other.date)
    }
}
impl Ord for Beatmap {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.cmp(&other.date)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BeatmapType {
    Key,
    Hash,
    #[serde(rename = "levelID")]
    LevelId,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeatmapDifficulty {
    pub name: String,
    pub characteristic: String,
}

impl BeatmapDifficulty {
    pub(crate) fn validate(&self) -> Result<(), BeatmapDifficultyError> {
        if utils::str_is_empty_or_has_newlines(&self.name) {
            return Err(BeatmapDifficultyError::InvalidField {
                field: "name",
                value: self.name.clone(),
            });
        }
        if utils::str_is_empty_or_has_newlines(&self.characteristic) {
            return Err(BeatmapDifficultyError::InvalidField {
                field: "characteristic",
                value: self.characteristic.clone(),
            });
        }
        Ok(())
    }
}
