use rand::Rng;
use std::num::Wrapping;
use std::f64::consts::PI;

include!("temperament.rs");


const TABLE_BITS: usize = 19;
const TABLE_SIZE: usize = 1 << TABLE_BITS;
const SHIFT: u32 = 32 - TABLE_BITS as u32;
const resolution: f64 = (1_i64 << 32) as f64;

type Index = u32;


#[derive(Clone, Copy)]
struct Counter {
	phase: Wrapping<Index>,
	incr: Wrapping<Index>,
	bits: Wrapping<Index>,
}
impl Into<usize> for Counter {
	fn into(self) -> usize {
		(self.phase.0 >> self.bits.0) as usize
	}
}
impl Counter {
	pub fn new() -> Counter {
		Counter {
			phase: Wrapping(0_u32),
			incr: Wrapping(1_u32),
			bits: Wrapping(SHIFT),
		}
	}
	fn calc_freq(f: f64) -> f64 {
		const dsr: f64 = resolution / SAMPLE_RATE as f64;
		f * dsr
	}
	fn set_freq(&mut self, f: Frequency) {
		self.incr = Wrapping(Self::calc_freq(f) as u32);
	}
	/*
	fn note_freq(temper: &Temperament, n: u8) -> Frequency {
		Self::calc_freq( temper.lookup(n) )
	}
	fn set_note(&mut self, freq: u32) {
		self.incr = Self::note_freq(freq) as u32;
	}
	*/
	fn set_shift(&mut self, i: Index){
		self.bits.0 = i;
	}
	fn increment(&mut self) {
		self.phase += self.incr;
	}
	fn set_phase(&mut self, i: u8) {
		self.phase = Wrapping(((i as Index) << self.bits.0).into());
	}
	fn int(&self) -> Index {
		self.phase.0 >> self.bits.0
	}
	fn modulo(&self) -> Index { // remainder as integer component
		self.phase.0 & (( 2 << self.bits.0 ) - 1)
	}
	fn frac(&self) -> f64 { // remainder as a fraction
		self.modulo() as f64 / (1 << self.bits.0) as f64
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Waveforms {
	Sine,
	Square,
	Triangle,
	Saw,
	Noise,
}

type Sample = f64;

struct WaveTable {
	table_size: usize,
	table: Vec<Sample>,
	//i64 tableFactor
}
type Waveform = WaveTable;

impl WaveTable {
	pub fn new(waveform: Waveforms) -> WaveTable {
		println!("WaveTable::new");
		let mut ret = WaveTable {
			table_size: TABLE_SIZE,
			table: vec![0.; TABLE_SIZE],
		};
		ret.setup_table(waveform);
		ret
	}
	fn lookup(&self, phase: &mut Counter) -> Sample {
		let y0: Sample = self.table[phase.phase.0 as usize];
		let y1: Sample = self.table[1 + phase.phase.0 as usize];
		let f0: Sample = phase.frac();
		let f1: Sample = 1. - f0;
		phase.increment();
		y0 * f1  +  y1 * f0
	}
	fn setup_table(&mut self, waveform: Waveforms) {
		let table_size = self.table_size;
		use Waveforms::*;
		match waveform {
			Sine => {
				let f = 2. * PI / table_size as Frequency;
				for t in 0..table_size {
					self.table[t] = (t as Frequency * f).sin() as Sample;
				}
			},
			Triangle => {
				for t in 0..(table_size / 4) {
					self.table[t] = t as f64 / (table_size as Frequency / 4.);
				}
				for t in (table_size / 4)..(3 * table_size / 4) {
					self.table[t] = 2.0 - (t as Frequency) / (table_size as Frequency / 4.);
				}
				for t in (3 * table_size / 4)..table_size {
					self.table[t] = -4.0 + (t as Frequency / (table_size as Frequency / 4.));
				}
			},
			Square => {
				let cycle: Sample = 0.5;
				let duty: usize = (table_size as Frequency * cycle) as usize;

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
				let f: f64 =  2.0 / (table_size as Frequency);
				for t in 0..(table_size / 2) {
					self.table[t] = t as Frequency * f;
				}
				for t in (table_size / 2)..table_size {
					self.table[t] = (t as Frequency * f) - 2.;
				}
			},
			Noise => {
				for t in 0..table_size {
					self.table[t] = 2.0 * rand::thread_rng().gen::<Sample>() - 1.0;
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
		println!("Oscillator::new");
		let mut osc = Oscillator {
			notes: vec![Note::new(); 128],
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
			note.vel = v as Sample / 127.;
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

/*
impl Oscillator {
	fn generate(out: [Vec<f64>; 2]) {
		let mut o: Sample = 0.;
		let mut sL: Sample = 0.;
		let mut sR: Sample = 0.;

		for i in 0..frames_per_buf {
			sL = 0.0;

			for n in low_note..high_note {
				let note = &notes[n];
				if note.num != 0 {
					do_adsr(n);
				}
				if note.num != 0 {
					sL += wf.lookup(note) * note.amp * note.vel;
				}
			}
			clk += 1;
			
			//o = applyEffects(sL);
			
			sL = o * pan.amp_l;
			sR = o * pan.amp_r;
			
			out[0][i] = sL;
			out[1][i] = sR;
		}
		//paContinue
	}
}
*/

#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
lazy_static! {
	static ref WAVEFORMS: HashMap<Waveforms, WaveTable> = {
		use Waveforms::*;
		let mut hash: HashMap<Waveforms, WaveTable> = HashMap::new();
		for wave in &[Sine, Saw, Triangle, Square, Noise] {
			hash.insert(*wave, WaveTable::new(*wave));
		}
		hash
	};
}

#[test]
fn test_oscillator() {
	Oscillator::new(Waveforms::Saw);
}

#[test]
fn test_counter() {
	assert_eq!(1 << 19, 524288);
	let mut c = Counter::new();
	let size = TABLE_SIZE as u32;
	c.set_freq(SAMPLE_RATE as Frequency / 4.);
	assert_eq!(c.int(), 0);
	c.increment(); assert_eq!(c.int(), size/4);
	c.increment(); assert_eq!(c.int(), size/2);
	c.increment(); assert_eq!(c.int(), 3*size/4);
	c.increment(); assert_eq!(c.int(), 0);
}
