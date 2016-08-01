use audio::Audio;
use instrument;
use map::Map;
use pitch;
use sampler::PlayingSample;
use std;
use Velocity;

pub use instrument::mode::{Mono, MonoKind, Poly, Dynamic};

/// The "mode" with which the Sampler will handle notes.
pub trait Mode {

    /// Handle a `note_on` event.
    ///
    /// Is called immediately following `instrument::Mode::note_on`.
    fn note_on<A>(&self,
                  note_hz: pitch::Hz,
                  note_velocity: Velocity,
                  map: &Map<A>,
                  voices: &mut [Option<PlayingSample<A>>])
        where A: Audio;

    /// Handle a `note_off` event.
    fn note_off<A>(&self,
                   note_hz: pitch::Hz,
                   map: &Map<A>,
                   voices: &mut [Option<PlayingSample<A>>])
        where A: Audio;
}


// Helper function for constructing a `PlayingSample`.
fn play_sample<A>(hz: pitch::Hz, vel: Velocity, map: &Map<A>) -> Option<PlayingSample<A>>
    where A: Audio,
{
    play_sample_from_playhead_idx(0, hz, vel, map)
}

// Helper function for constructing a `PlayingSample` with a given playhead index.
fn play_sample_from_playhead_idx<A>(idx: usize,
                                    hz: pitch::Hz,
                                    vel: Velocity,
                                    map: &Map<A>) -> Option<PlayingSample<A>>
    where A: Audio,
{
    map.sample(hz, vel).map(|sample| PlayingSample::from_playhead_idx(idx, hz, vel, sample))
}


impl Mode for Mono {

    fn note_on<A>(&self,
                  note_hz: pitch::Hz,
                  note_vel: Velocity,
                  map: &Map<A>,
                  voices: &mut [Option<PlayingSample<A>>])
        where A: Audio,
    {
        let Mono(ref kind, ref note_stack) = *self;

        // If we're in `Legato` mode, begin the note from the same index as the previous note's
        // current state if there is one.
        let sample = if let instrument::mode::MonoKind::Legato = *kind {
            note_stack.last()
                .and_then(|&last_hz| {
                    voices.iter()
                        .filter_map(|v| v.as_ref())
                        .find(|sample| instrument::mode::does_hz_match(sample.note_on_hz.hz(), last_hz))
                        .and_then(|sample| {
                            let idx = sample.rate_converter.source().idx;
                            play_sample_from_playhead_idx(idx, note_hz, note_vel, map)
                        })
                })
                .or_else(|| play_sample(note_hz, note_vel, map))
        // Otherwise, we're in `Retrigger` mode, so start from the beginning of the sample.
        } else {
            play_sample(note_hz, note_vel, map)
        };

        if let Some(sample) = sample {
            for voice in voices {
                *voice = Some(sample.clone());
            }
        }
    }

    fn note_off<A>(&self,
                   note_hz: pitch::Hz,
                   map: &Map<A>,
                   voices: &mut [Option<PlayingSample<A>>])
        where A: Audio,
    {
        let Mono(kind, ref note_stack) = *self;

        let should_reset = voices.iter()
            .filter_map(|v| v.as_ref())
            .any(|v| instrument::mode::does_hz_match(v.note_on_hz.hz(), note_hz.hz()));

        if !should_reset {
            return;
        }

        // If there is some note to fall back to, do so.
        if let Some(&fallback_note_hz) = note_stack.last() {
            let hz = fallback_note_hz.into();
            for voice in voices {
                if let Some(ref mut playing_sample) = *voice {
                    let idx = match kind {
                        MonoKind::Retrigger => 0,
                        MonoKind::Legato => playing_sample.rate_converter.source().idx,
                    };
                    let vel = playing_sample.note_on_vel;
                    if let Some(sample) = play_sample_from_playhead_idx(idx, hz, vel, map) {
                        *playing_sample = sample;
                    }
                }
            }
        }

        // No need to manually set voices to `None` as this will be done when frames yielded by
        // `instrument` run out.
    }

}

impl Mode for Poly {

    fn note_on<A>(&self,
                  note_hz: pitch::Hz,
                  note_vel: Velocity,
                  map: &Map<A>,
                  voices: &mut [Option<PlayingSample<A>>])
        where A: Audio,
    {
        let sample = match play_sample(note_hz, note_vel, map) {
            Some(sample) => sample,
            None => return,
        };

        // Find the right voice to play the note.
        let mut oldest = None;
        let mut oldest_time_of_note_on = std::time::Instant::now();
        for voice in voices.iter_mut() {
            if let None = *voice {
                *voice = Some(sample);
                return;
            }
            let time_of_note_on = voice.as_ref().unwrap().time_of_note_on;
            if time_of_note_on < oldest_time_of_note_on {
                oldest_time_of_note_on = time_of_note_on;
                oldest = voice.as_mut();
            }
        }
        if let Some(voice) = oldest {
            *voice = sample;
        }
    }

    fn note_off<A>(&self,
                   _note_hz: pitch::Hz,
                   _map: &Map<A>,
                   _voices: &mut [Option<PlayingSample<A>>])
        where A: Audio,
    {
        // No need to do anything here as voices will be set to `None` when frames yielded by
        // `instrument` run out.
    }

}

impl Mode for Dynamic {

    fn note_on<A>(&self,
                  note_hz: pitch::Hz,
                  note_vel: Velocity,
                  map: &Map<A>,
                  voices: &mut [Option<PlayingSample<A>>])
        where A: Audio,
    {
        match *self {
            Dynamic::Mono(ref mono) => mono.note_on(note_hz, note_vel, map, voices),
            Dynamic::Poly(ref poly) => poly.note_on(note_hz, note_vel, map, voices),
        }
    }

    fn note_off<A>(&self,
                   note_hz: pitch::Hz,
                   map: &Map<A>,
                   voices: &mut [Option<PlayingSample<A>>])
        where A: Audio,
    {
        match *self {
            Dynamic::Mono(ref mono) => mono.note_off(note_hz, map, voices),
            Dynamic::Poly(ref poly) => poly.note_off(note_hz, map, voices),
        }
    }

}
