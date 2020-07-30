extern crate cpal;

mod audio {
	use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
	pub struct System {

		host: cpal::Host,
		device: cpal::Device,
		sample_format: cpal::SampleFormat,
		pub config: cpal::StreamConfig,
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
			}
		}
		pub fn sample_format(&self) -> cpal::SampleFormat {
			self.sample_format
		}
		pub fn run_config(&self) -> Result<(), anyhow::Error> {
			match self.sample_format {
				cpal::SampleFormat::F32 => Self::run::<f32>(&self.device, &self.config)?,
				cpal::SampleFormat::I16 => Self::run::<i16>(&self.device, &self.config)?,
				cpal::SampleFormat::U16 => Self::run::<u16>(&self.device, &self.config)?,
			}

			Ok(())
		}
		fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
		where
			T: cpal::Sample,
		{
			let sample_rate = config.sample_rate.0 as f32;
			let channels = config.channels as usize;

			// Produce a sinusoid of maximum amplitude.
			let mut sample_clock = 0f32;
			let mut next_value = move || {
				sample_clock = (sample_clock + 1.0) % sample_rate;
				(sample_clock * 440.0 * 2.0 * 3.141592 / sample_rate).sin()
			};

			let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

			let stream = device.build_output_stream(
				config,
				move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
					Self::write_data(data, channels, &mut next_value)
				},
				err_fn,
			)?;
			stream.play()?;

			std::thread::sleep(std::time::Duration::from_millis(1000));

			Ok(())
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
}
