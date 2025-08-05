use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, StreamConfig};
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

fn main() {
    let sample_rate = 44100.0; // or read from config.sample_rate.0
    let next_sample = create_c_major_generator(sample_rate);


    let stream = run_output(next_sample.clone());
    std::thread::sleep(std::time::Duration::from_secs(5));
    drop(stream);
}

/*
let next_sample = {
        let frequency = 440.0;
        let sample_rate = 44100.0;
        let mut phase = 0.0;
        let phase_inc = 2.0 * std::f64::consts::PI * frequency / sample_rate;
        Arc::new(Mutex::new(move || {
            phase += phase_inc;
            if phase > 2.0 * std::f64::consts::PI {
                phase -= 2.0 * std::f64::consts::PI;
            }
            let value = phase.sin();
            (value, value)
        }))
    };
*/




fn create_c_major_generator(sample_rate: f64) -> Arc<Mutex<dyn FnMut() -> (f64, f64) + Send>> {
    let freqs = [261.6, 329.628, 391.995]; // C, E, G
    let mut phases = vec![0.0; freqs.len()];
    let phase_incs: Vec<f64> = freqs
        .iter()
        .map(|f| 2.0 * PI * f / sample_rate)
        .collect();

    Arc::new(Mutex::new(move || {
        let mut sample = 0.0;
        for (i, phase) in phases.iter_mut().enumerate() {
            *phase += phase_incs[i];
            if *phase > 2.0 * PI {
                *phase -= 2.0 * PI;
            }
            sample += (*phase).sin();
        }
        sample /= freqs.len() as f64; // normalize volume
        (sample, sample) // stereo
    }))
}

fn run_output(
    next_sample: Arc<Mutex<dyn FnMut() -> (f64, f64) + Send>>,
) -> cpal::Stream {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();
    match config.sample_format() {  
        SampleFormat::F32 => run_synth::<f32>(next_sample, device, config.into()),
        SampleFormat::I16 => run_synth::<i16>(next_sample, device, config.into()),
        SampleFormat::U16 => run_synth::<u16>(next_sample, device, config.into()),
        _ => panic!("Unsupported format"),
    }
}

fn run_synth<T>(
    next_sample: Arc<Mutex<dyn FnMut() -> (f64, f64) + Send>>,
    device: cpal::Device,
    config: StreamConfig,
) -> cpal::Stream
where
    T: SizedSample + FromSample<f64> + 'static,
{
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);

    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [T], _| {
                write_data(data, channels, &mut *next_sample.lock().unwrap());
            },
            err_fn,
            None,
        )
        .expect("failed to build output stream");

    stream.play().expect("Failed to play stream");
    println!("Audio stream playing");

    stream
}

fn write_data<T: SizedSample + FromSample<f64>>(
    output: &mut [T],
    channels: usize,
    next_sample: &mut dyn FnMut() -> (f64, f64),
) {
    for frame in output.chunks_mut(channels) {
        let (left, right) = next_sample();
        let l: T = T::from_sample(left);
        let r: T = T::from_sample(right);
        for (i, sample) in frame.iter_mut().enumerate() {
            *sample = if i % 2 == 0 { l } else { r };
        }
    }
}
