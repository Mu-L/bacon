use std::{
    fmt,
    io,
    path::PathBuf,
};

#[derive(Debug)]
pub enum SoundError {
    Interrupted,
    UnknownSoundName(String),
    Read(PathBuf, io::Error),
    RodioStream(rodio::StreamError),
    RodioPlay(rodio::PlayError),
}
impl From<rodio::StreamError> for SoundError {
    fn from(e: rodio::StreamError) -> Self {
        SoundError::RodioStream(e)
    }
}
impl From<rodio::PlayError> for SoundError {
    fn from(e: rodio::PlayError) -> Self {
        SoundError::RodioPlay(e)
    }
}
impl fmt::Display for SoundError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            SoundError::Interrupted => write!(f, "sound interrupted"),
            SoundError::UnknownSoundName(name) => {
                write!(f, "unknown sound name: {}", name)
            }
            SoundError::Read(path, e) => {
                write!(f, "sound file {} can't be read: {}", path.display(), e)
            }
            SoundError::RodioStream(e) => write!(f, "rodio stream error: {}", e),
            SoundError::RodioPlay(e) => write!(f, "rodio play error: {}", e),
        }
    }
}

impl std::error::Error for SoundError {}
