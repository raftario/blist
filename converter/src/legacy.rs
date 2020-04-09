use anyhow::Result;
use blist::{beatmap::BeatmapType, Beatmap, Playlist};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Map, Value};

const PNG_MAGIC_NUMBER_LEN: usize = 8;
const PNG_MAGIC_NUMBER: &[u8; PNG_MAGIC_NUMBER_LEN] =
    &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

const JPG_MAGIC_NUMBER_LEN: usize = 3;
const JPG_MAGIC_NUMBER: &[u8; JPG_MAGIC_NUMBER_LEN] = &[0xFF, 0xD8, 0xFF];

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
            let data = base64::decode(c)?;
            if data.len() >= PNG_MAGIC_NUMBER_LEN
                && constant_time_eq::constant_time_eq(
                    &data[..PNG_MAGIC_NUMBER_LEN],
                    PNG_MAGIC_NUMBER,
                )
            {
                playlist.set_png_cover(data.as_slice())?;
            } else if data.len() >= JPG_MAGIC_NUMBER_LEN
                && constant_time_eq::constant_time_eq(
                    &data[..JPG_MAGIC_NUMBER_LEN],
                    JPG_MAGIC_NUMBER,
                )
            {
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
