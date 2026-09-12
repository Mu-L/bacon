use {
    super::*,
    std::{
        path::Path,
        thread,
    },
    termimad::crossbeam::channel::{
        self,
        Sender,
        select,
    },
};

/// The maximum number of sounds that can be queued, as we don't want
/// a long list of sounds for events that are triggered in a loop
/// (on quit, the queue will be interrupted anyway).
const MAX_QUEUED_SOUNDS: usize = 2;

/// Manage a thread to play sounds without blocking bacon
pub struct SoundPlayer {
    thread: Option<thread::JoinHandle<()>>,
    s_die: Option<Sender<()>>,
    s_sound: Sender<(Sound, Volume)>,
    sounds: SoundLibrary,
}
impl SoundPlayer {
    pub fn new(
        base_volume: Volume,
        library: &SoundLibrary,
        base_dir: &Path,
    ) -> anyhow::Result<Self> {
        let mut sounds = DEFAULT_SOUNDS.clone();
        sounds.apply(library);
        sounds.resolve_paths(base_dir);
        let (s_sound, r_sound) = channel::bounded::<(Sound, Volume)>(MAX_QUEUED_SOUNDS);
        let (s_die, r_die) = channel::bounded(1);
        let thread = thread::spawn(move || {
            loop {
                select! {
                    recv(r_die) -> _ => {
                        info!("sound player thread is stopping");
                        break;
                    }
                    recv(r_sound) -> received => {
                        match received {
                            Ok((sound, volume)) => {
                                if !r_die.is_empty() {
                                    continue;
                                }
                                match sound.play(volume * base_volume, &r_die) {
                                    Ok(()) => {
                                        debug!("sound played");
                                    }
                                    Err(SoundError::Interrupted) => {
                                        // only reason is sound player is dying
                                        info!("sound interrupted");
                                        break;
                                    }
                                    Err(e) => {
                                        error!("sound error: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("sound player channel error: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });
        Ok(Self {
            thread: Some(thread),
            s_die: Some(s_die),
            s_sound,
            sounds,
        })
    }
    /// Request a sound, unless too many of them are already queued.
    ///
    /// The sound is resolved here, so that a bad name or a missing file
    /// is reported to the caller instead of being buried in the log.
    pub fn play(
        &self,
        psc: &PlaySoundCommand,
    ) -> Result<(), SoundError> {
        debug!("play sound: {psc:#?}");
        let name = psc.name.as_deref().unwrap_or(DEFAULT_SOUND_NAME);
        let sound = self
            .sounds
            .get(name)
            .ok_or_else(|| SoundError::UnknownSoundName(name.to_string()))?;
        sound.check()?;
        if self.s_sound.try_send((sound.clone(), psc.volume)).is_err() {
            warn!("Too many sounds in the queue, dropping one");
        }
        Ok(())
    }
    /// Make the beeper thread synchronously stop
    /// (interrupting the current sound if any)
    pub fn die(&mut self) {
        if let Some(sender) = self.s_die.take() {
            if let Err(e) = sender.send(()) {
                warn!("failed to send 'kill' signal: {e}");
            }
        }
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                warn!("child_thread.join() failed"); // should not happen
            } else {
                info!("SoundPlayer gracefully stopped");
            }
        }
    }
}
impl Drop for SoundPlayer {
    fn drop(&mut self) {
        self.die();
    }
}
