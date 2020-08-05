include!("lib.rs");
use crossbeam::channel::{Sender};
use midistream::Msg;

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

fn dispatch_midi_in(msg: midistream::Msg, tx: &mut Sender<Msg>) {
	use midistream::*;
	match msg {
		Msg::Simple(x) => match x {
			SimpleMsg::NoteOn(y) => {
				println!("Note on {:?}", y);
				tx.send(msg);
			},
			y => {
				println!("{:?}", y);
			},
		},
		Msg::Complex(x) => {
			println!("Received {:?}", x);
		},
		Msg::Sysex(x) => {
			println!("Received {:?}", x);
		},
	}
}

fn main() {
	let osc = crate::oscillator::Oscillator::new(crate::oscillator::Waveforms::Saw);
	let mut midi = crate::midi::InputThread::new();
	let mut sys = crate::audio::System::new(Box::new(osc));
	println!("Sample format: {:?}", sys.sample_format());
	println!("Config = {:?}", sys.config);

	midi.run();
	sys.run().unwrap();

	'outer: loop {
		let mut tx1 = sys.tx.clone();
		if let Some(msg) = midi.rx.recv() {
			dispatch_midi_in(msg, &mut tx1);
		}
	};
}
