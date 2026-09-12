use {
    crate::*,
    schemars::{
        JsonSchema,
        Schema,
        SchemaGenerator,
        json_schema,
    },
    serde::{
        Deserialize,
        Deserializer,
        de,
    },
    std::{
        borrow::Cow,
        fmt,
        path::{
            Path,
            PathBuf,
        },
        time::Duration,
    },
};
#[cfg(feature = "sound")]
use {
    rodio::OutputStreamBuilder,
    std::io::Cursor,
    termimad::crossbeam::channel::{
        Receiver,
        RecvTimeoutError,
    },
};

/// How often the end of a sound of unknown duration is checked
#[cfg(feature = "sound")]
const END_CHECK_PERIOD: Duration = Duration::from_millis(100);

/// A sound, either embedded in the bacon executable or given by the
/// path of an audio file, with an optional duration after which
/// playing is cut.
///
/// In configuration, a sound is either the path of its file, or a table
/// with a path and a duration, eg
///
/// ```TOML
/// [sounds]
/// bepop = "~/audio/bepop.mp3"
/// success = { path = "~/audio/tada.ogg", duration = "1500ms" }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sound {
    source: SoundSource,
    cut: Option<Duration>,
}

#[derive(Clone, PartialEq, Eq)]
enum SoundSource {
    Embedded(&'static [u8]),
    File(PathBuf),
}

impl Sound {
    /// Build a sound from bytes embedded in the executable, cut after the
    /// given duration
    pub const fn embedded(
        bytes: &'static [u8],
        cut_millis: u64,
    ) -> Self {
        Self {
            source: SoundSource::Embedded(bytes),
            cut: Some(Duration::from_millis(cut_millis)),
        }
    }
    /// Duration after which the sound is cut, if any
    pub fn cut(&self) -> Option<Duration> {
        self.cut
    }
    /// The path of the sound file, when the sound isn't embedded
    pub fn path(&self) -> Option<&Path> {
        match &self.source {
            SoundSource::Embedded(_) => None,
            SoundSource::File(path) => Some(path),
        }
    }
    /// Expand the tilde and make the path absolute, relative paths
    /// being taken from the given directory
    pub fn resolve_path(
        &mut self,
        base_dir: &Path,
    ) {
        if let SoundSource::File(path) = &mut self.source {
            let expanded = expand_tilde(path);
            *path = if expanded.is_absolute() {
                expanded.into_owned()
            } else {
                base_dir.join(expanded)
            };
        }
    }
}

impl fmt::Debug for SoundSource {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Self::Embedded(bytes) => write!(f, "embedded sound of {} bytes", bytes.len()),
            Self::File(path) => write!(f, "{}", path.display()),
        }
    }
}

#[cfg(feature = "sound")]
impl Sound {
    /// Check the sound can be read, without decoding it
    pub fn check(&self) -> Result<(), SoundError> {
        if let SoundSource::File(path) = &self.source {
            std::fs::metadata(path).map_err(|e| SoundError::Read(path.clone(), e))?;
        }
        Ok(())
    }
    /// The encoded audio data, read from the file if the sound isn't embedded
    fn bytes(&self) -> Result<Cow<'static, [u8]>, SoundError> {
        match &self.source {
            SoundSource::Embedded(bytes) => Ok(Cow::Borrowed(bytes)),
            SoundSource::File(path) => std::fs::read(path)
                .map(Cow::Owned)
                .map_err(|e| SoundError::Read(path.clone(), e)),
        }
    }
    /// Play the sound, returning when it's finished, cut, or interrupted
    pub fn play(
        &self,
        volume: Volume,
        interrupt: &Receiver<()>,
    ) -> Result<(), SoundError> {
        let bytes = self.bytes()?;
        let stream = OutputStreamBuilder::open_default_stream()?;
        let sink = rodio::play(stream.mixer(), Cursor::new(bytes))?;
        sink.set_volume(volume.as_part());
        match self.cut {
            Some(cut) => {
                if is_interrupted(interrupt, cut) {
                    return Err(SoundError::Interrupted);
                }
            }
            None => {
                while !sink.empty() {
                    if is_interrupted(interrupt, END_CHECK_PERIOD) {
                        return Err(SoundError::Interrupted);
                    }
                }
            }
        }
        Ok(())
    }
}

/// Wait for the given duration, telling whether an interruption was received
/// (a disconnected channel means the sound player is gone)
#[cfg(feature = "sound")]
fn is_interrupted(
    interrupt: &Receiver<()>,
    duration: Duration,
) -> bool {
    !matches!(
        interrupt.recv_timeout(duration),
        Err(RecvTimeoutError::Timeout)
    )
}

impl<'de> Deserialize<'de> for Sound {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(SoundVisitor)
    }
}

struct SoundVisitor;
impl<'de> de::Visitor<'de> for SoundVisitor {
    type Value = Sound;
    fn expecting(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            f,
            "a sound file path, or a table with a path and a duration"
        )
    }
    fn visit_str<E: de::Error>(
        self,
        path: &str,
    ) -> Result<Sound, E> {
        Ok(Sound {
            source: SoundSource::File(path.into()),
            cut: None,
        })
    }
    fn visit_map<M: de::MapAccess<'de>>(
        self,
        mut map: M,
    ) -> Result<Sound, M::Error> {
        let mut path: Option<PathBuf> = None;
        let mut cut: Option<Duration> = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_ref() {
                "path" => {
                    path = Some(map.next_value()?);
                }
                "duration" => {
                    // a zero duration (eg "none") means no cut
                    let period = map.next_value::<Period>()?;
                    cut = (!period.is_zero()).then_some(period.duration);
                }
                _ => {
                    return Err(de::Error::unknown_field(&key, &["path", "duration"]));
                }
            }
        }
        let path = path.ok_or_else(|| de::Error::missing_field("path"))?;
        Ok(Sound {
            source: SoundSource::File(path),
            cut,
        })
    }
}

impl JsonSchema for Sound {
    fn schema_name() -> Cow<'static, str> {
        "Sound".into()
    }
    fn schema_id() -> Cow<'static, str> {
        concat!(module_path!(), "::Sound").into()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "oneOf": [
                {
                    "type": "string",
                    "description": "Path of the sound file.",
                },
                {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "duration": { "type": "string" },
                    },
                    "required": ["path"],
                },
            ],
        })
    }
    fn inline_schema() -> bool {
        true
    }
}
