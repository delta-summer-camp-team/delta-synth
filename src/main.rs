mod audiomodules;

use crate::audiomodules::oscillator::Waveforma;
use audiomodules::advanced_gate::AdvGate;
use audiomodules::oscillator::Oscillator;
use audiomodules::AudioModule;
use std::sync::{atomic::Ordering, Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, SupportedStreamConfig};
use anyhow::Result;
use midir::MidiInputConnection;

mod synth_state; // подключаем модуль
use crate::synth_state::SynthState; // импортируем структуру

use crate::midi_service::initiate_midi_connection;
mod midi_service;

/// Инициализация аудиоустройства и конфигурации
fn init_audio_device() -> Option<(Device, SupportedStreamConfig)> {
  let host = cpal::default_host();
  let device = host.default_output_device()?;
  let config = device.default_output_config().ok()?;
  Some((device, config))
}

fn init_synth_core() -> Result<(MidiInputConnection<()>, Arc<Arc<SynthState>>)> {
  let synth_state = Arc::new(SynthState::new());
  let conn_in =
    initiate_midi_connection(Arc::clone(&synth_state)).expect("Не удалось подключить MIDI"); // распаковываем Result
  Ok((conn_in, synth_state))
}

fn build_audio_modules() -> Vec<Arc<Mutex<dyn AudioModule>>> {
  let osc = Oscillator::new(440.0, 44100.0, Waveforma::Quadrat, 0.5);
  let osc1 = Oscillator::new(440.0, 44100.0, Waveforma::Sine, 0.5);
  let osc2 = Oscillator::new(440.0, 44100.0, Waveforma::Saw, 0.5);

  vec![
    Arc::new(Mutex::new(osc)),  // Квадрат
    Arc::new(Mutex::new(osc1)), // син
    Arc::new(Mutex::new(osc2)), // пила
  ]
}

fn start_audio_stream(
  device: cpal::Device,
  config: cpal::StreamConfig,
  modules: Vec<Arc<Mutex<dyn AudioModule>>>,
) -> Stream {
  let data_callback = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    for module in &modules {
      if let Ok(mut m) = module.lock() {
        m.process(data);
      }
    }
  };

  let error_callback = |err| {
    eprintln!("Ошибка потока: {}", err);
  };

  device
    .build_output_stream(&config, data_callback, error_callback, None)
    .expect("Не удалось создать поток вывода")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let modules = build_audio_modules();
  let Some((device, config))  = init_audio_device() else { panic!("Init device failed") };
  let _conn_in = init_synth_core()?;
  println!("SynthState готов");
  start_audio_stream(device, config.into(), modules)
    .play()
    .expect("Could not start stream");

  loop {
    std::thread::sleep(std::time::Duration::from_millis(500));
  }
}
