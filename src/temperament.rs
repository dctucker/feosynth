use std::fmt;
use super::types::{Cents, Frequency};

#[derive(Clone, Copy)]
pub enum Tuning {
	EquaTemp,
	MeanTemp,
	Just5Temp,
	KeplTemp,
	PythTemp,
	HammTemp,
	PtolTemp,
	ChinTemp,
	Dowland,
	Kirnberger,
}

trait OctaveTuning {
	fn init_octave() -> [Frequency; 12];
}

pub fn cents(f1: Frequency, f2: Frequency) -> Cents {
	1200. * (f2 / f1).log2()
}

#[derive(Clone, Copy)]
pub struct TuningData {
	pub note : i8,
	pub freq_table : [Frequency; 128],
	pub fund_table : [Frequency; 128],
	pub intervals  : [Frequency; 12],
	pub freq_a : Frequency, lo_tt : Frequency, hi_tt : Frequency,
	pub tuning : Tuning,
}
impl fmt::Display for TuningData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for n in 0..128 {
            write!(fmt, "{}: {}, ", self.fund_table[n], self.freq_table[n])?;
        }
		write!(fmt, "\n")?;
		for n in 0..12 {
            write!(fmt, "{}, ", self.intervals[n])?;
		}
        Ok(())
    }
}

fn pow2i(x: u32) -> Frequency {
	2u32.pow(x) as Frequency
}
fn pow3i(x: u32) -> Frequency {
	3u32.pow(x) as Frequency
}
fn pow2(x: Frequency) -> Frequency {
	(2.0 as Frequency).powf(x)
}
fn pow3(x: Frequency) -> Frequency {
	(3.0 as Frequency).powf(x)
}
fn pow5(x: Frequency) -> Frequency {
	(5.0 as Frequency).powf(x)
}

impl TuningData {
	pub fn new(t : Tuning) -> TuningData {
		let mut td = TuningData {
			note: 0,
			tuning: t,
			freq_table: [0.; 128],
			fund_table: [0.; 128],
			intervals:  [0.; 12],
			freq_a: 440.0, lo_tt: 0., hi_tt: 0.
		};
		td.init_octave();
		td
	}
	fn init_octave(&mut self) {
		let mut e_f = [0.; 12];
        //println!("{:?}", &self.tuning);
		match &self.tuning {
			Tuning::EquaTemp => {
				//println!("Init EquaTemp");
				for i in 0..12 {
					e_f[i] = pow2( i as Frequency / 12. );
				}
				self.init(e_f);
			},
			Tuning::MeanTemp => {
				let p = pow5(1./4.);
				let t = pow5( 1./2. ) / 2.;
				let s = 8. / pow5(5./4.);
				let z = t / s;
				e_f = [
					1.    ,   z   ,
					t     ,   t*s ,
					t*t   ,
					t*t*s , t*t*t ,
					p     ,   p*z ,
					p*t   , p*t*s ,
					p*t*t
				];
				self.init(e_f);
			},
			Tuning::Just5Temp => {
				// only factors 2,3,5 are used
				e_f = [
					1.0  , 16./15.,
					9./8.,  6./5. ,
					5./4.,
					4./3., 45./32.,
					3./2.,  8./5. ,
					5./3., 16./9. ,
					15./8.
				];
				let g_flat = 64./45.;
				let f_sharp = 45./32.;
				self.init_just( e_f, g_flat, f_sharp )
			},
			Tuning::KeplTemp => {
				let u = 1. ;
				let min2 = 135. / 128. ; // lemma = 15:16 / 8:9
				let maj2 =   9. /   8. ; // major whole tone
				let min3 =   6. /   5. ; // minor third
				let maj3 =   5. /   4. ; // major third
				let p4   =   4. /   3. ; // perfect fourth
				let tt   =  45. /  32. ; // augmented fourth
				let p5   =   3. /   2. ; // perfect fifth
				let min6 =   8. /   5. ; // minor sixth
				let maj6 =  27. /  16. ; // pythagorean major sixth
				let min7 =   9. /   5. ; // 16:9 or 9:5	
				let maj7 =  15. /   8. ; // 

				e_f = [
					u   , min2,
					maj2, min3,
					maj3,
					p4  , tt,
					p5  , min6,
					maj6, min7,
					maj7
				];
				self.init(e_f);
			},
			Tuning::PythTemp => {
				// all fifths tuned to 3:2
				//Wikipedia
				//					Ab			Eb			Bb		F		C		G		D	A		E		B		F#		C#			G#
				//float pF[13] = { 1024./729., 256./243., 128./81., 32./27., 16./9., 4./3., 1.0, 3./2., 9./8., 27./16., 81./64., 243./128., 729/512. }

				e_f = [
					   1.0   , 256./243.,
					  9./8.  ,  32./27. ,
					 81./64. ,
					  4./3.  , 729./512., //1024./729.,
					  3./2.  , 128./81. ,
					 27./16. ,  16./9.  ,
					243./128.
				];
				let a_flat = 1024./729.;
				let g_sharp = 729./512.;
				self.init_just(e_f, a_flat, g_sharp)
			},
			Tuning::HammTemp => {
				e_f = [
					 88./64.   , // A = 1.375; 20 * 16 * 1.375 = 440 Hz
					 67./46.   ,
					108./70.   ,
					 85./104.  , // C
					 71./82.   ,
					 67./73.   ,
					105./108.  ,
					103./100.  ,
					 84./77.   ,
					 74./64.   ,
					 98./80.   ,
					 96./74.
				];
				for i in 0..12 {
					e_f[i] /= 1.375;
					if e_f[i] < 1.0 {
						e_f[i] *= 2.0;
					}
				}
				self.init(e_f);
			},
			Tuning::PtolTemp => {
				e_f = [
					1.0,
					16. / 15.,
					 9. /  8.,
					 6. /  5.,
					 5. /  4.,
					 4. /  3.,
					 7. /  5.,
					 3. /  2.,
					 8. /  5.,
					 5. /  3.,
					 7. /  4.,
					15. /  8.
				];
				self.init(e_f);
			},
			Tuning::ChinTemp => {
				fn cfrac(x : u32, y : u32) -> Frequency { pow3i(x) / pow2i(y) }
				e_f = [
					cfrac( 0, 0), cfrac( 7,11),
					cfrac( 2, 3), cfrac( 9,14),
					cfrac( 4, 6),
					cfrac(11,17), cfrac( 6, 9),
					cfrac( 1, 1), cfrac( 8,12),
					cfrac( 3, 4), cfrac(10,15),
					cfrac( 5, 7)
				];
				self.init(e_f);
			},
			Tuning::Dowland => {
				e_f = [
					  1./   1.,   33./ 31.,
					  9./   8.,   33./ 28.,
					264./ 211.,
					  4./   3.,   24./ 17.,
					  3./   2.,   99./ 62.,
					 27./  16.,   99./ 56.,
					396./ 211. 
				];
				self.init(e_f);
			},
			Tuning::Kirnberger => {
				e_f = [
					1./  1.,   256./243.,
					9./  8.,    32./ 27.,
					5./  4., 
					4./  3.,    45./ 32.,
					3./  2.,   128./ 81.,
				  270./161.,    16./  9.,
				   15./  8. 		
				];
				self.init(e_f);
			},
		};
	}
	pub fn lookup(&self, n : i8) -> Frequency {
		if n < 0 {
			0.
		} else {
			self.freq_table[n as usize]
		}
	}
	pub fn retune(&mut self, a : Frequency) {
		self.freq_a = a;
		self.init(self.intervals);
	}

	pub fn init(&mut self, j_f : [Frequency; 12]) {
		self.init_intervals(j_f);
		for i in 0..128 {
			self.freq_table[i] = self.freq_a;
			self.freq_table[i] *= self.intervals[(i+3) % 12];
			self.freq_table[i] *= self.note_octave(i);
		}
		self.init_fund();
	}

	pub fn init_fund(&mut self) {
		for i in 0..12 {
			self.fund_table[i] = self.freq_table[69 + i];
			//println!("Init Interval {} = {}", i, self.fund_table[i]);
		}
	}

	pub fn init_intervals(&mut self, j_f : [Frequency; 12]) {
		for i in 0..12 {
			self.intervals[i] = j_f[i];
		}
	}

	pub fn note_octave(&self, i : usize) -> Frequency {
		pow2((((i as i32 + 3) / 12) - 6) as Frequency)
	}

	pub fn modulate(&mut self, m : i8) {
		let m_n = 69 + m as usize;
		
		// fails on Just5 .. missing tritone [on upper octave]?
		
		let new_center: Frequency = self.fund_table[m as usize];
		println!("New center: {}", new_center);
		for i in 0..12 {
			let mut n: usize = i + m_n as usize;
			self.freq_table[n as usize] = self.intervals[i] * new_center;
			//println!("Interval {} = {}", i, self.freq_table[n]);

			n -= 12;
			let mut o = -1;
			loop {
				self.freq_table[n] = self.intervals[i] * new_center * pow2(o as Frequency);
				if n < 12 { break; }
				n -= 12;
				o -= 1;
			}

			n = 12 + i + m_n as usize;
			o = 1;
			loop {
				self.freq_table[n] = self.intervals[i] * new_center * pow2(o as Frequency);
				if n > 116 { break; }
				n += 12;
				o += 1;
			}
		}
	}

	pub fn init_just(&mut self, j_f : [Frequency; 12], g_flat : Frequency, f_sharp : Frequency) {
		let j = 3; // oh no, we're tuning relative to A=440
		self.lo_tt = g_flat;
		self.hi_tt = f_sharp;

		self.init_intervals(j_f);
		for i in 0..128 {
			self.freq_table[i] = self.freq_a * self.note_octave(i);
			
			if self.intervals[j % 12] == 0. {
				if i < 62 {
					self.freq_table[i] *= self.lo_tt;
				} else {
					self.freq_table[i] *= self.hi_tt;
				}
			} else {
				self.freq_table[i] *= self.intervals[j % 12];
			}
		}
		self.init_fund();
	}

	pub fn init_temp(tuning : Tuning) -> Tuning {
		tuning
	}
}

use std::ops::Index;

pub struct Tunings<T> {
	equa_temp : T,
	mean_temp : T,
	just5_temp : T,
	kepl_temp : T,
	pyth_temp : T,
	hamm_temp : T,
	ptol_temp : T,
	chin_temp : T,
	dowland : T,
	kirnberger : T,
}
impl Tunings<TuningData> {
	pub fn new() -> Tunings<TuningData> {
		Tunings {
			equa_temp:   TuningData::new(Tuning::EquaTemp),
			mean_temp:   TuningData::new(Tuning::MeanTemp),
			just5_temp:  TuningData::new(Tuning::Just5Temp),
			kepl_temp:   TuningData::new(Tuning::KeplTemp),
			pyth_temp:   TuningData::new(Tuning::PythTemp),
			hamm_temp:   TuningData::new(Tuning::HammTemp),
			ptol_temp:   TuningData::new(Tuning::PtolTemp),
			chin_temp:   TuningData::new(Tuning::ChinTemp),
			dowland:     TuningData::new(Tuning::Dowland),
			kirnberger:  TuningData::new(Tuning::Kirnberger),
		}
	}
}
impl Index<Tuning> for Tunings<TuningData> {
	type Output = TuningData;
	fn index(&self, preset : Tuning) -> &Self::Output {
		match preset {
			Tuning::EquaTemp   => &self.equa_temp,
			Tuning::MeanTemp   => &self.mean_temp,
			Tuning::Just5Temp  => &self.just5_temp,
			Tuning::KeplTemp   => &self.kepl_temp,
			Tuning::PythTemp   => &self.pyth_temp,
			Tuning::HammTemp   => &self.hamm_temp,
			Tuning::PtolTemp   => &self.ptol_temp,
			Tuning::ChinTemp   => &self.chin_temp,
			Tuning::Dowland    => &self.dowland,
			Tuning::Kirnberger => &self.kirnberger,
		}
	}
}

type Temperament = Tuning;

lazy_static! {
	pub static ref TUNINGS: Tunings<TuningData> = Tunings::new();
	/*
	static ref TUNINGS: HashMap<Tuning, TuningData> = {
		use Tuning::*;
		let mut hash: HashMap<Tuning, TuningData> = HashMap::new();
		for temp in &[EquaTemp, MeanTemp, Just5Temp, KeplTemp, PythTemp, HammTemp, PtolTemp, ChinTemp, Dowland, Kirnberger] {
			hash.insert(*temp, Tuning::new(*temp));
		}
		hash
	};
	*/
}

#[test]
fn test_temperaments() {
	let ptol = &TUNINGS[Tuning::PtolTemp];
	assert_eq!( ptol.lookup(60), 264. );
	assert_eq!( ptol.lookup(64), 330. );
}
