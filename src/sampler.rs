use instrument::{self, Instrument};
use map::{self, Map};
use pitch;
use sample::{self, Frame, Sample as PcmSample};
use std;
use time;
use Velocity;

/// A Sampler instrument.
#[derive(Clone, Debug)]
pub struct Sampler<M, NFG, A>
    where NFG: instrument::NoteFreqGenerator,
          A: map::Audio,
{
    pub instrument: Instrument<M, NFG>,
    pub map: Map<A>,
    voices: Voices<A>,
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
pub struct Voices<A>
    where A: map::Audio,
{
    map: Vec<Option<PlayingSample<A>>>,
}

/// A sample that is currently being played back.
#[derive(Clone)]
pub struct PlayingSample<A>
    where A: map::Audio,
{
    /// The pitch in hz at which the `note_on` was triggered.
    pub note_on_hz: pitch::Hz,
    pub note_on_vel: Velocity,
    base_hz: pitch::Hz,
    base_vel: Velocity,
    /// Rate-adjustable interpolation of audio.
    pub rate_converter: sample::rate::Converter<Playhead<A>>,
}

/// An owned iterator that wraps an audio file but does not 
#[derive(Clone)]
pub struct Playhead<A>
    where A: map::Audio,
{
    /// The position of the playhead over the `Sample`.
    pub idx: usize,
    audio: A,
}

/// An iterator yielding one frame from the `Sampler` at a time.
pub struct Frames<'a, A: 'a, NF: 'a>
    where A: map::Audio,
{
    voices: &'a mut Voices<A>,
    instrument_frames: instrument::Frames<'a, NF>,
}


impl<A> std::fmt::Debug for Voices<A>
    where A: map::Audio,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Voices {{ num: {:?} }}", self.map.len())
    }
}

impl<NFG, A> Sampler<instrument::mode::Mono, NFG, A>
    where NFG: instrument::NoteFreqGenerator,
          A: map::Audio,
{
    /// Construct a `Sampler` with a `Mono::Legato` playback mode.
    pub fn legato(nfg: NFG, map: Map<A>) -> Self {
        Self::new(instrument::mode::Mono::legato(), nfg, map)
    }
}

impl<NFG, A> Sampler<instrument::mode::Mono, NFG, A>
    where NFG: instrument::NoteFreqGenerator,
          A: map::Audio,
{
    /// Construct a `Sampler` with a `Mono::Retrigger` playback mode.
    pub fn retrigger(nfg: NFG, map: Map<A>) -> Self {
        Self::new(instrument::mode::Mono::retrigger(), nfg, map)
    }
}

impl<NFG, A> Sampler<instrument::mode::Poly, NFG, A>
    where NFG: instrument::NoteFreqGenerator,
          A: map::Audio,
{
    /// Construct a `Sampler` with a `Poly` playback mode.
    pub fn poly(nfg: NFG, map: Map<A>) -> Self {
        Self::new(instrument::mode::Poly, nfg, map)
    }
}


impl<M, NFG, A> Sampler<M, NFG, A>
    where NFG: instrument::NoteFreqGenerator,
          A: map::Audio,
{

    /// Construct a new `Sampler`.
    pub fn new(mode: M, note_freq_gen: NFG, map: Map<A>) -> Self {
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
    pub fn map_instrument<Map, NewM, NewNFG>(self, f: Map) -> Sampler<NewM, NewNFG, A>
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

    /// Return the number of voices for use within the `Sampler`.
    pub fn voice_count(&self) -> usize {
        self.voices.map.len()
    }

    /// Detune the `note_on` hz by the given amount.
    pub fn detune(self, detune: f32) -> Self {
        self.map_instrument(|inst| inst.detune(detune))
    }

    /// Set the attack.
    pub fn attack<Attack>(self, attack: Attack) -> Self
        where Attack: Into<time::Ms>,
    {
        self.map_instrument(|inst| inst.attack(attack))
    }

    /// Set the release.
    pub fn release<Release>(self, release: Release) -> Self
        where Release: Into<time::Ms>,
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

    /// Stop playback and clear the current notes.
    #[inline]
    pub fn stop(&mut self)
        where M: instrument::Mode,
    {
        let Sampler { ref mut instrument, ref mut voices, .. } = *self;
        instrument.stop();
        voices.map.clear();
    }

    /// Produces an iterator that yields `Frame`s of audio data.
    pub fn frames(&mut self, sample_hz: f64) -> Frames<A, NFG::NoteFreq>
        where A: map::Audio,
              <A::Frame as Frame>::Sample: sample::Duplex<f64>,
              <<A::Frame as Frame>::Sample as PcmSample>::Float: sample::FromSample<f32>,
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
    pub fn fill_slice<F>(&mut self, output: &mut [F], sample_hz: f64)
        where F: Frame,
              F::Sample: sample::Duplex<f64>,
              <F::Sample as PcmSample>::Float: sample::FromSample<f32>,
              A: map::Audio<Frame=F>,
    {
        let mut frames = self.frames(sample_hz);
        sample::slice::map_in_place(output, |f| {
            f.zip_map(frames.next_frame(), |a, b| {
                a.add_amp(b.to_sample::<<F::Sample as PcmSample>::Signed>())
            })
        });
    }

}


#[cfg(feature="serde_serialization")]
pub mod private {
    use instrument::{self, Instrument};
    use map::{self, Map};

    /// A private constructor for use within serde.rs.
    pub fn new<M, NFG, A>(instrument: Instrument<M, NFG>,
                          map: Map<A>,
                          num_voices: usize) -> super::Sampler<M, NFG, A>
        where NFG: instrument::NoteFreqGenerator,
              A: map::Audio,
    {
        super::Sampler {
            instrument: instrument,
            map: map,
            voices: super::Voices { map: vec![None; num_voices] },
        }
    }
}


impl<A> PlayingSample<A>
    where A: map::Audio,
{

    /// Construct a new `PlayingSample` from the given note hz, velocity and the associated
    /// `Sample` from the `Map`.
    pub fn new(hz: pitch::Hz, vel: Velocity, sample: map::Sample<A>) -> Self {
        Self::from_playhead_idx(0, hz, vel, sample)
    }

    /// Construct a new `PlayingSample` from the given note hz, velocity and the associated
    /// `Sample` from the `Map`.
    ///
    /// The given `Sample`'s audio will begin playing from the given `idx`.
    pub fn from_playhead_idx(idx: usize,
                             hz: pitch::Hz,
                             vel: Velocity,
                             sample: map::Sample<A>) -> Self
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


impl<A> Playhead<A>
    where A: map::Audio,
{
    /// Wrap the given `Audio` with a `Playhead` starting from 0.
    pub fn new(audio: A) -> Self {
        Self::from_idx(0, audio)
    }

    /// Wrap the given `Audio` with a `Playhead` starting from the given playhead index.
    pub fn from_idx(idx: usize, audio: A) -> Self {
        Playhead {
            idx: idx,
            audio: audio,
        }
    }
}

impl<A> Iterator for Playhead<A>
    where A: map::Audio,
{
    type Item = A::Frame;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.idx;
        self.idx += 1;
        map::Audio::data(&self.audio).get(idx).map(|&f| f)
    }
}


impl<'a, A, NF> Frames<'a, A, NF>
    where A: map::Audio,
          <A::Frame as Frame>::Sample: sample::Duplex<f64>,
          <<A::Frame as Frame>::Sample as PcmSample>::Float: sample::FromSample<f32>,
          NF: instrument::NoteFreq,
{
    /// Yields the next audio `Frame`.
    #[inline]
    pub fn next_frame(&mut self) -> A::Frame {
        let Frames {
            ref mut voices,
            ref mut instrument_frames,
        } = *self;

        let frame_per_voice = instrument_frames.next_frame_per_voice();
        voices.map.iter_mut()
            .zip(frame_per_voice)
            .filter_map(|(v, amp_hz)| amp_hz.map(|amp_hz| (v, amp_hz)))
            .fold(<A::Frame as Frame>::equilibrium(), |frame, (voice, (amp, hz))| {
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
                                    f.add_amp(s.to_sample::<<<A::Frame as Frame>::Sample as PcmSample>::Signed>())
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

impl<'a, A, NF> Iterator for Frames<'a, A, NF>
    where A: map::Audio,
          <A::Frame as Frame>::Sample: sample::Duplex<f64>,
          <<A::Frame as Frame>::Sample as PcmSample>::Float: sample::FromSample<f32>,
          NF: instrument::NoteFreq,
{
    type Item = A::Frame;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_frame())
    }
}
