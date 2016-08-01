use sample;
use std;


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

/// A wrapper around `sampler::map::Audio` types that slices a specific range of frames.
#[derive(Clone, Debug, PartialEq)]
pub struct Range<A> {
    /// The start index of the range.
    pub start: usize,
    /// The end index of the range.
    pub end: usize,
    /// Some audio type that implements `Audio` and can yield a slice of frames.
    pub audio: A,
}


impl<A> Range<A> {
    /// Construct a new `Range` with a max playback range.
    pub fn new(audio: A) -> Self
        where A: Audio,
    {
        Range {
            start: 0,
            end: audio.data().len(),
            audio: audio,
        }
    }
}

impl<A> Audio for std::sync::Arc<A>
    where A: Audio,
{
    type Frame = A::Frame;
    #[inline]
    fn data(&self) -> &[Self::Frame] {
        A::data(self)
    }
}

impl<A> Audio for Range<A>
    where A: Audio,
{
    type Frame = A::Frame;
    #[inline]
    fn data(&self) -> &[Self::Frame] {
        let slice = self.audio.data();
        let len = slice.len();
        if self.start < len && self.end <= len {
            &slice[self.start..self.end]
        } else {
            &[]
        }
    }
}


#[cfg(feature="wav")]
pub mod wav {
    use hound;
    use sample;
    use std;


    /// WAV data loaded into memory as a single contiguous slice of PCM frames.
    #[derive(Clone, Debug, PartialEq)]
    pub struct Audio<F> {
        pub path: std::path::PathBuf,
        pub data: Box<[F]>,
        pub sample_hz: f64,
    }

    /// Errors that may occur during `WAV` loading
    #[derive(Debug)]
    pub enum Error {
        /// Some error returned from the `hound` crate during sample loading.
        Hound(hound::Error),
        /// The bit depth of the given WAV file is unsupported.
        UnsupportedBitsPerSample(u16),
        /// There is no obvious way to map the given channels described in the WAV spec to the
        /// number of channels in the `Frame` type.
        ///
        /// Contains the source number of channels and the target number of channels.
        UnsupportedChannelMapping(u16, u16),
    }


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
              F::Sample: sample::Duplex<f64> + sample::Duplex<i32>,
              Box<[F::Sample]>: sample::ToBoxedFrameSlice<F>,
    {

        /// Loads a `Sample` from the `.wav` file at the given `path`.
        ///
        /// The PCM data retrieved from the file will be:
        /// - re-sized from its source bit rate to that of the target and
        /// - re-sampled upon loading (rather than at playback) to the given target sample rate for
        /// efficiency.
        pub fn from_file<P>(path: P, target_sample_hz: f64) -> Result<Self, Error>
            where P: AsRef<std::path::Path>,
        {
            use sample::{Frame, Sample, Signal};

            let path = path.as_ref();
            let mut wav_reader = try!(hound::WavReader::open(path));

            let spec = wav_reader.spec();

            // Collect the samples in a loop so that we may handle any errors if necessary.
            let mut samples: Vec<F::Sample> = Vec::new();

            // Reads the samples to their correct format type, and then converts them to the target
            // `F::Sample` type.
            type WavReader = hound::WavReader<std::io::BufReader<std::fs::File>>;
            fn fill_samples<S, H>(samples: &mut Vec<S>, mut wav_reader: WavReader)
                    -> Result<(), hound::Error>
                where S: sample::FromSample<i32>,
                      H: sample::Sample + sample::ToSample<i32> + hound::Sample,
            {
                for sample in wav_reader.samples() {
                    let read_sample: H = try!(sample);
                    let i32_sample: i32 = sample::Sample::to_sample(read_sample);
                    samples.push(sample::Sample::to_sample(i32_sample));
                }
                Ok(())
            }

            match spec.sample_format {
                hound::SampleFormat::Float => match spec.bits_per_sample {
                    32 => try!(fill_samples::<_, f32>(&mut samples, wav_reader)),
                    n => return Err(Error::UnsupportedBitsPerSample(n)),
                },
                hound::SampleFormat::Int => match spec.bits_per_sample {
                    8 => try!(fill_samples::<_, i8>(&mut samples, wav_reader)),
                    16 => try!(fill_samples::<_, i16>(&mut samples, wav_reader)),
                    32 => try!(fill_samples::<_, i32>(&mut samples, wav_reader)),
                    // The sample crate uses a specific type for 24 bit samples, so this 24-bit sample
                    // conversion requires this special case.
                    24 => {
                        for sample in wav_reader.samples() {
                            let read_sample: i32 = try!(sample);
                            let i24_sample = try!(sample::I24::new(read_sample)
                                .ok_or(hound::Error::FormatError("Incorrectly formatted 24-bit sample \
                                                            received from hound::read::WavSamples")));
                            let i32_sample: i32 = sample::Sample::to_sample(i24_sample);
                            samples.push(sample::Sample::to_sample(i32_sample));
                        }
                    },
                    n => return Err(Error::UnsupportedBitsPerSample(n)),
                },
            }

            let boxed_samples = samples.into_boxed_slice();
            let boxed_frames: Box<[F]> = match (spec.channels, F::n_channels() as u16) {

                // In the case that the `spec` has a different number of channels to the actual
                // slice, just collect as many valid frames as we can and discard the final
                // mismatching frame.
                (source, target) if source == target => {
                    let samples = boxed_samples.iter().cloned();
                    let vec: Vec<F> = sample::signal::from_interleaved_samples(samples)
                        .collect();
                    vec.into_boxed_slice()
                },

                // Sum the left and right channels together when mapping to a mono signal.
                (2, 1) => {
                    let samples = boxed_samples.iter().cloned();
                    let vec: Vec<F> = 
                        sample::signal::from_interleaved_samples::<_, [F::Sample; 2]>(samples)
                            .filter_map(|f| {
                                let mut channels = f.channels();
                                channels.next()
                                    .and_then(|l| channels.next().map(|r| (l, r)))
                                    .map(|(l, r)| {
                                        let sum = l.add_amp(r.to_signed_sample());
                                        F::from_fn(|_| sum)
                                    })
                            })
                            .collect();
                    vec.into_boxed_slice()
                },

                // Simply copy the single mono channel to both channels in the output stereo
                // signal.
                (1, 2) => {
                    let samples = boxed_samples.iter().cloned();
                    let vec: Vec<F> = samples.map(|s| F::from_fn(|_| s)).collect();
                    vec.into_boxed_slice()
                },

                (source, target) => {
                    return Err(Error::UnsupportedChannelMapping(source, target))
                },
                
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

    impl From<hound::Error> for Error {
        fn from(err: hound::Error) -> Self {
            Error::Hound(err)
        }
    }

    impl std::error::Error for Error {
        fn description(&self) -> &str {
            match *self {
                Error::Hound(ref hound) => std::error::Error::description(hound),
                Error::UnsupportedBitsPerSample(_n_bits) => "unsupported bits per sample",
                Error::UnsupportedChannelMapping(_source, _target) => "unsupported channel mapping",
            }
        }
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            std::fmt::Debug::fmt(self, f)
        }
    }

}
