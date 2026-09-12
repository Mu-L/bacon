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
        path::Path,
    },
};

/// Sounds by name.
///
/// The default library, embedded in the executable, is built at compile
/// time, while the one of the user comes from the `sounds` config map and
/// is applied on top of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundLibrary {
    sounds: Cow<'static, [(Cow<'static, str>, Sound)]>,
}

impl Default for SoundLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl SoundLibrary {
    pub const fn new() -> Self {
        Self {
            sounds: Cow::Borrowed(&[]),
        }
    }
    pub const fn from_static(sounds: &'static [(Cow<'static, str>, Sound)]) -> Self {
        Self {
            sounds: Cow::Borrowed(sounds),
        }
    }
    pub fn get(
        &self,
        name: &str,
    ) -> Option<&Sound> {
        self.sounds
            .iter()
            .find(|(n, _)| n.as_ref() == name)
            .map(|(_, sound)| sound)
    }
    pub fn set(
        &mut self,
        name: Cow<'static, str>,
        sound: Sound,
    ) {
        let sounds = self.sounds.to_mut();
        match sounds.iter_mut().find(|(n, _)| *n == name) {
            Some((_, s)) => *s = sound,
            None => sounds.push((name, sound)),
        }
    }
    /// Expand the tildes and make the paths absolute, relative paths
    /// being taken from the given directory
    pub fn resolve_paths(
        &mut self,
        base_dir: &Path,
    ) {
        for (_, sound) in self.sounds.to_mut() {
            sound.resolve_path(base_dir);
        }
    }
    /// Add the sounds of the other library, replacing those of same name
    pub fn apply(
        &mut self,
        library: &SoundLibrary,
    ) {
        for (name, sound) in library.sounds.iter() {
            self.set(name.clone(), sound.clone());
        }
    }
}

impl<'de> Deserialize<'de> for SoundLibrary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(SoundLibraryVisitor)
    }
}

struct SoundLibraryVisitor;
impl<'de> de::Visitor<'de> for SoundLibraryVisitor {
    type Value = SoundLibrary;
    fn expecting(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(f, "a map of sounds by name")
    }
    fn visit_map<M: de::MapAccess<'de>>(
        self,
        mut map: M,
    ) -> Result<SoundLibrary, M::Error> {
        let mut sounds = Vec::new();
        while let Some((name, sound)) = map.next_entry::<String, Sound>()? {
            check_sound_name(&name).map_err(de::Error::custom)?;
            sounds.push((Cow::Owned(name), sound));
        }
        Ok(SoundLibrary {
            sounds: Cow::Owned(sounds),
        })
    }
}

/// Ensure the name can be used in a `play-sound(name=...)` action,
/// whose parsing splits on ',' and '=' and trims the values
fn check_sound_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.trim() != name || name.contains([',', '=']) {
        return Err(format!(
            "invalid sound name {name:?}: a name can't be empty, \
             be padded with spaces, or contain ',' or '='"
        ));
    }
    Ok(())
}

impl JsonSchema for SoundLibrary {
    fn schema_name() -> Cow<'static, str> {
        "SoundLibrary".into()
    }
    fn schema_id() -> Cow<'static, str> {
        concat!(module_path!(), "::SoundLibrary").into()
    }
    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "object",
            "description": "Sound files, keyed by the name used in the play-sound action.",
            "additionalProperties": Sound::json_schema(generator),
        })
    }
}

#[test]
fn test_sound_name_check() {
    for name in ["bepop", "90s-game-ui-6", "a b"] {
        assert!(check_sound_name(name).is_ok(), "{name:?} should be valid");
    }
    for name in ["", " bepop", "bepop ", "a,b", "a=b"] {
        assert!(
            check_sound_name(name).is_err(),
            "{name:?} should be invalid"
        );
    }
    let err = toml::from_str::<SoundLibrary>(r#""a,b" = "/tmp/a.mp3""#).unwrap_err();
    assert!(err.to_string().contains("invalid sound name"));
}

#[test]
fn test_sound_path_resolution() {
    let mut library: SoundLibrary = toml::from_str(
        r#"
        rel = "audio/bepop.mp3"
        abs = "/opt/audio/tada.ogg"
        "#,
    )
    .unwrap();
    library.resolve_paths(Path::new("/project"));
    assert_eq!(
        library.get("rel").unwrap().path(),
        Some(Path::new("/project/audio/bepop.mp3"))
    );
    assert_eq!(
        library.get("abs").unwrap().path(),
        Some(Path::new("/opt/audio/tada.ogg"))
    );
}

#[test]
fn test_sound_library_deserialization() {
    use std::time::Duration;
    let library: SoundLibrary = toml::from_str(
        r#"
        bepop = "~/audio/bepop.mp3"
        tada = { path = "/tmp/tada.ogg", duration = "3s" }
        whole = { path = "/tmp/whole.ogg", duration = "none" }
        "#,
    )
    .unwrap();
    assert_eq!(library.get("bepop").unwrap().cut(), None);
    assert_eq!(library.get("whole").unwrap().cut(), None);
    assert_eq!(
        library.get("tada").unwrap().cut(),
        Some(Duration::from_secs(3))
    );
    assert!(library.get("unknown").is_none());
}
