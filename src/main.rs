include!("adsr.rs");
include!("oscillator.rs");
//include!("temperament.rs");

fn temperaments() {
	let equal = &TUNINGS[&Tuning::EquaTemp];
	let other = &TUNINGS[&Tuning::PtolTemp];
	for n in 57..70 {
		let e = equal.lookup(n);
		let c = other.lookup(n);
		let diff = cents(e, c);
		/*
		println!("Note {}: {} Hz", n, equal.lookup(n));
		println!("Note {}: {} Hz", n, other.lookup(n));
		*/
		println!("Difference {}: {}", n, diff);
	}
}

fn adsr() {
	let mut adsr = ADSR::new();
	adsr.gate_open();
	let mut v = adsr.val;
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

fn oscillator() {
	let mut osc = Oscillator::new();
}

fn main() {
	temperaments();
	oscillator();
}
