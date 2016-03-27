pub use instrument::mode::{Mono, MonoKind, Poly, Dynamic};
use map::Map;
use pitch;
use sample::Frame;
use sampler::PlayingSample;
use Velocity;

/// The "mode" with which the Sampler will handle notes.
pub trait Mode {

    /// Handle a `note_on` event.
    ///
    /// Is called immediately following `instrument::Mode::note_on`.
    fn note_on<F>(&mut self,
                  note_hz: pitch::Hz,
                  note_velocity: Velocity,
                  map: &Map<F>,
                  voices: &mut [Option<PlayingSample<F>>])
        where F: Frame;

    /// Handle a `note_off` event.
    fn note_off<F>(&self,
                   note_hz: pitch::Hz,
                   map: &Map<F>,
                   voices: &mut [Option<PlayingSample<F>>])
        where F: Frame;

    /// Handle a `stop` event.
    fn stop(&mut self) {}
}


// Helper function for constructing a `PlayingSample`.
fn play_sample<F>(hz: pitch::Hz, vel: Velocity, map: &Map<F>) -> Option<PlayingSample<F>>
    where F: Frame,
{
    play_sample_from_playhead_idx(0, hz, vel, map)
}

// Helper function for constructing a `PlayingSample` with a given playhead index.
fn play_sample_from_playhead_idx<F>(idx: usize,
                                    hz: pitch::Hz,
                                    vel: Velocity,
                                    map: &Map<F>) -> Option<PlayingSample<F>>
    where F: Frame,
{
    map.sample(hz, vel).map(|sample| PlayingSample::from_playhead_idx(idx, hz, vel, sample))
}


impl Mode for Mono {

    fn note_on<F>(&mut self,
                  note_hz: pitch::Hz,
                  note_vel: Velocity,
                  map: &Map<F>,
                  voices: &mut [Option<PlayingSample<F>>])
        where F: Frame,
    {
        let sample = match play_sample(note_hz, note_vel, map) {
            Some(sample) => sample,
            None => return,
        };
        for voice in voices {
            *voice = Some(sample.clone());
        }
    }

    fn note_off<F>(&self,
                   note_hz: pitch::Hz,
                   map: &Map<F>,
                   voices: &mut [Option<PlayingSample<F>>])
        where F: Frame,
    {
        let Mono(kind, ref note_stack) = *self;

        let should_reset = voices.iter().next()
            .and_then(|v| v.as_ref().map(|v| v.note_on_hz == note_hz))
            .unwrap_or(false);

        if should_reset {
            let maybe_fallback_note_hz = note_stack.iter().last();
            for voice in voices {
                // If there's some fallback note in the note stack, play it.
                if let Some(ref mut playing_sample) = *voice {
                    if let Some(&hz) = maybe_fallback_note_hz {
                        let hz = pitch::Hz(hz.into());
                        let idx = match kind {
                            MonoKind::Legato => playing_sample.rate_converter.source().idx,
                            MonoKind::Retrigger => 0,
                        };
                        let vel = playing_sample.note_on_vel;
                        if let Some(sample) = play_sample_from_playhead_idx(idx, hz, vel, map) {
                            *playing_sample = sample;
                            continue;
                        }
                    }
                }
                // Otherwise, set the voices to `None`.
                *voice = None;
            }
        }
    }

}

impl Mode for Poly {

    fn note_on<F>(&mut self,
                  note_hz: pitch::Hz,
                  note_vel: Velocity,
                  map: &Map<F>,
                  voices: &mut [Option<PlayingSample<F>>])
        where F: Frame,
    {
        let sample = match play_sample(note_hz, note_vel, map) {
            Some(sample) => sample,
            None => return,
        };

        // Find the right voice to play the note.
        let mut oldest = None;
        let mut max_sample_count = 0;
        for voice in voices.iter_mut() {
            match *voice {
                None => {
                    *voice = Some(sample);
                    return;
                },
                Some(ref mut playing_sample) => {
                    let playhead = playing_sample.rate_converter.source().idx;
                    if playhead >= max_sample_count {
                        max_sample_count = playhead;
                        oldest = Some(playing_sample);
                    }
                },
            }
        }
        if let Some(voice) = oldest {
            *voice = sample;
        }
    }

    fn note_off<F>(&self,
                   note_hz: pitch::Hz,
                   _map: &Map<F>,
                   voices: &mut [Option<PlayingSample<F>>])
        where F: Frame,
    {
        for voice in voices {
            let should_reset = voice.as_ref().map(|v| v.note_on_hz == note_hz).unwrap_or(false);
            if should_reset {
                *voice = None;
            }
        }
    }

}
