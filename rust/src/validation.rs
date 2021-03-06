use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlaylistError {
    #[error("playlist field `{field}` has value of `{value}` which doesn't respect the schema")]
    InvalidField { field: &'static str, value: String },
    #[error(transparent)]
    InvalidCover(#[from] PlaylistCoverError),
    #[error("beatmap at index `{idx}` is invalid: {error}")]
    InvalidBeatmap {
        idx: usize,
        #[source]
        error: BeatmapError,
    },
}

#[derive(Debug, Error)]
pub enum PlaylistCoverError {
    #[error("playlist cover has an unknown type")]
    UnknownCoverType,
    #[error("playlist cover of type `{ty}` has invalid path `{}`", .path.display())]
    InvalidCoverPath { ty: &'static str, path: PathBuf },
    #[error("playlist cover of type `{ty}` has invalid data")]
    InvalidCoverData { ty: &'static str },
}

#[derive(Debug, Error)]
pub enum BeatmapError {
    #[error("missing field `{field}` in beatmap of type `{ty}`")]
    MismatchedType {
        ty: &'static str,
        field: &'static str,
    },
    #[error("beatmap field `{field}` has value of `{value}` which doesn't respect the schema")]
    InvalidField { field: &'static str, value: String },
    #[error("beatmap difficulty at index `{idx}` is invalid: {error}")]
    InvalidDifficulty {
        idx: usize,
        #[source]
        error: BeatmapDifficultyError,
    },
}

#[derive(Debug, Error)]
pub enum BeatmapDifficultyError {
    #[error("beatmap difficulty field `{field}` has value of `{value}` which doesn't respect the schema")]
    InvalidField { field: &'static str, value: String },
}
