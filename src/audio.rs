extern crate cpal;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use super::types::{SampleRate};

pub struct System {
	host: cpal::Host,
	device: cpal::Device,
	sample_format: cpal::SampleFormat,
	pub config: cpal::StreamConfig,
	stream: Option<cpal::Stream>,
}
impl System {
	pub fn new() -> System {
		let host = cpal::default_host();
		let device = host.default_output_device().expect("no output device available");
		let config = device.default_output_config().expect("no default config available");

		System {
			host: host,
			device: device,
			sample_format: config.sample_format(),
			config: config.into(),
			stream: None,
		}
	}
	pub fn sample_format(&self) -> cpal::SampleFormat {
		self.sample_format
	}
	pub fn run(&mut self) -> Result<(), anyhow::Error> {
		let output_stream = match self.sample_format {
			cpal::SampleFormat::F32 => self.run_config::<f32>(),
			cpal::SampleFormat::I16 => self.run_config::<i16>(),
			cpal::SampleFormat::U16 => self.run_config::<u16>(),
		}.unwrap();
		output_stream.play().unwrap();
		self.stream = Some(output_stream);

		Ok(())
	}
	fn run_config<T>(&mut self) -> Result<cpal::Stream, anyhow::Error>
	where
		T: cpal::Sample,
	{
		// Produce a sinusoid of maximum amplitude.
		let mut sample_clock = 0f32;
		let channels = self.config.channels as usize;
		let sample_rate = self.config.sample_rate.0 as f32;
		let mut next_value = move || {
			sample_clock = (sample_clock + 1.0) % sample_rate;
			(0.2 - 0.00001 * sample_clock).max(0.0) * (sample_clock * 440.0 * 2.0 * 3.141592 / sample_rate).sin()
		};
		let stream = self.device.build_output_stream(
			&self.config,
			move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
				Self::write_data(data, channels, &mut next_value)
			},
			|err| eprintln!("an error occurred on stream: {}", err),
		)?;

		//std::thread::sleep(std::time::Duration::from_millis(1000));
		Ok(stream)
	}
	fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
	where
		T: cpal::Sample,
	{
		for frame in output.chunks_mut(channels) {
			let value: T = cpal::Sample::from::<f32>(&next_sample());
			for sample in frame.iter_mut() {
				*sample = value;
			}
		}
	}
}

pub trait SampleRated {
	fn set_sample_rate(&mut self, sample_rate: SampleRate);
}
