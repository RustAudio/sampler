use pitch;
use sample;
use std;
use Velocity;


/// A type that maps frequncy and velocity ranges to audio samples.
#[derive(Clone, Debug, PartialEq)]
pub struct Map<A> {
    pub pairs: Vec<SampleOverRange<A>>,
}

// /// Some slice of PCM samples that represents a single audio sample.
// ///
// /// **Note:** The `sampler` crate currently assumes that the `Audio` you give it has the same
// /// format as the parameters with which audio is requested. We are hoping to enforce this using
// /// types with some changes to the `sample` crate.
// #[derive(Clone, Debug, PartialEq)]
// pub struct Audio<F> {
//     pub data: std::sync::Arc<Box<[F]>>,
// }

/// The audio data that provides the slice of frames that are to be rendered.
///
/// By making this a trait instead of a hard type, we can allow users to use their own `Audio`
/// types which might require other data (i.e. file paths, names, etc) for unique serialization
/// implementations.
pub trait Audio: Clone {
    /// The type of `Frame` data associated with the audio.
    type Frame: sample::Frame;
    /// A reference to the slice of frames used to play the audio.
    fn data(&self) -> &[Self::Frame];
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
pub struct HzVelRange {
    pub hz: Range<pitch::Hz>,
    pub vel: Range<Velocity>,
}

/// A range paired with a specific sample.
#[derive(Clone, Debug, PartialEq)]
pub struct SampleOverRange<A> {
    pub range: HzVelRange,
    pub sample: Sample<A>,
}

/// A continuous range of `T` from the `min` to the `max`.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
}


impl<T> Audio for std::sync::Arc<T>
    where T: Audio,
{
    type Frame = T::Frame;
    fn data(&self) -> &[Self::Frame] {
        T::data(self)
    }
}

impl Range<pitch::Hz> {
    /// Is the given hz greater than or equal to the `min` and smaller than the `max`.
    pub fn is_over(&self, hz: pitch::Hz) -> bool {
        self.min <= hz && hz < self.max
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

}

impl<A> Map<A>
    where A: Audio,
{

    /// Construct an empty `Map`.
    pub fn empty() -> Self {
        Map { pairs: vec![] }
    }

    /// Construct a `Map` from a series of mappings, starting from (-C2, 1.0).
    pub fn from_sequential_mappings<I>(mappings: I) -> Self
        where I: IntoIterator<Item=(pitch::Hz, Velocity, Sample<A>)>,
    {
        const MIN_HZ: pitch::Hz = pitch::Hz(0.0);
        let (mut last_hz, mut last_vel) = (MIN_HZ, 1.0);
        let pairs = mappings.into_iter().map(|(hz, vel, sample)| {
            let range = HzVelRange {
                hz: Range { min: last_hz, max: hz },
                vel: Range { min: last_vel, max: vel },
            };
            last_hz = hz;
            last_vel = vel;
            SampleOverRange { range: range, sample: sample }
        }).collect();
        Map { pairs: pairs }
    }

    /// Creates a `Map` with a single sample mapped to the entire Hz and Velocity range.
    pub fn from_single_sample(sample: Sample<A>) -> Self {
        let range = HzVelRange {
            hz: Range { min: pitch::Hz(0.0), max: pitch::Hz(std::f32::MAX) },
            vel: Range { min: 0.0, max: 1.0 },
        };
        let pairs = vec![SampleOverRange { range: range, sample: sample }];
        Map { pairs: pairs }
    }

    /// Inserts a range -> audio mapping into the Map.
    pub fn insert(&mut self, range: HzVelRange, sample: Sample<A>) {
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
        for &SampleOverRange { ref range, ref sample } in &self.pairs {
            if range.hz.is_over(hz) && range.vel.is_over(vel) {
                return Some(sample.clone());
            }
        }
        None
    }

}


#[cfg(feature="wav")]
pub mod wav {
    use hound;
    use map;
    use pitch;
    use sample;
    use std;

    /// WAV data loaded into memory as a single contiguous slice of PCM frames.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Audio<F> {
        pub path: std::path::PathBuf,
        pub data: Box<[F]>,
        pub sample_hz: f64,
    }

    /// An alias for the `wav` `Sample` type.
    pub type Sample<F> = super::Sample<std::sync::Arc<Audio<F>>>;

    impl<F> super::Audio for Audio<F>
        where F: sample::Frame,
    {
        type Frame = F;
        fn data(&self) -> &[Self::Frame] {
            &self.data[..]
        }
    }

    impl<F> Audio<F>
        where F: sample::Frame,
              F::Sample: sample::Duplex<f64> + hound::Sample,
              Box<[F::Sample]>: sample::ToBoxedFrameSlice<F>,
    {

        /// Loads a `Sample` from the `.wav` file at the given `path`.
        ///
        /// The PCM data retrieved from the file will be re-sampled upon loading (rather than at
        /// playback) to the given target sample rate for efficiency.
        pub fn from_file<P>(path: P, target_sample_hz: f64) -> Result<Self, hound::Error>
            where P: AsRef<std::path::Path>,
        {
            use sample::Signal;

            let path = path.as_ref();
            let mut wav_reader = try!(hound::WavReader::open(path));

            let spec = wav_reader.spec();
            // TODO: Return an error instead of panic!ing! OR do some sort of frame /
            // channel layout conversion.
            assert!(spec.channels as usize == F::n_channels(),
                    "The number of channels in the audio file differs from the number of \
                    channels in the frame");

            // Collect the samples in a loop so that we may handle any errors if necessary.
            let mut samples: Vec<F::Sample> = Vec::new();
            for sample in wav_reader.samples() {
                samples.push(try!(sample));
            }

            let boxed_samples = samples.into_boxed_slice();
            let boxed_frames: Box<[F]> = match sample::slice::to_boxed_frame_slice(boxed_samples) {
                Some(slice) => slice,
                // TODO: Return an error instead of panic!ing! OR do some sort of frame /
                // channel layout conversion.
                None => panic!("The number of samples produced from the wav file does not \
                               match the number of channels ({}) in the given `Frame` type",
                               F::n_channels()),
            };

            // Convert the sample rate to our target sample rate.
            let frames: Vec<F> = boxed_frames.iter().cloned()
                .from_hz_to_hz(spec.sample_rate as f64, target_sample_hz)
                .collect();

            Ok(Audio {
                path: path.to_path_buf(),
                sample_hz: target_sample_hz,
                data: frames.into_boxed_slice(),
            })
        }

    }

    impl<F> Sample<F>
        where F: sample::Frame,
              F::Sample: sample::Duplex<f64> + hound::Sample,
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
        pub fn from_wav_file<P>(path: P, target_sample_hz: f64) -> Result<Self, hound::Error>
            where P: AsRef<std::path::Path>,
        {
            let path = path.as_ref();

            const DEFAULT_LETTER_OCTAVE: pitch::LetterOctave = pitch::LetterOctave(pitch::Letter::C, 1);
            let base_letter_octave = read_base_letter_octave(path).unwrap_or(DEFAULT_LETTER_OCTAVE);
            let base_hz = base_letter_octave.to_hz();
            let base_vel = 1.0;

            let audio = std::sync::Arc::new(try!(Audio::from_file(path, target_sample_hz)));

            Ok(map::Sample::new(base_hz, base_vel, audio))
        }
    }

    // impl<F> map::Map<F>
    //     where F: sample::Frame,
    //           F::Sample: sample::Duplex<f64> + hound::Sample,
    //           Box<[F::Sample]>: sample::ToBoxedFrameSlice<F>,
    // {

    //     /// Loads a `Map` from the given directory.
    //     ///
    //     /// All `.wav` files that can be successfully loaded will be loaded into the `Map`.
    //     ///
    //     /// If the `.wav` file has a musical note in the file name, that note's playback frequency in
    //     /// `hz` will be used as the `base_hz`.
    //     ///
    //     /// For efficiency, all files will be re-sampled upon loading (rather than at playback) to the
    //     /// given target sample rate.
    //     pub fn from_wav_directory<P>(path: P, target_sample_hz: f64) -> Result<Self, hound::Error>
    //         where P: AsRef<std::path::Path>,
    //     {
    //         use sample::Signal;
    //         use std::cmp::Ordering;

    //         let path = path.as_ref();

    //         struct SampleReader {
    //             base_letter_octave: Option<pitch::LetterOctave>,
    //             wav_reader: hound::WavReader<std::io::BufReader<std::fs::File>>,
    //         }

    //         let mut sample_readers: Vec<SampleReader> = Vec::new();

    //         // Find all .wav files in the given directory and store them as `SampleReader`s.
    //         for entry in try!(std::fs::read_dir(path)) {
    //             let file_name = try!(entry).file_name();

    //             // If the entry is a wave file, add it to our list.
    //             if has_wav_ext(file_name.as_ref()) {
    //                 let wav_reader = try!(hound::WavReader::open(&file_name));
    //                 let sample_reader = SampleReader {
    //                     base_letter_octave: read_base_letter_octave(file_name.as_ref()),
    //                     wav_reader: wav_reader,
    //                 };
    //                 sample_readers.push(sample_reader);
    //             }
    //         }

    //         // Sort the readers by their base hz.
    //         sample_readers.sort_by(|a, b| match (a.base_letter_octave, b.base_letter_octave) {
    //             (Some(_), None) => Ordering::Less,
    //             (None, Some(_)) => Ordering::Greater,
    //             (Some(a), Some(b)) => a.cmp(&b),
    //             (None, None) => Ordering::Equal,
    //         });

    //         const DEFAULT_LETTER_OCTAVE: pitch::LetterOctave =
    //             pitch::LetterOctave(pitch::Letter::C, 1);
    //         let mut maybe_last_step = None;

    //         // We must imperatively collect the mappings so that we can handle any errors.
    //         let mut mappings = Vec::with_capacity(sample_readers.len());
    //         for SampleReader { base_letter_octave, mut wav_reader } in sample_readers {
    //             let base_vel = 1.0;
    //             let base_hz = match base_letter_octave {
    //                 Some(letter_octave) => {
    //                     maybe_last_step = Some(letter_octave.step());
    //                     letter_octave.to_hz()
    //                 },
    //                 None => {
    //                     let last_step = maybe_last_step.unwrap_or(DEFAULT_LETTER_OCTAVE.step());
    //                     let step = last_step + 1.0;
    //                     maybe_last_step = Some(step);
    //                     pitch::Step(step).to_hz()
    //                 },
    //             };

    //             let audio = {
    //                 let spec = wav_reader.spec();

    //                 // Collect the samples in a loop so that we may handle any errors if necessary.
    //                 let mut samples: Vec<F::Sample> = Vec::new();
    //                 for sample in wav_reader.samples() {
    //                     samples.push(try!(sample));
    //                 }

    //                 let boxed_samples = samples.into_boxed_slice();
    //                 let boxed_frames: Box<[F]> = match sample::slice::to_boxed_frame_slice(boxed_samples) {
    //                     Some(slice) => slice,
    //                     // TODO: Return an error instead of panic!ing! OR do some sort of frame /
    //                     // channel layout conversion.
    //                     None => panic!("The number of samples produced from the wav file does not \
    //                                    match the number of channels ({}) in the given `Frame` type",
    //                                    F::n_channels()),
    //                 };

    //                 // Convert the sample rate to our target sample rate.
    //                 let frames: Vec<F> = boxed_frames.iter().cloned()
    //                     .from_hz_to_hz(spec.sample_rate as f64, target_sample_hz)
    //                     .collect();

    //                 map::Audio::new(frames.into_boxed_slice())
    //             };

    //             let sample = map::Sample::new(base_hz, base_vel, audio);

    //             // The `Hz` range that triggers this sample will span from the last sample's Hz (or
    //             // the minimum if there is no last sample) to the following `to_hz` value.
    //             //
    //             // TODO: Investigate a nicer way of evenly spreading samples across the keyboard.
    //             let to_hz = pitch::Step(base_hz.step() + 0.5).to_hz();
    //             let to_vel = base_vel;
    //             mappings.push((to_hz, to_vel, sample));
    //         }

    //         Ok(Self::from_sequential_mappings(mappings))
    //     }

    // }


    ///// Utility functions.
    
    
    /// Determines whether the given `Path` leads to a wave file.
    fn has_wav_ext(path: &std::path::Path) -> bool {
        let ext = path.extension()
            .and_then(|s| s.to_str())
            .map_or("".into(), std::ascii::AsciiExt::to_ascii_lowercase);
        match &ext[..] {
            "wav" | "wave" => true,
            _ => false,
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
