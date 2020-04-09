use crate::{beatmap::Beatmap, utils, validation::PlaylistError};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    cover: Option<PathBuf>,
    #[serde(skip)]
    cover_data: Option<Vec<u8>>,
    pub maps: Vec<Beatmap>,
    #[serde(default = "Map::new", skip_serializing_if = "Map::is_empty")]
    pub custom_data: Map<String, Value>,
}

impl Playlist {
    pub fn validate(&self) -> Result<(), PlaylistError> {
        if utils::str_is_empty_or_has_newlines(&self.title) {
            return Err(PlaylistError::InvalidField {
                field: "title",
                value: self.title.clone(),
            });
        }
        if let Some(a) = &self.author {
            if utils::str_is_empty_or_has_newlines(a) {
                return Err(PlaylistError::InvalidField {
                    field: "author",
                    value: a.clone(),
                });
            }
        }
        if let Some(d) = &self.description {
            if d.is_empty() {
                return Err(PlaylistError::InvalidField {
                    field: "description",
                    value: d.clone(),
                });
            }
        }

        for (idx, m) in self.maps.iter().enumerate() {
            if let Err(error) = m.validate() {
                return Err(PlaylistError::InvalidBeatmap { idx, error });
            }
        }

        Ok(())
    }
}
