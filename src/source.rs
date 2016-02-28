
/// The source of samples, which may be either dynamic or static.
pub trait Source {}

pub struct Dynamic;

pub struct Static;


pub trait PcmSampleSource {
    fn samples<I, S>(&self) -> I where I: Iterator<Item=S>;
}
