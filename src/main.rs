#[macro_use]
extern crate lazy_static;

mod types;
mod audio;
mod adsr;
mod oscillator;
mod temperament;
//include!("temperament.rs");

fn temperaments() {
	use temperament::{TUNINGS, cents};
	let equal = &TUNINGS[temperament::Tuning::EquaTemp];
	let other = &TUNINGS[temperament::Tuning::PtolTemp];
	for n in 57..70 {
		let e = equal.lookup(n);
		let c = other.lookup(n);
		let diff = cents(e, c);
		/*
		println!("Note {}: {} Hz", n, equal.lookup(n));
		println!("Note {}: {} Hz", n, other.lookup(n));
		*/
		println!("{}: {} - {} = {}", n, c, e, diff);
	}
}

fn adsr(sr: types::SampleRate) {
	use adsr::*;
	let mut adsr = ADSR::new(sr);
	adsr.gate_open();
	let mut v = adsr.value();
	for t in 0..96000 {
		if t % 4800 == 0 {
			println!("ADSR = {}", v);
		}
		v = adsr.run();
		if t == 48000 {
			adsr.gate_close();
		}
	}
}

fn oscillator(sr: types::SampleRate) {
	use oscillator::{Oscillator, Waveforms};
	let mut _osc = Oscillator::new(sr, Waveforms::Sine);
}

fn main() {
	let sys = audio::System::new();
	let sample_rate = sys.config.sample_rate.0;
	temperaments();
	adsr(sample_rate);
	oscillator(sample_rate);
	println!("Sample format: {:?}", sys.sample_format());
	println!("Config = {:?}", sys.config);
	sys.run_config();
}
