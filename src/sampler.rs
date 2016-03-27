use instrument::{self, Instrument};
use map::{self, Map};
use pitch;
use sample::{self, Frame, Sample as PcmSample};
use time;
use Velocity;

/// A Sampler instrument.
#[derive(Clone)]
pub struct Sampler<M, NFG, F>
    where NFG: instrument::NoteFreqGenerator,
          F: Frame,
{
    map: Map<F>,
    voices: Voices<F>,
    instrument: Instrument<M, NFG>,
}

/// Samples that are currently active along with the `Hz` with which they were triggered.
///
/// A new pair is pushed on each `note_on`, and pairs are removed on their associated `note_off`.
///
/// In `Mono` mode, the sampler always fills the buffer using the last pair on the stack.
///
/// In `Poly` mode, each pair is mapped directly to each of the `Instrument`'s `voices` via their
/// `Vec` indices.
#[derive(Clone)]
pub struct Voices<F>
    where F: Frame,
{
    map: Vec<Option<PlayingSample<F>>>,
}

/// A sample that is currently being played back.
#[derive(Clone)]
pub struct PlayingSample<F>
    where F: Frame,
{
    /// The pitch in hz at which the `note_on` was triggered.
    pub note_on_hz: pitch::Hz,
    pub note_on_vel: Velocity,
    base_hz: pitch::Hz,
    base_vel: Velocity,
    /// Rate-adjustable interpolation of audio.
    pub rate_converter: sample::rate::Converter<Playhead<F>>,
}

/// An owned iterator that wraps an audio file but does not 
#[derive(Clone)]
pub struct Playhead<F>
    where F: Frame,
{
    /// The position of the playhead over the `Sample`.
    pub idx: usize,
    audio: map::Audio<F>,
}

/// An iterator yielding one frame from the `Sampler` at a time.
pub struct Frames<'a, F: 'a, NF: 'a>
    where F: Frame,
{
    voices: &'a mut Voices<F>,
    instrument_frames: instrument::Frames<'a, NF>,
}



impl<NFG, F> Sampler<instrument::mode::Mono, NFG, F>
    where NFG: instrument::NoteFreqGenerator,
          F: Frame,
{
    /// Construct a `Sampler` with a `Mono::Legato` playback mode.
    pub fn legato(nfg: NFG, map: Map<F>) -> Self {
        Self::new(instrument::mode::Mono::legato(), nfg, map)
    }
}

impl<NFG, F> Sampler<instrument::mode::Mono, NFG, F>
    where NFG: instrument::NoteFreqGenerator,
          F: Frame,
{
    /// Construct a `Sampler` with a `Mono::Retrigger` playback mode.
    pub fn retrigger(nfg: NFG, map: Map<F>) -> Self {
        Self::new(instrument::mode::Mono::retrigger(), nfg, map)
    }
}

impl<NFG, F> Sampler<instrument::mode::Poly, NFG, F>
    where NFG: instrument::NoteFreqGenerator,
          F: Frame,
{
    /// Construct a `Sampler` with a `Poly` playback mode.
    pub fn poly(nfg: NFG, map: Map<F>) -> Self {
        Self::new(instrument::mode::Poly, nfg, map)
    }
}


impl<M, NFG, F> Sampler<M, NFG, F>
    where NFG: instrument::NoteFreqGenerator,
          F: Frame,
{

    /// Construct a new `Sampler`.
    pub fn new(mode: M, note_freq_gen: NFG, map: Map<F>) -> Self {
        let instrument = Instrument::new(mode, note_freq_gen);
        let n_voices = instrument.voices.len();
        Sampler {
            map: map,
            voices: Voices { map: vec![None; n_voices] },
            instrument: instrument,
        }
    }

    /// Map the `Instrument` to a new `Instrument` in place.
    ///
    /// This is useful for providing wrapper builder methods for the Synth.
    #[inline]
    pub fn map_instrument<Map, NewM, NewNFG>(self, f: Map) -> Sampler<NewM, NewNFG, F>
        where Map: FnOnce(Instrument<M, NFG>) -> Instrument<NewM, NewNFG>,
              NewNFG: instrument::NoteFreqGenerator,
    {
        let Sampler {
            map,
            voices,
            instrument,
        } = self;

        Sampler {
            map: map,
            voices: voices,
            instrument: f(instrument),
        }
    }

    /// Build the `Sampler` with the given number of voices.
    pub fn num_voices(mut self, n: usize) -> Self {
        self.set_num_voices(n);
        self
    }

    /// Detune the `note_on` hz by the given amount.
    pub fn detune(self, detune: f32) -> Self {
        self.map_instrument(|inst| inst.detune(detune))
    }

    /// Set the attack.
    pub fn attack<A>(self, attack: A) -> Self
        where A: Into<time::Ms>,
    {
        self.map_instrument(|inst| inst.attack(attack))
    }

    /// Set the release.
    pub fn release<R>(self, release: R) -> Self
        where R: Into<time::Ms>,
    {
        self.map_instrument(|inst| inst.release(release))
    }

    /// Set the number of voices to use for 
    pub fn set_num_voices(&mut self, n: usize) {
        self.instrument.set_num_voices(n);
        self.voices.map.resize(n, None);
    }

    /// Begin playback of a note.
    #[inline]
    pub fn note_on<T>(&mut self, note_hz: T, note_vel: Velocity)
        where M: instrument::Mode + super::Mode,
              T: Into<pitch::Hz>
    {
        let Sampler { ref mut instrument, ref mut voices, ref map, .. } = *self;
        let hz = note_hz.into();
        instrument.note_on(hz, note_vel);
        super::Mode::note_on(&mut instrument.mode, hz, note_vel, map, &mut voices.map);
    }

    /// Stop playback of the note that was triggered with the matching frequency.
    #[inline]
    pub fn note_off<T>(&mut self, note_hz: T)
        where M: instrument::Mode + super::Mode,
              T: Into<pitch::Hz>
    {
        let Sampler { ref mut instrument, ref mut voices, ref map, .. } = *self;
        let hz = note_hz.into();
        instrument.note_off(hz);
        super::Mode::note_off(&mut instrument.mode, hz, map, &mut voices.map);
    }

    /// Produces an iterator that yields `Frame`s of audio data.
    pub fn frames(&mut self, sample_hz: f64) -> Frames<F, NFG::NoteFreq>
        where F: Frame,
              F::Sample: sample::Duplex<f64>,
              <F::Sample as PcmSample>::Float: sample::FromSample<f32>,
    {
        Frames {
            voices: &mut self.voices,
            instrument_frames: self.instrument.frames(sample_hz),
        }
    }

    /// Returns whether or not the `Sampler` is currently playing any notes.
    pub fn is_active(&self) -> bool {
        for voice in &self.voices.map {
            if voice.is_some() {
                return true;
            }
        }
        false
    }

    /// Fills the given slice of frames with the `Sampler::frames` iterator.
    pub fn fill_slice(&mut self, output: &mut [F], sample_hz: f64)
        where F: Frame,
              F::Sample: sample::Duplex<f64>,
              <F::Sample as PcmSample>::Float: sample::FromSample<f32>,
    {
        let mut frames = self.frames(sample_hz);
        sample::slice::map_in_place(output, |f| {
            f.zip_map(frames.next_frame(), |a, b| {
                a.add_amp(b.to_sample::<<F::Sample as PcmSample>::Signed>())
            })
        });
    }

}



impl<F> PlayingSample<F>
    where F: Frame,
{

    /// Construct a new `PlayingSample` from the given note hz, velocity and the associated
    /// `Sample` from the `Map`.
    pub fn new(hz: pitch::Hz, vel: Velocity, sample: map::Sample<F>) -> Self {
        Self::from_playhead_idx(0, hz, vel, sample)
    }

    /// Construct a new `PlayingSample` from the given note hz, velocity and the associated
    /// `Sample` from the `Map`.
    ///
    /// The given `Sample`'s audio will begin playing from the given `idx`.
    pub fn from_playhead_idx(idx: usize,
                             hz: pitch::Hz,
                             vel: Velocity,
                             sample: map::Sample<F>) -> Self
    {
        let map::Sample { base_hz, base_vel, audio } = sample;
        let playhead = Playhead::from_idx(idx, audio);
        let rate_converter = sample::rate::Converter::scale_playback_hz(playhead, 1.0);
        PlayingSample {
            note_on_hz: hz,
            note_on_vel: vel,
            base_hz: base_hz,
            base_vel: base_vel,
            rate_converter: rate_converter,
        }
    }

}


impl<F> Playhead<F>
    where F: Frame,
{
    /// Wrap the given `Audio` with a `Playhead` starting from 0.
    pub fn new(audio: map::Audio<F>) -> Self {
        Self::from_idx(0, audio)
    }

    /// Wrap the given `Audio` with a `Playhead` starting from the given playhead index.
    pub fn from_idx(idx: usize, audio: map::Audio<F>) -> Self {
        Playhead {
            idx: idx,
            audio: audio,
        }
    }
}

impl<F> Iterator for Playhead<F>
    where F: Frame,
{
    type Item = F;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        self.idx += 1;
        self.audio.data.get(idx).map(|&f| f)
    }
}


impl<'a, F, NF> Frames<'a, F, NF>
    where F: Frame,
          F::Sample: sample::Duplex<f64>,
          <F::Sample as PcmSample>::Float: sample::FromSample<f32>,
          NF: instrument::NoteFreq,
{
    /// Yields the next audio `Frame`.
    #[inline]
    pub fn next_frame(&mut self) -> F {
        let Frames {
            ref mut voices,
            ref mut instrument_frames,
        } = *self;

        let frame_per_voice = instrument_frames.next_frame_per_voice();
        voices.map.iter_mut()
            .zip(frame_per_voice)
            .filter_map(|(v, amp_hz)| amp_hz.map(|amp_hz| (v, amp_hz)))
            .fold(F::equilibrium(), |frame, (voice, (amp, hz))| {
                match *voice {
                    None => return frame,
                    Some(ref mut voice) => {
                        let playback_hz_scale = hz / voice.base_hz.hz();
                        voice.rate_converter.set_playback_hz_scale(playback_hz_scale as f64);
                        match voice.rate_converter.next_frame() {
                            Some(wave) => {
                                let amp = amp * voice.base_vel;
                                let scaled = wave.scale_amp(amp.to_sample());
                                return frame.zip_map(scaled, |f, s| {
                                    f.add_amp(s.to_sample::<<F::Sample as PcmSample>::Signed>())
                                });
                            },
                            None => (),
                        }
                    },
                }

                // If we made it this far, the voice has finished playback of the note.
                *voice = None;
                frame
            })
    }
}

impl<'a, F, NF> Iterator for Frames<'a, F, NF>
    where F: Frame,
          F::Sample: sample::Duplex<f64>,
          <F::Sample as PcmSample>::Float: sample::FromSample<f32>,
          NF: instrument::NoteFreq,
{
    type Item = F;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_frame())
    }
}


// pub fn fill_buffer_with_voice<NF, F>(
//     voice: &mut instrument::Voice<NF>,
//     output: &mut [F],
//     sample: &map::Sample<F>,
//     sample_hz: f64,
//     n_channels: usize,
//     loop_data: Option<&(instrument::unit::LoopStart, instrument::unit::LoopEnd)>,
//     fade_data: Option<&(instrument::unit::Attack, instrument::unit::Release)>
// ) where F: sample::Frame + sample::Duplex<f32>,
//         NF: instrument::NoteFreq,
// {
//     let instrument::Voice {
//         ref mut playhead,
//         ref mut loop_playhead,
//         ref mut maybe_note,
//     } = *voice;
// 
//     let (attack, release) = fade_data.map_or_else(|| (0, 0), |&(a, r)| (a, r));
//     let velocity = maybe_note.as_ref().map_or_else(|| 1.0, |&(_, _, _, velocity)| velocity);
// 
//     // Determine the velocity by which we will multiply each sample by normalising using the
//     // `Sample`'s base velocity.
//     let velocity = velocity / sample.base_vel;
//     let base_hz = sample.base_hz.hz();
// 
//     let frame_ms = time::Samples(1).ms(sample_hz);
// 
//     for frame in output.chunks_mut(n_channels) {
// 
//         // Calculate the amplitude of the current frame.
//         let wave = if maybe_note.is_some() && *loop_playhead < duration {
// 
//             let (note_state, hz) = maybe_note.as_mut()
//                 .map(|&mut(note_state, _, ref mut freq, _)| {
//                     freq.step_frame(frame_ms);
//                     (note_state, freq.hz())
//                 }).unwrap();
// 
//             let freq_multi = hz as f64 / base_hz as f64;
// 
//             //let wave = sample.audio.data[*
// 
//         };
// 
//     }
// }

