extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use super::types::{Generator, MidiDispatcher, SampleRated};
use crossbeam::channel::{Receiver, Sender};
use crate::midi::Msg;

type Device = cpal::Device;
type SampleFormat = cpal::SampleFormat;
type Stream = cpal::Stream;
type StreamConfig = cpal::StreamConfig;

pub struct System {
	device: Device,
	sample_format: SampleFormat,
	pub config: StreamConfig,
	stream: Option<Stream>,
	rx: Receiver<Msg>,
	pub tx: Sender<Msg>,
}
impl System {
	pub fn new() -> System {
		//let host;
		let device;
		/*
		if cfg!(target_os = "windows") {
			host = match cpal::host_from_id(cpal::HostId::Asio) {
				Ok(h) => { h },
				_ => { cpal::default_host() },
			};
		} else {
			host = cpal::default_host();
		}
		device = host.default_output_device().expect("no output device available");
		*/
		device = cpal::default_host().default_output_device().expect("no output device available");
		/*
		device = match host.default_output_device() {
			Some(d) => { d },
			_ => {
				let host = cpal::default_host();
				host.default_output_device().expect("no output device available")
			},
		};
		*/

		let config = device.default_output_config().expect("no default config available");
		let (tx, rx) = crossbeam::channel::bounded(256);

		System {
			device: device,
			sample_format: config.sample_format(),
			config: config.into(),
			stream: None,
			rx: rx,
			tx: tx,
		}
	}
	pub fn sample_format(&self) -> cpal::SampleFormat {
		self.sample_format
	}
	fn run_config<G,S>(config: StreamConfig, device: Device, mut generator: Box<G>, rx: Receiver<midistream::Msg>) -> Result<Stream, anyhow::Error>
	where
		G: Generator + SampleRated + MidiDispatcher + Send + Sync + 'static,
		S: cpal::Sample,
	{
		//use rand::Rng;
		let channels = config.channels as usize;
		let sample_rate = config.sample_rate.0;
		generator.set_sample_rate(sample_rate);

		let stream = device.build_output_stream(
			&config,
			move |output: &mut [S], _: &cpal::OutputCallbackInfo| {
				//let mut rng = rand::thread_rng();
				//Self::write_data(data, channels, &mut next_value)
				for frame in output.chunks_mut(channels) {
					let next_sample = generator.generate()[0];
					//let next_sample = rng.gen();
					let value: S = cpal::Sample::from::<f32>(&next_sample);
					for sample in frame.iter_mut() {
						*sample = value;
					}
				}
				while let Some(msg) = rx.try_recv() {
					generator.dispatch_midi_in(&msg);
				}
			},
			|err| eprintln!("an error occurred on stream: {}", err),
		)?;

		Ok(stream)
	}
	pub fn run<G>(mut self, generator: Box<G>) -> Result<cpal::Stream, anyhow::Error>
	where G: Generator + SampleRated + MidiDispatcher + Send + Sync + 'static
	{
		let output_stream = match self.sample_format {
			cpal::SampleFormat::F32 => Self::run_config::<G,f32>(self.config, self.device, generator, self.rx)?,
			cpal::SampleFormat::I16 => Self::run_config::<G,i16>(self.config, self.device, generator, self.rx)?,
			cpal::SampleFormat::U16 => Self::run_config::<G,u16>(self.config, self.device, generator, self.rx)?,
		};
		output_stream.play()?;
		self.stream = Some(output_stream);

		Ok(self.stream.unwrap())
	}
}
