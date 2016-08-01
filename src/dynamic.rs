use audio;
use instrument;
use map;
use sampler;

/// An alias for a dynamic `Mode` type.
pub type Mode = instrument::mode::Dynamic;

/// An alias for a dynamic `NoteFreqGenerator` type.
pub type NoteFreqGenerator = instrument::note_freq::DynamicGenerator;

/// An alias for a `Sampler` type that uses a dynamic instrument and note frequency mode.
pub type Sampler<A> = sampler::Sampler<Mode, NoteFreqGenerator, A>;

impl<A> Sampler<A>
    where A: audio::Audio,
{

    /// Construct a dynamic `Sampler`.
    pub fn dynamic(mode: instrument::mode::Dynamic, map: map::Map<A>) -> Self {
        let nfg = instrument::note_freq::DynamicGenerator::Constant;
        Self::new(mode, nfg, map)
    }

    /// Construct a dynamic `Sampler` initialised with a `Mono::Legato` playback mode.
    pub fn dynamic_legato(map: map::Map<A>) -> Self {
        Self::dynamic(instrument::mode::Dynamic::legato(), map)
    }

    /// Construct a dynamic `Sampler` initialised with a `Mono::Retrigger` playback mode.
    pub fn dynamic_retrigger(map: map::Map<A>) -> Self {
        Self::dynamic(instrument::mode::Dynamic::retrigger(), map)
    }

    /// Construct a dynamic `Sampler` initialised with a `Poly` playback mode.
    pub fn dynamic_poly(map: map::Map<A>) -> Self {
        Self::dynamic(instrument::mode::Dynamic::poly(), map)
    }

}
