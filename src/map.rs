use pitch;
use std;
use Velocity;

#[derive(Clone, Debug, PartialEq)]
pub struct SampleMap<S> {
    pairs: Vec<Pair<S>>,
}

/// Some slice of PCM samples that represents a single audio sample.
#[derive(Clone, Debug, PartialEq)]
pub struct Audio<S> {
    data: std::sync::Arc<[S]>,
}

/// A range paired with a specific sample.
#[derive(Clone, Debug, PartialEq)]
struct Pair<S> {
    range: HzVelRange,
    audio: Audio<S>,
}

/// A 2-dimensional space, represented as a frequency range and a velocity range.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct HzVelRange {
    hz: Range<pitch::Hz>,
    vel: Range<Velocity>,
}

/// A continuous range of `T` from the `min` to the `max`.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Range<T> {
    min: T,
    max: T,
}

impl<T> Range<T> {
    /// Is the given `T` greater than or equal to the `min` and smaller than the `max`.
    pub fn is_over(&self, t: T) -> bool
        where T: PartialOrd,
    {
        self.min <= t && t < self.max
    }
}

impl<S> SampleMap<S> {

    /// Construct a `SampleMap` from a series of mappings, starting from (-C2, 1.0).
    pub fn from_sequential_mappings<I>(mappings: I) -> Self
        where I: IntoIterator<Item=(pitch::Hz, Velocity, Audio<S>)>,
    {
        let (mut last_hz, mut last_vel) = (pitch::Step(0.0).to_hz(), 1.0);
        let pairs = mappings.into_iter().map(|(hz, vel, audio)| {
            let range = HzVelRange {
                hz: Range { min: last_hz, max: hz },
                vel: Range { min: last_vel, max: vel },
            };
            last_hz = hz;
            last_vel = vel;
            Pair { range: range, audio: audio }
        }).collect();
        SampleMap { pairs: pairs }
    }

    /// Inserts a range -> audio mapping into the SampleMap.
    pub fn insert(&mut self, range: HzVelRange, audio: Audio<S>) {
        for i in 0..self.pairs.len() {
            if self.pairs[i].range > range {
                self.pairs.insert(i, Pair { range: range, audio: audio });
                return;
            }
        }
        self.pairs.push(Pair { range: range, audio: audio });
    }

}


// let sampler = sample_map!{
//     wav "amen_brother":
//         step { min: C 1, base: C 1, max: C 8 }
//         vel { min: 1.0, base: 1.0, max: 1.0 }
// };
