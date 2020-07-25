include!("temperament.rs");

fn main() {
	let equal = Tuning::new(Tuning::EquaTemp);
	for n in 57..70 {
		println!("Note {}: {} Hz", n, equal.lookup(n));
	}
}
