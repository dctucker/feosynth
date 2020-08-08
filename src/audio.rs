extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use super::types::{Generator, MidiDispatcher, SampleRated};
use crossbeam::channel::{Receiver, Sender};


pub struct System {
	device: cpal::Device,
	sample_format: cpal::SampleFormat,
	pub config: cpal::StreamConfig,
	stream: Option<cpal::Stream>,
	rx: Receiver<midistream::Msg>,
	pub tx: Sender<midistream::Msg>,
}
impl System
{
	pub fn new() -> System {
		let host = cpal::default_host();
		let device = host.default_output_device().expect("no output device available");
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
	fn run_config<G,S>(config: cpal::StreamConfig, device: cpal::Device, mut generator: Box<G>, rx: Receiver<midistream::Msg>) -> Result<cpal::Stream, anyhow::Error>
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
