use anyhow::Result;
use blist::{beatmap::BeatmapType, playlist::SCHEMA, Beatmap, Playlist};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Map, Value};

const PNG_B64_PREFIX: &str = "data:image/png;base64,";
const JPG_B64_PREFIX: &str = "data:image/jpg;base64,";
const JPEG_B64_PREFIX: &str = "data:image/jpeg;base64,";

#[derive(Deserialize)]
pub struct LegacyPlaylist {
    #[serde(rename = "playlistTitle")]
    title: String,
    #[serde(rename = "playlistAuthor")]
    author: Option<String>,
    #[serde(rename = "playlistDescription")]
    description: Option<String>,
    #[serde(rename = "songs", default)]
    maps: Vec<LegacyBeatmap>,
    #[serde(rename = "image")]
    cover: Option<String>,

    #[serde(flatten, default)]
    custom_data: Map<String, Value>,
}

impl LegacyPlaylist {
    pub fn into_playlist(self, preserve_custom_data: bool) -> Result<Playlist> {
        let Self {
            title,
            author,
            description,
            maps,
            cover,
            custom_data,
        } = self;

        let mut playlist = Playlist {
            _schema: SCHEMA,
            title,
            author,
            description,
            cover: None,
            maps: maps
                .into_iter()
                .map(|m| m.into_beatmap(preserve_custom_data))
                .collect::<Result<Vec<Beatmap>>>()?,
            custom_data: if preserve_custom_data {
                custom_data
            } else {
                Map::new()
            },
        };
        if let Some(c) = cover {
            if c.starts_with(PNG_B64_PREFIX) {
                let mut b64 = &c[PNG_B64_PREFIX.len()..];
                while b64.starts_with(' ') {
                    b64 = &b64[1..];
                }
                let data = base64::decode(b64)?;
                playlist.set_png_cover(data.as_slice())?;
            } else if c.starts_with(JPG_B64_PREFIX) {
                let mut b64 = &c[JPG_B64_PREFIX.len()..];
                while b64.starts_with(' ') {
                    b64 = &b64[1..];
                }
                let data = base64::decode(b64)?;
                playlist.set_jpg_cover(data.as_slice())?;
            } else if c.starts_with(JPEG_B64_PREFIX) {
                let mut b64 = &c[JPEG_B64_PREFIX.len()..];
                while b64.starts_with(' ') {
                    b64 = &b64[1..];
                }
                let data = base64::decode(b64)?;
                playlist.set_jpg_cover(data.as_slice())?;
            }
        }
        Ok(playlist)
    }
}

#[derive(Deserialize)]
struct LegacyBeatmap {
    key: Option<String>,
    hash: Option<String>,
    #[serde(rename = "dateAdded")]
    date: Option<DateTime<Utc>>,

    #[serde(flatten, default)]
    custom_data: Map<String, Value>,
}

impl LegacyBeatmap {
    fn into_beatmap(self, preserve_custom_data: bool) -> Result<Beatmap> {
        let Self {
            key,
            hash,
            date,
            custom_data,
        } = self;

        let ty = if key.is_some() {
            BeatmapType::Key
        } else if hash.is_some() {
            BeatmapType::Hash
        } else {
            BeatmapType::LevelId
        };

        Ok(Beatmap {
            ty,
            date,
            difficulties: Vec::new(),
            key,
            hash,
            level_id: None,
            custom_data: if preserve_custom_data {
                custom_data
            } else {
                Map::new()
            },
        })
    }
}
