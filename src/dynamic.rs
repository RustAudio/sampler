use instrument;
use sampler;

/// An alias for a `Sampler` type that uses a dynamic instrument and note frequency mode.
pub type Sampler<F> =
    sampler::Sampler<instrument::mode::Dynamic, instrument::note_freq::DynamicGenerator, F>;
