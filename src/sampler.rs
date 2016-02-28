use map::SampleMap;
use pitch;
use sample::Sample as PcmSample;
use Velocity;
use voice::Voice;

#[derive(Clone, Debug, PartialEq)]
pub struct Sampler<S> {
    map: SampleMap<S>,
    voices: Vec<Voice>,
}

impl<S> Sampler<S> {

    /// Construct a new `Sampler`.
    pub fn new(map: SampleMap<S>) -> Self {
        Sampler {
            map: map,
            voices: vec![],
        }
    }

    /// Begin playback of a note.
    ///
    /// `Sampler` will try to use a free `Voice` to do this. If no `Voice`s are free, the one
    /// playing the oldest note will be chosen to play the new note instead.
    #[inline]
    pub fn note_on<T>(&mut self, note_hz: T, note_vel: Velocity)
        where T: Into<pitch::Hz>
    {
    }

    /// Stop playback of the note that was triggered with the matching frequency.
    #[inline]
    pub fn note_off<T>(&mut self, note_hz: T)
        where T: Into<pitch::Hz>
    {
    }

}
