use {Step, Velocity, MIN_STEP, MAX_STEP};
use audio::Audio;
use pitch;


/// A type that maps frequncy and velocity ranges to audio samples.
#[derive(Clone, Debug, PartialEq)]
pub struct Map<A> {
    pub pairs: Vec<SampleOverRange<A>>,
}

/// A performable `Sample` with some base playback Hz and Velocity.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample<A> {
    pub base_hz: pitch::Hz,
    pub base_vel: Velocity,
    pub audio: A,
}

/// A 2-dimensional space, represented as a frequency range and a velocity range.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct StepVelRange {
    pub step: Range<Step>,
    pub vel: Range<Velocity>,
}

/// A range paired with a specific sample.
#[derive(Clone, Debug, PartialEq)]
pub struct SampleOverRange<A> {
    pub range: StepVelRange,
    pub sample: Sample<A>,
}

/// A continuous range of `T` from the `min` to the `max`.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
}


impl Range<Step> {
    /// Is the given step greater than or equal to the `min` and smaller than the `max`.
    pub fn is_over(&self, step: Step) -> bool {
        self.min <= step && step <= self.max
    }
}

impl Range<Velocity> {
    /// Is the given velocity greater than or equal to the `min` and smaller than the `max`.
    pub fn is_over(&self, vel: Velocity) -> bool {
        self.min <= vel && vel <= self.max
    }
}

impl<A> Sample<A> {

    /// Constructor for a new `Sample` with the given base Hz and Velocity.
    pub fn new(base_hz: pitch::Hz, base_vel: Velocity, audio: A) -> Self {
        Sample {
            base_hz: base_hz,
            base_vel: base_vel,
            audio: audio,
        }
    }

    /// Maps the `Sample` with some `Audio` type `A` to a `Sample` with some `Audio` type `B`.
    pub fn map_audio<F, B>(self, map: F) -> Sample<B>
        where F: FnOnce(A) -> B,
    {
        let Sample { base_hz, base_vel, audio } = self;
        Sample {
            base_hz: base_hz,
            base_vel: base_vel,
            audio: map(audio),
        }
    }

}

impl<A> Map<A>
    where A: Audio,
{

    /// Construct an empty `Map`.
    pub fn empty() -> Self {
        Map { pairs: vec![] }
    }

    /// Construct a `Map` from a series of mappings, starting from (-C2, 1.0).
    pub fn from_sequential_steps<I>(mappings: I) -> Self
        where I: IntoIterator<Item=(Step, Velocity, Sample<A>)>,
    {
        let (mut last_step, mut last_vel) = (0, 1.0);
        let pairs = mappings.into_iter().map(|(step, vel, sample)| {
            let range = StepVelRange {
                step: Range { min: last_step, max: step },
                vel: Range { min: last_vel, max: vel },
            };
            last_step = step;
            last_vel = vel;
            SampleOverRange { range: range, sample: sample }
        }).collect();
        Map { pairs: pairs }
    }

    /// Creates a `Map` with a single sample mapped to the entire Step and Velocity range.
    pub fn from_single_sample(sample: Sample<A>) -> Self {
        let range = StepVelRange {
            step: Range { min: MIN_STEP, max: MAX_STEP },
            vel: Range { min: 0.0, max: 1.0 },
        };
        let pairs = vec![SampleOverRange { range: range, sample: sample }];
        Map { pairs: pairs }
    }

    /// Inserts a range -> audio mapping into the Map.
    pub fn insert(&mut self, range: StepVelRange, sample: Sample<A>) {
        for i in 0..self.pairs.len() {
            if self.pairs[i].range > range {
                self.pairs.insert(i, SampleOverRange { range: range, sample: sample });
                return;
            }
        }
        self.pairs.push(SampleOverRange { range: range, sample: sample });
    }

    /// Returns the `Audio` associated with the range within which the given hz and velocity exist.
    ///
    /// TODO: This would probably be quicker with some sort of specialised RangeMap.
    pub fn sample(&self, hz: pitch::Hz, vel: Velocity) -> Option<Sample<A>> {
        let step = hz.step().round() as Step;
        for &SampleOverRange { ref range, ref sample } in &self.pairs {
            if range.step.is_over(step) && range.vel.is_over(vel) {
                return Some(sample.clone());
            }
        }
        None
    }

}


#[cfg(feature="wav")]
pub mod wav {
    use audio;
    use map;
    use pitch;
    use sample;
    use std;


    /// An alias for the `wav` `Sample` type.
    pub type Sample<F> = super::Sample<std::sync::Arc<audio::wav::Audio<F>>>;


    impl<F> Sample<F>
        where F: sample::Frame,
              F::Sample: sample::Duplex<f64> + sample::Duplex<i32>,
              Box<[F::Sample]>: sample::ToBoxedFrameSlice<F>,
    {

        /// Loads a `Sample` from the `.wav` file at the given `path`.
        ///
        /// If the `.wav` file has a musical note in the file name, that note's playback frequency in
        /// `hz` will be used as the `base_hz`.
        ///
        /// If a musical note cannot be determined automatically, a default `C1` will be used.
        ///
        /// The PCM data retrieved from the file will be re-sampled upon loading (rather than at
        /// playback) to the given target sample rate for efficiency.
        pub fn from_wav_file<P>(path: P, target_sample_hz: f64) -> Result<Self, audio::wav::Error>
            where P: AsRef<std::path::Path>,
        {
            let path = path.as_ref();

            const DEFAULT_LETTER_OCTAVE: pitch::LetterOctave = pitch::LetterOctave(pitch::Letter::C, 1);
            let base_letter_octave = read_base_letter_octave(path).unwrap_or(DEFAULT_LETTER_OCTAVE);
            let base_hz = base_letter_octave.to_hz();
            let base_vel = 1.0;

            let audio = std::sync::Arc::new(try!(audio::wav::Audio::from_file(path, target_sample_hz)));

            Ok(map::Sample::new(base_hz, base_vel, audio))
        }
    }


    /// Scans the given path for an indication of its pitch.
    fn read_base_letter_octave(path: &std::path::Path) -> Option<pitch::LetterOctave> {
        use pitch::Letter::*;
        use std::ascii::AsciiExt;

        let s = path.to_str().map_or("".into(), |s| s.to_ascii_lowercase());

        // Check to see if the path contains a note for the given `letter` for any octave
        // between -8 and 24. If so, return the `LetterOctave`.
        let contains_letter = |letter: &str| -> Option<pitch::LetterOctave> {
            for i in -8i8..24 {
                let pattern = format!("{}{}", letter, i);
                if s.contains(&pattern) {
                    let letter = match letter {
                        "c" => C,
                        "c#" | "csh" => Csh,
                        "d" => D,
                        "d#" | "dsh" => Dsh,
                        "e" => E,
                        "f" => F,
                        "f#" | "fsh" => Fsh,
                        "g" => G,
                        "g#" | "gsh" => Gsh,
                        "a" => A,
                        "a#" | "ash" => Ash,
                        "b" => B,
                        _ => unreachable!(),
                    };
                    return Some(pitch::LetterOctave(letter, i as pitch::Octave));
                }
            }
            None
        };

        let list = [
            "c", "c#", "csh", "d", "d#", "dsh", "e", "f", "f#", "fsh", "g", "g#", "gsh",
            "a", "a#", "ash", "b",
        ];

        for letter in &list[..] {
            if let Some(letter_octave) = contains_letter(letter) {
                return Some(letter_octave);
            }
        }

        None
    }

}
