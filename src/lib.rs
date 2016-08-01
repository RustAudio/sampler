#[cfg(feature="wav")] extern crate hound;
pub extern crate instrument;
pub extern crate sample;
extern crate pitch_calc as pitch;
extern crate time_calc as time;

pub use audio::Audio;
pub use map::{Map, Sample};
pub use mode::Mode;
pub use sampler::{Frames, Sampler};

pub mod audio;
pub mod dynamic;
pub mod map;
mod mode;
mod sampler;

#[cfg(feature="serde_serialization")]
mod serde;

/// `pitch::Step` represented in discretes intervals, useful for range mapping.
pub type Step = i16;
/// The force with which a note was pressed on a keyboard.
pub type Velocity = f32;

pub const MIN_STEP: Step = 0;
pub const MAX_STEP: Step = 127;
