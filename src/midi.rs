extern crate midir;
extern crate midi;

use crossbeam::channel::{Sender};
use midistream::*;
use midir::{MidiInput,
//MidiOutput,
//MidiInputPort,
MidiInputConnection,
//MidiOutputConnection,
Ignore};

pub struct InputThread {
	connection: Option<MidiInputConnection<()>>,
}
impl InputThread {
	pub fn new() -> InputThread {
		InputThread {
			connection: None,
		}
	}

	pub fn run(&mut self, tx: Sender<Msg>) {
		let mut input = MidiInput::new("feosynth midi input").unwrap();
		input.ignore(Ignore::None);
		let in_port = &input.ports()[0];

		let in_port_name = input.port_name(&in_port).unwrap();
		println!("Opening MIDI connection {}", in_port_name);
		let tx1 = tx.clone();
		let _conn_in = input.connect(&in_port, "feosynth-read-input", move |_stamp, message, _| {
			let messages = MsgDecoder::new(message.iter().map(|x| *x));
			for msg in messages {
				match msg {
					Ok(x) => { tx1.send(x); },
					_ => {},
				};
			}
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
