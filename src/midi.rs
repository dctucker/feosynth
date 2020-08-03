extern crate midir;

use crossbeam::channel::{Sender, Receiver};

use midir::{MidiInput,
//MidiOutput,
//MidiInputPort,
MidiInputConnection,
//MidiOutputConnection,
Ignore};

pub struct InputThread {
	connection: Option<MidiInputConnection<()>>,
	tx: Sender<Vec<u8>>,
	pub rx: Receiver<Vec<u8>>,
}
impl InputThread {
	pub fn new() -> InputThread {
		let (tx, rx) = crossbeam::channel::bounded(256);
		InputThread {
			connection: None,
			tx: tx,
			rx: rx,
		}
	}

	pub fn run(&mut self) {
		let mut input = MidiInput::new("feosynth midi input").unwrap();
		input.ignore(Ignore::None);
		let in_port = &input.ports()[0];

		let in_port_name = input.port_name(&in_port).unwrap();
		let tx = self.tx.clone();
		println!("Opening MIDI connection {}", in_port_name);
		let _conn_in = input.connect(&in_port, "feosynth-read-input", move |stamp, message, _| {
			tx.send(message.to_vec());
		}, ()).unwrap();

		self.connection = Some(_conn_in);
	}
}

/*
struct System {
	input: MidiInput,
	output: MidiOutput,
	inputs: Vec<MidiInputConnection<()>>,
	outputs: Vec<MidiOutputConnection>,
}

impl System {
	pub fn new() -> System {
		let mut input = MidiInput::new("feosynth midi input").unwrap();
		let mut sys = System {
			input: input,
			output: output,
			inputs: vec![],
			outputs: vec![],
		};
		sys.input.ignore(Ignore::None);
		sys
	}
	fn run(mut self) {
		let input = &mut self.input;
		let in_port = &input.ports()[0];

		{
			let in_port_name = input.port_name(&in_port).unwrap();
			println!("Opening MIDI connection {}", in_port_name);
		}
		let _conn_in = &input.connect(in_port, "feosynth-read-input", move |stamp, message, _| {
			println!("{}: {:?} (len = {})", stamp, message, message.len());
		}, ()).unwrap();

		//println!("Connection open, reading input from '{}' (press enter to exit) ...", in_port_name);
		std::thread::sleep(std::time::Duration::from_millis(5000));
	}

	fn list(&mut self) {
		let in_ports = self.input.ports();
		println!("\nAvailable input ports:");
		for (i, p) in in_ports.iter().enumerate() {
			println!("{}: {}", i, self.input.port_name(p).unwrap());
		}
	}
}
*/
