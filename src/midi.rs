extern crate midir;
extern crate midi;

use crossbeam::channel::{Sender, Receiver};

use midir::{MidiInput,
//MidiOutput,
//MidiInputPort,
MidiInputConnection,
//MidiOutputConnection,
Ignore};

/*
struct NoteOff {
	channel: u8,
	note: u8,
}
struct NoteOn {
	channel: u8,
	note: u8,
	vel: u8,
}
struct Control {
	channel: u8,
	num: u8,
	val: u8,
}
struct AfterTouch {
	channel: u8,
	note: u8,
	val: u8,
}
struct Patch {
	channel: u8,
	num: u8,
}
struct Pressure {
	channel: u8,
	val: u8,
}
struct Bend {
	channel: u8,
	val: u16,
}
struct SysEx {
	message: Vec<u8>,
}

pub enum MidiEvent {
	NoteOff,
	NoteOn,
	AfterTouch,
	Control,
	Patch,
	Pressure,
	Bend,
	SysEx,
}
*/
use ::midi::Message as MidiEvent;
use ::midi::Channel;
use ::midi::Manufacturer;

trait Fromu8 {
	fn from_u8(n: u8) -> Option<Channel>;
}
impl Fromu8 for Channel {
    fn from_u8(n: u8) -> Option<Self> {
        match n {
            0 => Some(Channel::Ch1),
            1 => Some(Channel::Ch2),
            2 => Some(Channel::Ch3),
            3 => Some(Channel::Ch4),
            4 => Some(Channel::Ch5),
            5 => Some(Channel::Ch6),
            6 => Some(Channel::Ch7),
            7 => Some(Channel::Ch8),
            8 => Some(Channel::Ch9),
            9 => Some(Channel::Ch10),
            10 => Some(Channel::Ch11),
            11 => Some(Channel::Ch12),
            12 => Some(Channel::Ch13),
            13 => Some(Channel::Ch14),
            14 => Some(Channel::Ch15),
            15 => Some(Channel::Ch16),
            _ => None
        }
    }
}

pub struct InputThread {
	connection: Option<MidiInputConnection<()>>,
	tx: Sender<MidiEvent>,
	pub rx: Receiver<MidiEvent>,
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
		let _conn_in = input.connect(&in_port, "feosynth-read-input", move |_stamp, message, _| {
			let event_type = message[0] & 0xf0;
			let ch = Channel::from_u8(message[0] & 0x0f).unwrap();
			let event: Option<MidiEvent> = match event_type {
				0x80 => MidiEvent::NoteOff(ch, message[1], message[2]).into(),
				0x90 => {
					let ret: MidiEvent = match message[2] {
						0 => MidiEvent::NoteOff(ch, message[1], message[2]).into(),
						_ => MidiEvent::NoteOn(ch, message[1], message[2]).into(),
					};
					ret.into()
				},
				0xa0 => MidiEvent::PolyphonicPressure(ch, message[1], message[2]).into(),
				0xb0 => MidiEvent::ControlChange(ch, message[1], message[2]).into(),
				0xc0 => MidiEvent::ProgramChange(ch, message[1]).into(),
				0xd0 => MidiEvent::ChannelPressure(ch, message[1]).into(),
				0xe0 => MidiEvent::PitchBend(ch, message[1] as u16 * 0x80 + message[2] as u16).into(),
				0xf0 => {
					let vec = message.to_vec();
					MidiEvent::SysEx(Manufacturer::OneByte(message[0]), vec).into()
				},
				_ => None,
			};
			match event {
				Some(x) => { tx.send(x); },
				None => {},
			};
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
