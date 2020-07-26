include!("temperament.rs");


const table_size: usize = 524288;

use std::f64::consts::PI;

const SHIFT: u32 = 32 - 19;
const resolution: f64 = 4294967295.;

#[derive(Clone, Copy)]
struct Counter {
	phase: u32,
	incr: u32,
	bits: u32,
}
impl Into<u32> for Counter {
	fn into(self) -> u32 {
		self.phase >> self.bits
	}
}
impl Into<usize> for Counter {
	fn into(self) -> usize {
		(self.phase >> self.bits) as usize
	}
}
impl Counter {
	pub fn new() -> Counter {
		Counter {
			phase: 0,
			incr: 1,
			bits: SHIFT,
		}
	}
	fn calc_freq(f: f64) -> f64 {
		const dsr: f64 = resolution / SAMPLE_RATE as f64;
		f * dsr
	}
	fn set_freq(&mut self, f: f64) {
		self.incr = Self::calc_freq(f) as u32;
	}
	/*
	fn note_freq(temper: &Temperament, n: u8) -> f64 {
		Self::calc_freq( temper.lookup(n) )
	}
	fn set_note(&mut self, freq: u32) {
		self.incr = Self::note_freq(freq) as u32;
	}
	*/
	fn set_shift(&mut self, i: u32){
		self.bits = i;
	}
	fn increment(&mut self) {
		self.phase += self.incr;
	}
	fn set_phase(&mut self, i: u8) {
		self.phase = ((i as u32) << self.bits).into();
	}
	fn int(&self) -> u32 { // remainder as integer component
		self.phase & (( 2 << self.bits ) - 1)
	}
	fn frac(&self) -> f64 { // remainder as a fraction
		self.int() as f64 / (1 << self.bits) as f64
	}
}
/*
trait Waveform {
	pub fn lookup(&self, phase: f64) -> f64;
}
*/

enum Waveforms {
	Sine,
}

struct WaveTable {
	table_size: usize,
	table: [f64; table_size],
	//i64 tableFactor
}

impl WaveTable {
	fn lookup(&self, phase: &mut Counter) -> f64 {
		let y0: f64 = self.table[phase.phase as usize];
		let y1: f64 = self.table[1 + phase.phase as usize];
		let f0: f64 = phase.frac();
		let f1: f64 = (1. - f0);
		phase.increment();
		y0 * f1  +  y1 * f0
	}
	fn setup_table(&mut self, waveform: Waveforms) {
		match waveform {
			Sine => {
				let f = 2. * PI / table_size as f64;
				for t in 0..table_size {
					self.table[t] = (t as f64 * f).sin();
				}
			}
		}
	}
}


#[derive(Clone, Copy)]
struct Note {
	phase: Counter,
}
impl Note {
	pub fn new() -> Note {
		Note {
			phase: Counter::new()
		}
	}
}

struct Oscillator {
	notes: [Note; 128],
	temper: TuningType,
}

impl Oscillator {
	pub fn new() -> Oscillator {
		let mut notes = [Note::new(); 128];
		let mut osc = Oscillator {
			notes: notes,
			temper: Tuning::EquaTemp,
		};
		osc.retemper();
		osc
	}
	fn retemper(&mut self) {
		let temp = &TUNINGS[&self.temper];
		for n in 0..128 {
			self.notes[n].phase.set_freq(temp.lookup(n as i8));
		}
	}
}
