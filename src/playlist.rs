use crate::{
    beatmap::Beatmap,
    error::Error,
    utils::{self, JPG_MAGIC_NUMBER, JPG_MAGIC_NUMBER_LEN, PNG_MAGIC_NUMBER, PNG_MAGIC_NUMBER_LEN},
    validation::{PlaylistCoverError, PlaylistError},
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    io::{Read, Seek},
    path::PathBuf,
};
use zip::ZipArchive;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<PlaylistCover>,
    pub maps: Vec<Beatmap>,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub custom_data: Map<String, Value>,
}

impl Playlist {
    pub fn new(title: String) -> Self {
        Self {
            title,
            author: None,
            description: None,
            cover: None,
            maps: Vec::new(),
            custom_data: Map::new(),
        }
    }

    pub fn read<R: Read + Seek>(reader: R) -> Result<Self, Error> {
        let mut zip = ZipArchive::new(reader)?;

        let mut playlist: Self = {
            let mut playlist_file = zip.by_name("playlist.json")?;
            serde_json::from_reader(&mut playlist_file)?
        };

        if let Some(c) = &mut playlist.cover {
            if !utils::path_is_invalid(&c.path) {
                let ext = c.path.extension().unwrap();
                if ext == "png" {
                    c.ty = PlaylistCoverType::Png;
                    let mut cover_file = zip.by_name(c.path.to_str().unwrap())?;
                    cover_file.read_to_end(&mut c.data)?;
                } else if ext == "jpg" || ext == "jpeg" {
                    c.ty = PlaylistCoverType::Jpg;
                    let mut cover_file = zip.by_name(c.path.to_str().unwrap())?;
                    cover_file.read_to_end(&mut c.data)?;
                }
            }
        }

        playlist.validate()?;
        Ok(playlist)
    }

    pub(crate) fn validate(&self) -> Result<(), PlaylistError> {
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

        if let Some(c) = &self.cover {
            c.validate()?;
        }

        for (idx, m) in self.maps.iter().enumerate() {
            if let Err(error) = m.validate() {
                return Err(PlaylistError::InvalidBeatmap { idx, error });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct PlaylistCover {
    #[serde(rename = "cover")]
    path: PathBuf,
    #[serde(skip)]
    data: Vec<u8>,
    #[serde(skip)]
    ty: PlaylistCoverType,
}

impl PlaylistCover {
    pub(crate) fn validate(&self) -> Result<(), PlaylistCoverError> {
        match self.ty {
            PlaylistCoverType::Png => {
                if utils::path_is_invalid(&self.path) || self.path.extension().unwrap() != "png" {
                    return Err(PlaylistCoverError::InvalidCoverPath {
                        ty: "png",
                        path: self.path.clone(),
                    });
                }
                if self.data.len() < PNG_MAGIC_NUMBER_LEN
                    || !constant_time_eq::constant_time_eq(
                        &self.data[0..PNG_MAGIC_NUMBER_LEN],
                        PNG_MAGIC_NUMBER,
                    )
                {
                    return Err(PlaylistCoverError::InvalidCoverData { ty: "png" });
                }
            }
            PlaylistCoverType::Jpg => {
                if utils::path_is_invalid(&self.path) {
                    return Err(PlaylistCoverError::InvalidCoverPath {
                        ty: "jpg",
                        path: self.path.clone(),
                    });
                }
                let ext = self.path.extension().unwrap();
                if ext != "jpg" && ext != "jpeg" {
                    return Err(PlaylistCoverError::InvalidCoverPath {
                        ty: "jpg",
                        path: self.path.clone(),
                    });
                }
                if self.data.len() < JPG_MAGIC_NUMBER_LEN
                    || !constant_time_eq::constant_time_eq(
                        &self.data[0..JPG_MAGIC_NUMBER_LEN],
                        JPG_MAGIC_NUMBER,
                    )
                {
                    return Err(PlaylistCoverError::InvalidCoverData { ty: "jpg" });
                }
            }
            PlaylistCoverType::Unknown => return Err(PlaylistCoverError::UnknownCoverType),
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PlaylistCoverType {
    Png,
    Jpg,
    Unknown,
}

impl Default for PlaylistCoverType {
    #[inline]
    fn default() -> Self {
        Self::Unknown
    }
}
