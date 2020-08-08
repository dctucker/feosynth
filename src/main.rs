extern crate signal_hook;
include!("lib.rs");
//use crossbeam::channel::{Sender};
//use midistream::Msg;

/*
fn temperaments() {
	use crate::temperament::{TUNINGS, cents};
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

fn adsr() {
	use crate::adsr::*;
	let mut adsr = ADSR::new();
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
*/

fn main() {
	let interrupt = std::sync::Arc::new( std::sync::atomic::AtomicBool::new( false ) );
	signal_hook::flag::register(signal_hook::SIGINT, std::sync::Arc::clone(&interrupt)).unwrap();

	let osc = Box::new(crate::oscillator::Oscillator::new(crate::oscillator::Waveforms::Sine));
	let mut midi = crate::midi::InputThread::new();
	let sys = crate::audio::System::new();
	println!("Sample format: {:?}", sys.sample_format());
	println!("Config = {:?}", sys.config);

	//let tx = sys.tx.clone(); tx.send(midistream::Msg::Simple(midistream::SimpleMsg::NoteOn(midistream::Note{channel: 1.into(), note: 64.into(), value: 120.into()})));
	midi.run(sys.tx.clone());
	let _stream = sys.run(osc).unwrap();
	//println!("{:?}", stream);

	'outer: loop {
		//std::thread::yield_now();
		std::thread::sleep(std::time::Duration::from_millis(500));
		if interrupt.swap(false, std::sync::atomic::Ordering::Relaxed) {
			println!("Caught interrupt, exiting outer loop");
			break;
		}
	};
}
