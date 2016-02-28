use {pitch, time, Velocity};

pub type Playhead = time::Samples;

/// The current state of the Voice's note playback.
#[derive(Copy, Clone, Debug)]
pub enum NoteState {
    /// The note is current playing.
    Playing,
    /// The note has been released and is fading out.
    Released(Playhead),
}

/// A single monophonic voice of a `Sampler`.
#[derive(Clone, Debug, PartialEq)]
pub struct Voice {
    note: Option<(NoteState, pitch::Hz, Velocity)>,
    playhead: Playhead,
}

impl Voice {

    /// Construct a new `Voice`.
    pub fn new() -> Self {
        Voice {
            note: None,
            playhead: 0,
        }
    }

    /// Trigger playback with the given note.
    #[inline]
    pub fn note_on(&mut self, hz: NoteHz, vel: NoteVelocity) {
        self.maybe_note = Some((NoteState::Playing, hz, vel));
    }

    /// Release playback of the current note if there is one.
    #[inline]
    pub fn note_off(&mut self) {
        if let Some(&mut(ref mut state, _, _)) = self.note.as_mut() {
            *state = NoteState::Released(0);
        }
    }

    pub fn fill_buffer(&mut self, buffer: &mut [S]) {
    }

}
