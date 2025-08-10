mod audiomodules;

use crate::audiomodules::oscillator::Waveforma;
use audiomodules::oscillator::Oscillator;
use audiomodules::AudioModule;
use std::sync::{atomic::Ordering, Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, SupportedStreamConfig};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let modules = build_audio_modules();
  let _module = modules[0].clone();
  let _ = init_audio_device();
  let _conn_in = init_synth_core()?;
  println!("SynthState готов");

  loop {
    std::thread::sleep(std::time::Duration::from_millis(500));
  }
}
