use crate::{
    beatmap::Beatmap,
    error::Error,
    utils::{self, JPG_MAGIC_NUMBER, JPG_MAGIC_NUMBER_LEN, PNG_MAGIC_NUMBER, PNG_MAGIC_NUMBER_LEN},
    validation::{PlaylistCoverError, PlaylistError},
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    io::{Read, Seek, Write},
    path::PathBuf,
};
use zip::{ZipArchive, ZipWriter};

pub const SCHEMA: &str =
    "https://raw.githubusercontent.com/raftario/blist/master/playlist.schema.json";
#[inline]
fn schema() -> &'static str {
    SCHEMA
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    #[serde(rename = "$schema", default = "schema", skip_deserializing)]
    pub _schema: &'static str,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
            _schema: SCHEMA,
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
                    let mut cover_file = zip.by_name(c.path.to_str().unwrap())?;

                    let mut magic_number = [0; PNG_MAGIC_NUMBER_LEN];
                    cover_file.read_exact(&mut magic_number)?;
                    if !constant_time_eq::constant_time_eq(
                        &magic_number[..PNG_MAGIC_NUMBER_LEN],
                        PNG_MAGIC_NUMBER,
                    ) {
                        return Err(Error::Validation(
                            PlaylistCoverError::InvalidCoverData { ty: "png" }.into(),
                        ));
                    }

                    cover_file.read_to_end(&mut c.data)?;
                    c.ty = PlaylistCoverType::Png;
                } else if ext == "jpg" || ext == "jpeg" {
                    let mut cover_file = zip.by_name(c.path.to_str().unwrap())?;

                    let mut magic_number = [0; JPG_MAGIC_NUMBER_LEN];
                    cover_file.read_exact(&mut magic_number)?;
                    if !constant_time_eq::constant_time_eq(
                        &magic_number[..JPG_MAGIC_NUMBER_LEN],
                        JPG_MAGIC_NUMBER,
                    ) {
                        return Err(Error::Validation(
                            PlaylistCoverError::InvalidCoverData { ty: "jpg" }.into(),
                        ));
                    }

                    cover_file.read_to_end(&mut c.data)?;
                    c.ty = PlaylistCoverType::Jpg;
                } else {
                    return Err(Error::Validation(
                        PlaylistCoverError::UnknownCoverType.into(),
                    ));
                }
            } else {
                return Err(Error::Validation(
                    PlaylistCoverError::InvalidCoverPath {
                        ty: "unknown",
                        path: c.path.clone(),
                    }
                    .into(),
                ));
            }
        }

        playlist.validate_inner(false)?;
        Ok(playlist)
    }
    pub fn write<W: Write + Seek>(&self, writer: W) -> Result<(), Error> {
        self.validate_inner(true)?;

        let mut zip = ZipWriter::new(writer);

        zip.start_file("playlist.json", Default::default())?;
        serde_json::to_writer(&mut zip, &self)?;

        if let Some(c) = &self.cover {
            zip.start_file_from_path(&c.path, Default::default())?;
            zip.write_all(&c.data)?;
        }

        zip.finish()?;
        Ok(())
    }

    pub fn set_png_cover<R: Read>(&mut self, mut reader: R) -> Result<(), Error> {
        let path = PathBuf::from("cover.png");
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        let ty = PlaylistCoverType::Png;

        if let Some(c) = self.cover.as_mut() {
            c.path = path;
            c.data = data;
            c.ty = ty;
        } else {
            self.cover = Some(PlaylistCover { path, data, ty });
        }

        Ok(())
    }
    pub fn set_jpg_cover<R: Read>(&mut self, mut reader: R) -> Result<(), Error> {
        let path = PathBuf::from("cover.jpg");
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        let ty = PlaylistCoverType::Jpg;

        if let Some(c) = self.cover.as_mut() {
            c.path = path;
            c.data = data;
            c.ty = ty;
        } else {
            self.cover = Some(PlaylistCover { path, data, ty });
        }

        Ok(())
    }

    #[inline]
    pub fn validate(&self) -> Result<(), Error> {
        Ok(self.validate_inner(true)?)
    }

    pub(crate) fn validate_inner(&self, validate_cover: bool) -> Result<(), PlaylistError> {
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

        if validate_cover {
            if let Some(c) = &self.cover {
                c.validate()?;
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct PlaylistCover {
    #[serde(rename = "cover")]
    pub path: PathBuf,
    #[serde(skip)]
    pub data: Vec<u8>,
    #[serde(skip)]
    pub ty: PlaylistCoverType,
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
                        &self.data[..PNG_MAGIC_NUMBER_LEN],
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
                        &self.data[..JPG_MAGIC_NUMBER_LEN],
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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

#[cfg(test)]
mod tests {
    use crate::{
        beatmap::BeatmapDifficulty,
        playlist::{PlaylistCover, PlaylistCoverType},
        Beatmap, Playlist,
    };
    use serde_json::Value;
    use std::{io::Cursor, path::PathBuf};

    #[test]
    fn write_and_read() {
        let mut old = Playlist::new("playlist".to_owned());
        old.author = Some("author".to_owned());
        old.description = Some("description".to_owned());
        old.custom_data
            .insert("key".to_owned(), Value::String("value".to_owned()));

        let mut map = Beatmap::new_key("16af".to_owned());
        map.difficulties.push(BeatmapDifficulty {
            name: "Expert+".to_owned(),
            characteristic: "normal".to_owned(),
        });
        old.maps.push(map);
        old.maps.push(Beatmap::new_hash(
            "0123456789abcdef0123456789abcdef01234567".to_string(),
        ));
        old.maps.push(Beatmap::new_level_id("level ID".to_string()));

        let mut buffer = Cursor::new(Vec::new());
        old.write(&mut buffer).unwrap();

        buffer.set_position(0);
        let new = Playlist::read(&mut buffer).unwrap();

        assert_eq!(old, new);
    }

    #[test]
    fn validation() {
        let string = "string".to_owned();
        let newline = "newline\n".to_owned();
        let empty = "".to_owned();

        let mut playlist = Playlist::new(string.clone());

        let invalid_title = Playlist::new(newline.clone());
        assert!(invalid_title.validate().is_err());

        let mut invalid_author = playlist.clone();
        invalid_author.author = Some(newline.clone());
        assert!(invalid_author.validate().is_err());

        let mut invalid_description = playlist.clone();
        invalid_description.description = Some(empty.clone());
        assert!(invalid_description.validate().is_err());

        let mut invalid_cover_path = playlist.clone();
        invalid_cover_path.cover = Some(PlaylistCover {
            path: PathBuf::from("subdirectory").join("cover.exe"),
            data: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            ty: PlaylistCoverType::Jpg,
        });
        assert!(invalid_cover_path.validate().is_err());

        let mut invalid_cover_data = playlist.clone();
        invalid_cover_data.cover = Some(PlaylistCover {
            path: PathBuf::from("cover.png"),
            data: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            ty: PlaylistCoverType::Png,
        });
        assert!(invalid_cover_data.validate().is_err());

        let mut unknown_cover_type = playlist.clone();
        unknown_cover_type.cover = Some(PlaylistCover {
            path: PathBuf::from("cover"),
            data: Vec::new(),
            ty: PlaylistCoverType::Unknown,
        });
        assert!(unknown_cover_type.validate().is_err());

        let invalid_key = Beatmap::new_key(string.clone());
        playlist.maps.push(invalid_key);
        assert!(playlist.validate().is_err());

        playlist.maps.clear();
        let invalid_hash = Beatmap::new_hash(string);
        playlist.maps.push(invalid_hash);
        assert!(playlist.validate().is_err());

        playlist.maps.clear();
        let invalid_level_id = Beatmap::new_level_id(empty.clone());
        playlist.maps.push(invalid_level_id);
        assert!(playlist.validate().is_err());

        playlist.maps.clear();
        let mut invalid_difficulty = Beatmap::new_key("16af".to_owned());
        invalid_difficulty.difficulties.push(BeatmapDifficulty {
            name: empty,
            characteristic: newline,
        });
        playlist.maps.push(invalid_difficulty);
        assert!(playlist.validate().is_err());
    }
}
