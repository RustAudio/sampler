extern crate find_folder; // For easily finding the assets folder.
extern crate portaudio as pa; // For audio I/O
extern crate pitch_calc as pitch; // To work with musical notes.
extern crate sample; // To convert portaudio sample buffers to frames.
extern crate sampler;

use sampler::Sampler;

const CHANNELS: i32 = 2;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;
const THUMB_PIANO: &'static str = "thumbpiano A#3.wav";


fn main() {
    run().unwrap();
}

fn run() -> Result<(), pa::Error> {

    // We'll create a sample map that maps a single sample to the entire note range.
    let assets = find_folder::Search::ParentsThenKids(5, 5).for_folder("assets").unwrap();
    let sample = sampler::Sample::from_wav_file(assets.join(THUMB_PIANO), SAMPLE_RATE).unwrap();
    let sample_map = sampler::Map::from_single_sample(sample);

    // Create a polyphonic sampler.
    let mut sampler = Sampler::poly((), sample_map).num_voices(4);

    // Initialise PortAudio and create an output stream.
    let pa = try!(pa::PortAudio::new());
    let settings = 
        try!(pa.default_output_stream_settings::<i16>(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER));

    let callback = move |pa::OutputStreamCallbackArgs { buffer, .. }| {
        let buffer: &mut [[i16; CHANNELS as usize]] =
            sample::slice::to_frame_slice_mut(buffer).unwrap();
        sample::slice::equilibrium(buffer);

        // If the sampler is not currently active, play a note.
        if !sampler.is_active() {
            let vel = 0.3;
            sampler.note_on(pitch::LetterOctave(pitch::Letter::Ash, 3).to_hz(), vel);
            sampler.note_on(pitch::LetterOctave(pitch::Letter::Dsh, 3).to_hz(), vel);
            sampler.note_on(pitch::LetterOctave(pitch::Letter::Gsh, 1).to_hz(), vel);
        }

        sampler.fill_slice(buffer, SAMPLE_RATE);

        pa::Continue
    };

    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));

    try!(stream.start());

    while let Ok(true) = stream.is_active() {
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    try!(stream.stop());
    try!(stream.close());

    Ok(())
}
