use rand::Rng;
use std::f64::consts::PI;

include!("temperament.rs");


const TABLE_SIZE: usize = 524288;
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

enum Waveforms {
	Sine,
	Square,
	Triangle,
	Saw,
	Noise,
}

struct WaveTable {
	table_size: usize,
	table: Vec<f64>,
	//i64 tableFactor
}
type Waveform = WaveTable;

impl WaveTable {
	pub fn new(waveform: Waveforms) -> WaveTable {
		println!("WaveTable::new");
		let mut ret = WaveTable {
			table_size: TABLE_SIZE,
			table: vec![0.0_f64; TABLE_SIZE],
		};
		ret.setup_table(waveform);
		ret
	}
	fn lookup(&self, phase: &mut Counter) -> f64 {
		let y0: f64 = self.table[phase.phase as usize];
		let y1: f64 = self.table[1 + phase.phase as usize];
		let f0: f64 = phase.frac();
		let f1: f64 = 1. - f0;
		phase.increment();
		y0 * f1  +  y1 * f0
	}
	fn setup_table(&mut self, waveform: Waveforms) {
		let table_size = self.table_size;
		use Waveforms::*;
		match waveform {
			Sine => {
				let f = 2. * PI / table_size as f64;
				for t in 0..table_size {
					self.table[t] = (t as f64 * f).sin();
				}
			},
			Triangle => {
				for t in 0..(table_size / 4) {
					self.table[t] = t as f64 / (table_size as f64 / 4.);
				}
				for t in (table_size / 4)..(3 * table_size / 4) {
					self.table[t] = 2.0 - (t as f64) / (table_size as f64 / 4.);
				}
				for t in (3 * table_size / 4)..table_size {
					self.table[t] = -4.0 + (t as f64 / (table_size as f64 / 4.));
				}
			},
			Square => {
				let cycle: f64 = 0.5;
				let duty: usize = (table_size as f64 * cycle) as usize;

				self.table[0] = 0.;	
				for t in 1..duty {
					self.table[t] = 1.;
				}
				self.table[duty] = 0.;
				for t in (duty+1)..table_size {
					self.table[t] = -1.;
				}
			},
			Saw => {
				let f: f64 =  2.0 / (table_size as f64);
				for t in 0..(table_size / 2) {
					self.table[t] = t as f64 * f;
				}
				for t in (table_size / 2)..table_size {
					self.table[t] = (t as f64 * f) - 2.;
				}
			},
			Noise => {
				for t in 0..table_size {
					self.table[t] = 2.0 * rand::thread_rng().gen::<f64>() - 1.0;
				}
			},
		}
	}
}


#[derive(Clone, Copy)]
struct Note {
	phase: Counter,
	amp_env: ADSR,
	flt_env: ADSR,
	down: bool,
	vel: f64,
	num: i8,
}
impl Note {
	pub fn new() -> Note {
		Note {
			phase: Counter::new(),
			amp_env: ADSR::new(),
			flt_env: ADSR::new(),
			down: false,
			vel: 0.,
			num: 0,
		}
	}
}

const max_poly: usize = 128;
struct Oscillator {
	active: bool,

	//filtLP1: Filter, filtHP1: Filter,
	//filtLP2: Filter, filtHP2: Filter,
	//delay: Delay,
	//pan: Pan,

	clk: usize,
	
	//amp: f64, pitchbend: f64, bendInt: f64,
	//dist: f64, fLP: f64, fHP: f64, qLP: f64, qHP: f64,
	
	//lfo: WaveTable,
	//lfoNote: Note,
	wf: Waveform,
	
	temper: TuningType,
	sus: i8, poly: usize,
	low_note: usize, high_note: usize, cur_note: usize,
	//hi_assign: usize, lo_assign: usize,
	//lfof: f64,
	//lfo2lp: f64, lfo2hp: f64, lfo2amp: f64, env2lp: f64,

	notes: Vec<Note>,
}

impl Oscillator {
	pub fn new(waveform: Waveforms) -> Oscillator {
		let mut notes = vec![Note::new(); 128];
		println!("Oscillator::new");
		let mut osc = Oscillator {
			notes: notes,
			temper: Tuning::EquaTemp,
			poly: 0,
			low_note: 0,
			high_note: 127,
			sus: 0,
			active: false,
			cur_note: 0,
			clk: 0,
			wf: Waveform::new(waveform),
		};
		osc.retemper();
		osc.active = true;
		osc
	}
	fn retemper(&mut self) {
		println!("Oscillator::retemper");
		let temp = &TUNINGS[&self.temper];
		for n in 0..128 {
			self.notes[n].phase.set_freq(temp.lookup(n as i8));
		}
	}
	fn note_off(&mut self, n: i8) {
		self.poly -= 1;
		let note: &mut Note = &mut self.notes[n as usize];
		note.down = false;
		if self.sus < 64 {
			note.amp_env.gate_close();
			note.flt_env.gate_close();
		}

		if self.high_note == n as usize {
			while self.high_note >  0  && self.notes[self.high_note].num == 0 {
				self.high_note -= 1;
			}
		}
		if self.low_note  == n as usize {
			while self.low_note  < 127 && self.notes[self.low_note ].num == 0 {
				self.low_note += 1;
			}
		}
		if self.low_note == 0 && self.high_note == 127 {
			self.active = false;
		}
		let mut c1: usize = self.cur_note;
		let mut c2: usize = self.cur_note;
		while c1 > 0 && c2 < 127 {
			if self.notes[c1].down {
				self.cur_note = c1;
				break;
			}
			if self.notes[c2].down {
				self.cur_note = c2;
				break;
			}
			c1 -= 1;
			c2 += 1;
		}
	}
	fn note_on(&mut self, n: i8, v: i8) {
		//self.notes[n].freq  = calc_freq(n); // this should already be precomputed.
		//self.notes[n].phase = 0; // let the piano class do this itself.	
		//self.notes[n].time  = 0;
		let un = n as usize;
		self.cur_note = un;
		let note: &mut Note = &mut self.notes[n as usize];
		note.num   = n;
		note.amp_env.gate_open();
		note.flt_env.gate_open();
		note.down  = true;

		if v >= 0 {
			note.vel = v as f64 / 127.;
		}	

		if self.poly < max_poly - 1 { self.poly += 1; }

		if self.high_note < un {
			self.high_note = un;
		}
		if self.low_note  > un {
			self.low_note  = un;
		}

		// self.calc_chord_ratio();

		self.active = true;
	}
}

#[test]
fn test_oscillator() {
	Oscillator::new();
}
