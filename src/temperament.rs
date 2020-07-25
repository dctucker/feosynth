enum Tuning {
	EquaTemp,
	MeanTemp,
	JustTemp,
	KeplTemp,
	PythTemp,
	HammTemp,
	PtolTemp,
	ChinTemp,
	Dowland
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
		match &self.tuning {
			EquaTemp => {
				let mut e_f = [0.0_f32; 12];
				for i in 0..12 {
					e_f[i] = 2.0_f32.powf( i as f32 / 12.0_f32 );
					println!("{}", e_f[i]);
				}
				e_f
			},
		}
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
