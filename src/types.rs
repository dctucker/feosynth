
pub type Frequency = f64;
pub type Cents = f64;
pub type Sample = f64;
pub type SampleRate = u32;
pub type Seconds = f64;

pub trait MidiDispatcher {
	fn dispatch_midi_in(&mut self, msg: &midistream::Msg);
}

pub trait Generator {
	fn generate(&mut self) -> [f32; 2];
}

pub trait SampleRated {
	fn set_sample_rate(&mut self, sample_rate: SampleRate);
}
