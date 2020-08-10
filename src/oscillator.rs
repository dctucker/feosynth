use std::collections::VecDeque;
use rand::Rng;
use std::num::Wrapping;
use std::f64::consts::PI;
use std::convert::TryInto;

use super::types::{SampleRated, Generator, SampleRate, Frequency, Sample, MidiDispatcher};
use super::adsr::*;
use super::temperament::{Tuning,TuningData};

const TABLE_BITS: usize = 19;
const TABLE_SIZE: usize = 1 << TABLE_BITS;
const SHIFT: u32 = 32 - TABLE_BITS as u32;
const RESOLUTION: f64 = (1_i64 << 32) as f64;

type TablePos = u32;

#[derive(Clone, Copy, Debug)]
struct Counter {
	phase: Wrapping<TablePos>,
	incr: Wrapping<TablePos>,
	bits: Wrapping<TablePos>,
	dsr: f64,
}
impl Into<usize> for Counter {
	fn into(self) -> usize {
		(self.phase.0 >> self.bits.0) as usize
	}
}
impl SampleRated for Counter {
	fn set_sample_rate(&mut self, sample_rate: SampleRate) {
		self.dsr = RESOLUTION / sample_rate as f64;
	}
}
impl Counter {
	pub fn new() -> Counter {
		Counter {
			phase: Wrapping(0_u32),
			incr: Wrapping(1_u32),
			bits: Wrapping(SHIFT),
			dsr: 0.,
		}
	}
	fn calc_freq(&self, f: f64) -> f64 {
		f * self.dsr
	}
	fn set_freq(&mut self, f: Frequency) {
		self.incr = Wrapping(self.calc_freq(f) as u32);
		//println!("{}", self.incr);
	}
	/*
	fn note_freq(temper: &Temperament, n: u8) -> Frequency {
		Self::calc_freq( temper.lookup(n) )
	}
	fn set_note(&mut self, freq: u32) {
		self.incr = Self::note_freq(freq) as u32;
	}
	*/
	fn increment(&mut self) {
		self.phase += self.incr;
	}
	fn set_phase(&mut self, i: u8) {
		self.phase = Wrapping(((i as TablePos) << self.bits.0).into());
	}
	fn int(&self) -> TablePos {
		self.phase.0 >> self.bits.0
	}

	#[allow(dead_code)]
	fn set_shift(&mut self, i: TablePos){
		self.bits.0 = i;
	}
	#[allow(dead_code)]
	fn modulo(&self) -> TablePos { // remainder as integer component
		self.phase.0 & (( 2 << self.bits.0 ) - 1)
	}
	#[allow(dead_code)]
	fn frac(&self) -> f64 { // remainder as a fraction
		self.modulo() as f64 / (1 << self.bits.0) as f64
	}
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Waveforms {
	Sine,
	Square,
	Triangle,
	Saw,
	Noise,
}

struct WaveTable {
	table_size: usize,
	table: Vec<Sample>,
	//i64 tableFactor
}

impl WaveTable {
	pub fn new(waveform: Waveforms) -> WaveTable {
		let mut ret = WaveTable {
			table_size: TABLE_SIZE,
			table: vec![0.; TABLE_SIZE],
		};
		ret.setup_table(waveform);
		ret
	}
	fn lookup(&self, phase: &mut Counter) -> Sample {
		let p0 = phase.int() as usize; phase.increment();
		//let p1 = phase.int() as usize;
		let y0: Sample = self.table[p0];
		y0
		//unsafe { let y0: &Sample = self.table.get_unchecked(p0); *y0 }
		/*
		let y1: Sample = self.table[p1];
		let f0: Sample = phase.frac();
		let f1: Sample = 1. - f0;
		y0 * f1  +  y1 * f0
		*/

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


#[derive(Clone, Debug)]
struct Note {
	phase: Counter,
	amp: Sample,
	flt: Sample,
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
			amp: 0.,
			amp_env: ADSR::new(),
			flt: 0.,
			flt_env: ADSR::new(),
			down: false,
			vel: 0.,
			num: 0,
		}
	}
}
impl SampleRated for Note {
	fn set_sample_rate(&mut self, sample_rate: SampleRate) {
		self.phase.set_sample_rate(sample_rate);
		self.amp_env.set_sample_rate(sample_rate);
		self.flt_env.set_sample_rate(sample_rate);
	}
}

const MAX_POLY: usize = 128;
pub struct Oscillator {
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
	wf: WaveTable,
	
	tuning_preset: Tuning,
	temperament: TuningData,
	sus: i8, poly: usize,
	//low_note: usize, high_note: usize, cur_note: usize,
	//hi_assign: usize, lo_assign: usize,
	//lfof: f64,
	//lfo2lp: f64, lfo2hp: f64, lfo2amp: f64, env2lp: f64,

	notes: Vec<Option<Box<Note>>>,
	active_notes: VecDeque<Box<Note>>,
}

impl SampleRated for Oscillator {
	fn set_sample_rate(&mut self, sample_rate: SampleRate) {
		for note1 in (&mut self.notes).iter_mut() {
			for note in note1.iter_mut() {
				note.set_sample_rate(sample_rate);
			}
		}
		self.retemper();
	}
}

impl Oscillator {
	pub fn new(waveform: Waveforms) -> Oscillator {
		let mut osc = Oscillator {
			notes: vec![Some(Box::new(Note::new())); 128],
			active_notes: VecDeque::new(),
			tuning_preset: Tuning::EquaTemp,
			temperament: TuningData::new(Tuning::EquaTemp),
			poly: 0,
			//low_note: 0, high_note: 127, cur_note: 0,
			sus: 0,
			active: false,
			clk: 0,
			wf: WaveTable::new(waveform),
		};
		osc.retemper();
		osc.active = true;
		//osc.note_on(64,120);
		osc
	}
	fn retemper(&mut self) {
		println!("Oscillator::retemper");
		self.temperament = super::temperament::TUNINGS[self.tuning_preset];
		//println!("{}", self.temperament);
		let mut n: usize = 0;
		for note1 in self.notes.iter_mut() {
			for note in note1.iter_mut() {
				note.num = n as i8;
				note.phase.set_freq(self.temperament.lookup(n as i8));
				println!("{:?}", note);
			}
			n += 1;
		}
	}
	fn note_off(&mut self, n: i8) {
		self.poly -= 1;
		let un = n as usize;
		//let note: &mut Note = &mut self.notes[un].unwrap();

		let mut n_off = 0;
		for note in self.active_notes.iter() {
			if note.num == n {
				let mut note = self.active_notes.remove(n_off as usize).unwrap();

				note.down = false;
				if self.sus < 64 {
					note.amp_env.gate_close();
					note.flt_env.gate_close();
				}
				self.notes[un].replace(note);
				break;
			}
			n_off += 1;
		}

		/*
		if self.high_note == n as usize {
			while self.high_note >  0  && self.notes[self.high_note].unwrap().num == 0 {
				self.high_note -= 1;
			}
		}
		if self.low_note  == n as usize {
			while self.low_note  < 127 && self.notes[self.low_note ].unwrap().num == 0 {
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
		*/
	}
	fn note_on(&mut self, n: i8, v: i8) {
		//self.notes[n].freq  = calc_freq(n); // this should already be precomputed.
		//self.notes[n].phase = 0; // let the piano class do this itself.	
		//self.notes[n].time  = 0;
		let un = n as usize;
		//self.cur_note = un;
		let mut note: Box<Note> = self.notes[un].take().unwrap();
		note.num   = n;
		note.amp_env.gate_open();
		note.flt_env.gate_open();
		note.down  = true;

		if v >= 0 {
			note.vel = v as Sample / 127.;
		}	

		if self.poly < MAX_POLY - 1 { self.poly += 1; }

		self.active_notes.push_front(note);
		/*
		if self.high_note < un {
			self.high_note = un;
		}
		if self.low_note  > un {
			self.low_note  = un;
		}
		*/

		// self.calc_chord_ratio();

		self.active = true;
		println!("{}", self.active_notes.len());
	}
	fn do_adsr(note: &mut Note) {
		note.amp = note.amp_env.run();
		note.flt = note.flt_env.run();

		if note.amp_env.is_off() {
			note.flt_env.gate_close();
			note.num = 0;
			note.phase.set_phase(0);
		}
	}
}


impl Generator for Oscillator {
	fn generate(&mut self) -> [f32; 2] {
		//let mut o: Sample = 0.;
		let mut left: Sample = 0.;
		let right: Sample;

		for note in self.active_notes.iter_mut() {
			Self::do_adsr(note);
			left += self.wf.lookup(&mut note.phase) * note.amp * note.vel;
		}
		self.clk += 1;

		if self.clk % 96000 == 0 {
			println!("{:?}", self.active_notes);
		}
		
		//o = applyEffects(left);
		//o = left;
		//left = o;// * self.pan.amp_l;
		right = left * 1.;// * self.pan.amp_r;
		
		[left as f32, right as f32]
	}
}
impl MidiDispatcher for Oscillator {
	fn dispatch_midi_in(&mut self, msg: &midistream::Msg) {
		use midistream::*;
		match msg {
			Msg::Simple(x) => match x {
				SimpleMsg::NoteOn(y) => {
					println!("Note on {:?}", y);
					let note: u8  = *y.note;
					let value: u8 = *y.value;
					self.note_on(note.try_into().unwrap(), value.try_into().unwrap());
				},
				SimpleMsg::NoteOff(y) => {
					println!("Note off {:?}", y);
					let note: u8  = *y.note;
					self.note_off(note.try_into().unwrap());
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
}

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
	let rate = 96000;
	let mut osc = Oscillator::new(Waveforms::Sine);
	assert!(osc.notes[0].phase.dsr == 0.);
	osc.set_sample_rate(rate);
	assert!(osc.notes[0].phase.dsr > 0.);
	assert!(osc.notes[64].phase.incr.0 > 0);
	println!("incr = {}", osc.notes[64].phase.incr.0);
	let v = osc.generate()[0]; println!("v = {}", v); assert!(v == 0.);

	assert!(osc.notes[64].phase.phase.0 == 0);
	assert!(osc.notes[64].down == false);
	osc.note_on(64, 120);
	assert!(osc.notes[64].down == true);
	let v = osc.generate()[0]; println!("v = {}", v); assert!(v == 0.);
	let v = osc.generate()[0]; println!("v = {}", v); assert!(v != 0.);
	assert!(osc.notes[64].phase.phase.0 > 0);
}

#[test]
fn test_wavetable() {
	let rate = 96000;
	let wt = WaveTable::new(Waveforms::Sine);
	let mut c = Counter::new();
	assert!(c.int() == 0);
	c.set_sample_rate(rate);
	c.set_freq(440.);
	assert!(c.int() == 0);
	let v = wt.lookup(&mut c) as f32; println!("v = {}", v); assert!(v == 0.);
	let v = wt.lookup(&mut c) as f32; println!("v = {}", v); assert!(v != 0.);
}

#[test]
fn test_counter() {
	let rate = 96000;
	assert_eq!(1 << 19, 524288);
	let mut c = Counter::new();
	c.set_sample_rate(rate);
	let size = TABLE_SIZE as u32;
	println!("table size = {}", size);
	c.set_freq(rate as Frequency / 8.);
	assert!(c.int() == 0);
	assert!(c.modulo() == 0);
	c.increment(); assert_eq!(c.int(), size*1/8);
	c.increment(); assert_eq!(c.int(), size*1/4);
	c.increment(); assert_eq!(c.int(), size*3/8);
	c.increment(); assert_eq!(c.int(), size*4/8);
	c.increment(); assert_eq!(c.int(), size*5/8);
	c.increment(); assert_eq!(c.int(), size*6/8);
	c.increment(); assert_eq!(c.int(), size*7/8);
	c.increment(); assert_eq!(c.int(), 0);
}