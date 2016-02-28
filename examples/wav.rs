extern crate find_folder;
extern crate hound;
extern crate portaudio as pa;

fn main() {
    run().unwrap();
}

fn run() -> Result<(), pa::Error> {

    let mut samples = {
        let assets = find_folder::Search::ParentsThenKids(5, 5).for_folder("assets").unwrap();
        let sample_file = assets.join("amen_brother.wav");
        let mut reader = hound::WavReader::open(&sample_file).unwrap();
        reader.samples().collect::<Result<Vec<i16>, _>>().unwrap().into_iter()
    };

    let pa = try!(pa::PortAudio::new());

    const CHANNELS: i32 = 1;
    const SAMPLE_RATE: f64 = 44_100.0;
    const FRAMES_PER_BUFFER: u32 = 64;

    let settings = try!(pa.default_output_stream_settings(CHANNELS, SAMPLE_RATE, FRAMES_PER_BUFFER));
    let callback = move |pa::OutputStreamCallbackArgs { buffer, .. }| {
        for sample in buffer.iter_mut() {
            *sample = match samples.next() {
                Some(s) => s,
                None => return pa::Complete,
            };
        }
        pa::Continue
    };

    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));

    try!(stream.start());

    while let Ok(true) = stream.is_active() {}

    try!(stream.stop());
    try!(stream.close());

    Ok(())
}
