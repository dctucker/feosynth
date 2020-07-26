const SAMPLE_RATE: u64 = 96000;
const ADSR_DIVISOR: u64 = 1;
const ADSR_MASK: u64 = 0x0;

trait Gate {
	fn gate_open(&mut self);
	fn gate_close(&mut self);
}

#[derive(PartialEq)]
#[derive(Debug)]
enum Stage {
	Off,
	Attack,
	Decay,
	Sustain,
	Release
}
type StageType = Stage;

struct ADSR {
	stage : StageType,
	a: f64, d: f64, s: f64, r: f64,
	da: f64, dd: f64, ds: f64, dr: f64,
	clk: u64,
	val: f64,
}

use Stage::*;

impl ADSR {

	pub fn new() -> ADSR {
		let mut adsr = ADSR {
			stage: Off,
			a: 0.1, d: 1.0, s: 0.75, r: 0.25,
			da: 0., dd: 0., ds: 0., dr: 0.,
			clk: 0,
			val: 0.
		};
		adsr.calc();
		adsr
	}
	pub fn calc(&mut self) {
		const k: f64 = SAMPLE_RATE as f64 / ADSR_DIVISOR as f64;
		self.da = 1.0_f64 / (self.a * k);
		self.dd = 1.0_f64 / (self.d * k);
		self.dr = 1.0_f64 / (self.r * k);
	}
	pub fn set(&mut self, a: f64, d: f64, s: f64, r: f64) {
		if a >= 0.0 { self.a = 15.0 * a.powf(6.0) + 0.01; }
		if d >= 0.0 { self.d =  5.0 * d.powf(2.0) + 0.01; }
		if s >= 0.0 { self.s = s; }
		if r >= 0.0 { self.r = 15.0 * r.powf(6.0) + 0.01; }
		self.calc();
	}
	pub fn run(&mut self) -> f64 {
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
	assert_eq!(adsr.val, 0.);
	adsr.gate_open();
	adsr.run();
	assert!(adsr.val > 0., "adsr.val <= 0");
	assert_eq!(adsr.stage, Stage::Attack);

	adsr.gate_close();
	assert_eq!(adsr.stage, Stage::Release);
}
