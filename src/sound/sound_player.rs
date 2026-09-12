use {
    super::*,
    std::{
        path::Path,
        thread,
    },
    termimad::crossbeam::channel::{
        self,
        Receiver,
        Sender,
        TrySendError,
        select,
    },
};

/// Manage a thread to play sounds without blocking bacon
pub struct SoundPlayer {
    thread: Option<thread::JoinHandle<()>>,
    s_die: Option<Sender<()>>,
    s_sound: Sender<(Sound, Volume)>,
    r_sound: Receiver<(Sound, Volume)>,
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
        // only one sound waits, and it's always the most recent request
        let (s_sound, r_sound) = channel::bounded::<(Sound, Volume)>(1);
        let r_played = r_sound.clone();
        let (s_die, r_die) = channel::bounded(1);
        let thread = thread::spawn(move || {
            loop {
                select! {
                    recv(r_die) -> _ => {
                        info!("sound player thread is stopping");
                        break;
                    }
                    recv(r_played) -> received => {
                        match received {
                            Ok((sound, volume)) => {
                                if !r_die.is_empty() {
                                    continue;
                                }
                                let interrupter = Interrupter::new(&r_die, &r_played);
                                match sound.play(volume * base_volume, &interrupter) {
                                    Ok(None) => {
                                        debug!("sound played");
                                    }
                                    Ok(Some(Interruption::NewSound)) => {
                                        debug!("sound cut by the next one");
                                    }
                                    Ok(Some(Interruption::Die)) => {
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
            r_sound,
            sounds,
        })
    }
    /// Request a sound, which replaces the one waiting to be played, if any.
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
        let request = (sound.clone(), psc.volume);
        if let Err(TrySendError::Full(request)) = self.s_sound.try_send(request) {
            // the waiting sound is older than this one, it's not worth playing
            let _ = self.r_sound.try_recv();
            if self.s_sound.try_send(request).is_err() {
                warn!("sound request dropped");
            }
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
