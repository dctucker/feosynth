enum Tuning {
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
type TuningType = Tuning;

trait OctaveTuning {
	fn init_octave() -> [f32; 12];
}


struct TuningData {
	pub note : i8,
	pub freq_table : [f32; 128],
	pub fund_table : [f32; 128],
	pub intervals  : [f32; 12],
	pub freq_a : f32, lo_tt : f32, hi_tt : f32,
	pub tuning : TuningType,
}

impl Tuning {
	pub fn new(t : TuningType) -> TuningData {
		let mut td = TuningData {
			note: 0,
			tuning: t,
			freq_table: [0.; 128],
			fund_table: [0.; 128],
			intervals:  [0.; 12],
			freq_a: 440.0_f32, lo_tt: 0., hi_tt: 0.
		};
		td.init(td.init_octave());
		td
	}
}

impl TuningData {
	fn init_octave(&self) -> [f32; 12] {
		let mut e_f = [0.0_f32; 12];
		match &self.tuning {
			EquaTemp => {
				for i in 0..12 {
					e_f[i] = 2.0_f32.powf( i as f32 / 12.0_f32 );
				}
			},
			MeanTemp => {
				let P = 5.0_f32.powf(1./4.);
				let T = 5.0_f32.powf( 1./2. ) / 2.;
				let S = 8.0_f32 / 5.0_f32.powf(5./4.);
				let Z = T / S;
				e_f = [
					1.    ,   Z   ,
					T     ,   T*S ,
					T*T   ,
					T*T*S , T*T*T ,
					P     ,   P*Z ,
					P*T   , P*T*S ,
					P*T*T
				];
			},
			Just5Temp => {
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
				let Gb = 64./45.;
				let Fs = 45./32.;
				//initJust( jF, Gb, Fs );
			},
			KeplTemp => {
				let U = 1. ;
				let cs  = 135. / 128. ; // lemma = 15:16 / 8:9
				let jM  =   9. /   8. ; // major whole tone
				let m3  =   6. /   5. ; // minor third
				let M3  =   5. /   4. ; // major third
				let P4  =   4. /   3. ; // perfect fourth
				let dT  =  45. /  32. ; // augmented fourth
				let P5  =   3. /   2. ; // perfect fifth
				let jm6 =   8. /   5. ; // minor sixth
				let pM6 =  27. /  16. ; // pythagorean major sixth
				let jm7 =   9. /   5. ; // 16:9 or 9:5	
				let jM7 =  15. /   8. ; // 

				e_f = [
					U   , cs,
					jM  , m3,
					M3  ,
					P4  , dT,
					P5  , jm6,
					pM6 , jm7,
					jM7
				];
			},
			PythTemp => {
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
				let Ab = 1024./729.;
				let Gs = 729./512.;
				//initJust(pF, Ab, Gs);
			},
			HammTemp => {
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
			},
			PtolTemp => {
				let e_f = [
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
			},
			ChinTemp => {
				fn cfrac(x : i64, y : i64) -> f32 { 3.0_f32.powf(x as f32) / 2.0_f32.powf(y as f32) }
				e_f = [
					cfrac( 0, 0), cfrac( 7,11),
					cfrac( 2, 3), cfrac( 9,14),
					cfrac( 4, 6),
					cfrac(11,17), cfrac( 6, 9),
					cfrac( 1, 1), cfrac( 8,12),
					cfrac( 3, 4), cfrac(10,15),
					cfrac( 5, 7)
				];
			},
			Dowlan => {
				e_f = [
					1./   1.,   33./ 31.,
					9./   8.,   33./ 28.,
					264./ 211.,
					4./   3.,   24./ 17.,
					3./   2.,   99./ 62.,
					27./  16.,   99./ 56.,
					396./ 211. 
				];
			},
			Kirnberger => {
				e_f = [
					1./  1.,   256./243.,
					9./  8.,    32./ 27.,
					5./  4., 
					4./  3.,    45./ 32.,
					3./  2.,   128./ 81.,
				  270./161.,    16./  9.,
				   15./  8. 		
				];
			},
		}
		e_f
	}
	pub fn lookup(&self, n : i8) -> f32 {
		if n < 0 {
			0.0_f32
		} else {
			self.freq_table[n as usize]
		}
	}
	pub fn retune(&mut self, a : f32) {
		self.freq_a = a;
		self.init(self.intervals);
	}

	pub fn init(&mut self, j_f : [f32; 12]) {
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
			println!("Init Interval {} = {}", i, self.fund_table[i]);
		}
	}

	pub fn init_intervals(&mut self, j_f : [f32; 12]) {
		for i in 0..12 {
			self.intervals[i] = j_f[i];
		}
	}

	pub fn note_octave(&self, i : usize) -> f32 {
		2.0_f32.powf((((i as i32 + 3) / 12) - 6) as f32)
	}

	pub fn modulate(&mut self, i : i8) {
	}

	pub fn init_just(&mut self, j_f : [f32; 12], g_flat : f32, f_sharp : f32) {
	}

	pub fn init_temp(T : Tuning) -> Tuning {
		T
	}

	pub fn nullify(i : i8) {
	}
}
