use {
    super::*,
    std::path::Path,
};

/// A dummy sound player which does nothing
pub struct SoundPlayer {}
impl SoundPlayer {
    pub fn new(
        _base_volume: Volume,
        _library: &SoundLibrary,
        _base_dir: &Path,
    ) -> anyhow::Result<Self> {
        Err(anyhow::anyhow!(
            "Bacon is compiled without the sound feature"
        ))
    }
    pub fn play(
        &self,
        _psc: &PlaySoundCommand,
    ) -> anyhow::Result<()> {
        // should never be called as the  sound player is not instantiated
        Ok(())
    }
}
