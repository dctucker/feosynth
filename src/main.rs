include!("temperament.rs");

fn main() {
	let equal = Tuning::new(Tuning::EquaTemp);
	let other = Tuning::new(Tuning::PtolTemp);
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

	for n in 57..70 {
	}
}
