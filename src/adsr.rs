use super::types::{SampleRated, SampleRate, Sample, Frequency, Seconds};

//const SAMPLE_RATE: u64 = 96000;
const ADSR_DIVISOR: u64 = 1;
const ADSR_MASK: u64 = 0x0;

pub trait Gate {
	fn gate_open(&mut self);
	fn gate_close(&mut self);
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum Stage {
	Off,
	Attack,
	Decay,
	Sustain,
	Release
}

#[derive(Copy, Clone)]
pub struct ADSR {
	stage : Stage,
	val: Sample,
	sample_rate: Frequency,
	clk: u64,
	a: Seconds, d: Seconds, s: Sample, r: Seconds,
	da: Frequency, dd: Frequency, dr: Frequency,
}

use Stage::*;

impl SampleRated for ADSR {
	fn set_sample_rate(&mut self, sample_rate: SampleRate) {
		self.sample_rate = sample_rate as Frequency / ADSR_DIVISOR as Frequency;
		self.calc();
	}
}

impl ADSR {
	pub fn new() -> ADSR {
		let mut adsr = ADSR {
			stage: Off,
			val: 0.,
			sample_rate: 0.,
			clk: 0,
			a: 0.1, d: 1.0, s: 0.75, r: 0.25,
			da: 0., dd: 0., dr: 0.,
		};
		adsr.calc();
		adsr
	}
	pub fn calc(&mut self) {
		self.da = 1. / (self.a * self.sample_rate);
		self.dd = 1. / (self.d * self.sample_rate);
		self.dr = 1. / (self.r * self.sample_rate);
	}
	pub fn set(&mut self, a: Seconds, d: Seconds, s: Sample, r: Seconds) {
		if a >= 0.0 { self.a = 15.0 * a.powf(6.0) + 0.01; }
		if d >= 0.0 { self.d =  5.0 * d.powf(2.0) + 0.01; }
		if s >= 0.0 { self.s = s; }
		if r >= 0.0 { self.r = 15.0 * r.powf(6.0) + 0.01; }
		self.calc();
	}
	pub fn value(&self) -> Sample {
		self.val
	}
	pub fn is_off(&self) -> bool {
		self.stage == Off
	}
	pub fn run(&mut self) -> Sample {
		match self.stage {
			Sustain => {
				if (self.clk | ADSR_MASK) != 0 {
					return self.val;
				}
			},
			Attack => {
				self.val += self.da;
				if self.val >= 1.0 {
					self.stage = Decay;
					self.val   = 1.;
				}
			},
			Decay => {
				self.val -= self.dd;
				if self.val <= 0.0 { self.val = 0.0; }

				if self.val < self.s {
					self.stage = Sustain;
					self.val   = self.s;
				}
			},
			Release => {
				self.val -= self.dr;
				if self.val <= 0.0 {
					self.val   = 0.0;
					self.stage = Off;
				}
			},
			Off => {},
		}
		self.val
	}
}

impl Gate for ADSR {
	fn gate_open(&mut self) {
		self.stage = Attack;
	}
	fn gate_close(&mut self) {
		self.stage = Release;
	}
}

#[test]
fn test_adsr() {
	let mut adsr = ADSR::new();
	adsr.set_sample_rate(96000);
	assert_eq!(adsr.val, 0.);
	adsr.gate_open();
	adsr.run();
	assert!(adsr.val > 0., "adsr.val <= 0");
	assert_eq!(adsr.stage, Stage::Attack);

	adsr.gate_close();
	assert_eq!(adsr.stage, Stage::Release);
}
