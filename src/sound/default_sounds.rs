use {
    crate::*,
    std::borrow::Cow,
};

/// Name of the sound played when the `play-sound` action doesn't specify one
pub const DEFAULT_SOUND_NAME: &str = "store-scanner";

const fn embedded(
    name: &'static str,
    bytes: &'static [u8],
    cut_millis: u64,
) -> (Cow<'static, str>, Sound) {
    (Cow::Borrowed(name), Sound::embedded(bytes, cut_millis))
}

/// The sounds embedded in the bacon executable.
///
/// Names here are as near as possible from the file names in the
/// resources directory but without the number, syntax inconsistency and
/// redundancy. Resource file names are kept identical to their original
/// names to ease retrieval for attribution).
///
/// Durations cut the files, which are usually much longer than what can
/// be heard in them.
pub static DEFAULT_SOUNDS: SoundLibrary = SoundLibrary::from_static(EMBEDDED_SOUNDS);

static EMBEDDED_SOUNDS: &[(Cow<'static, str>, Sound)] = &[
    embedded("2", include_bytes!("../../resources/2-100419.mp3"), 2000),
    embedded(
        "90s-game-ui-6",
        include_bytes!("../../resources/90s-game-ui-6-185099.mp3"),
        1300,
    ),
    embedded(
        "beep-6",
        include_bytes!("../../resources/beep-6-96243.mp3"),
        1000,
    ),
    embedded(
        "beep-beep",
        include_bytes!("../../resources/beep-beep-6151.mp3"),
        1200,
    ),
    embedded(
        "beep-warning",
        include_bytes!("../../resources/beep-warning-6387.mp3"),
        1200,
    ),
    embedded(
        "bell-chord",
        include_bytes!("../../resources/bell-chord1-83260.mp3"),
        1900,
    ),
    embedded(
        "car-horn",
        include_bytes!("../../resources/car-horn-beepsmp3-14659.mp3"),
        1700,
    ),
    embedded(
        "convenience-store-ring",
        include_bytes!("../../resources/conveniencestorering-96090.mp3"),
        1700,
    ),
    embedded(
        "cow-bells",
        include_bytes!("../../resources/cow_bells_01-98236.mp3"),
        1400,
    ),
    embedded(
        "pickup",
        include_bytes!("../../resources/pickup-sound-46472.mp3"),
        500,
    ),
    embedded(
        "positive-beeps",
        include_bytes!("../../resources/positive_beeps-85504.mp3"),
        600,
    ),
    embedded(
        "short-beep-tone",
        include_bytes!("../../resources/short-beep-tone-47916.mp3"),
        400,
    ),
    embedded(
        "slash",
        include_bytes!("../../resources/slash1-94367.mp3"),
        800,
    ),
    embedded(
        "store-scanner",
        include_bytes!("../../resources/store-scanner-beep-90395.mp3"),
        250,
    ),
    embedded(
        "success",
        include_bytes!("../../resources/success-48018.mp3"),
        2000,
    ),
];

/// Check the coherence of the hand written list of embedded sounds
#[test]
fn test_default_sounds() {
    assert!(DEFAULT_SOUNDS.get(DEFAULT_SOUND_NAME).is_some());
    let mut names: Vec<&str> = EMBEDDED_SOUNDS
        .iter()
        .map(|(name, _)| name.as_ref())
        .collect();
    let count = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), count, "duplicate sound name");
}
