#[cfg(feature = "sound")]
mod default_sounds;
#[cfg(not(feature = "sound"))]
mod no_sound;
#[allow(clippy::module_inception)]
mod sound;
mod sound_config;
#[cfg(feature = "sound")]
mod sound_error;
mod sound_library;
#[cfg(feature = "sound")]
mod sound_player;
mod volume;

#[cfg(not(feature = "sound"))]
pub use no_sound::*;
#[cfg(feature = "sound")]
pub use {
    default_sounds::*,
    sound_error::*,
    sound_player::*,
};
pub use {
    sound::*,
    sound_config::*,
    sound_library::*,
    volume::*,
};

/// A command to play a sound
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PlaySoundCommand {
    pub name: Option<String>,
    pub volume: Volume,
}
