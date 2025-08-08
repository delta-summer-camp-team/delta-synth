mod audiomodules;

use audiomodules::AudioModule;
use audiomodules::oscillator::Oscillator;
use std::sync::{Arc, Mutex, atomic::{Ordering},};
use crate::audiomodules::oscillator::Waveforma;
use crate::synth_state::SynthState;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, SupportedStreamConfig};

use cpal::{Stream, StreamConfig};
use cpal::traits::StreamTrait;


use anyhow::Result;
use midir::MidiInputConnection;


mod synth_state; // подключаем модуль
use crate::synth_state::SynthState; // импортируем структуру

use crate::midi_service::midi_service;
mod midi_service;

/// Инициализация аудиоустройства и конфигурации
fn init_audio_device() -> Option<(Device, SupportedStreamConfig)> {
  let host = cpal::default_host();
  let device = host.default_output_device()?;
  let config = device.default_output_config().ok()?;
  Some((device, config))
}

fn init_synth_core() -> Result<MidiInputConnection<()>> {
    let synth_state = Arc::new(SynthState::new());
    let conn_in = midi_service(Arc::clone(&synth_state))
        .expect("Не удалось подключить MIDI"); // распаковываем Result
    Ok(conn_in)
}

fn build_audio_modules(synthstate: Arc<SynthState>) -> Vec<Arc<Mutex<dyn AudioModule>>> {
  let osc = Oscillator::new(440.0, 44100.0, Waveforma::Quadrat, 0.5, synthstate.clone());
  let osc1 = Oscillator::new(440.0, 44100.0, Waveforma::Sine, 0.5, synthstate.clone());
  let osc2 = Oscillator::new(440.0, 44100.0, Waveforma::Saw, 0.5, synthstate.clone());
  let osc3 = Oscillator::new(440.0, 44100.0, Waveforma::Triugolnik, 0.5, synthstate.clone());


  vec![
    Arc::new(Mutex::new(osc)), // Квадрат
    Arc::new(Mutex::new(osc1)), // син
    Arc::new(Mutex::new(osc2)), // пила
    Arc::new(Mutex::new(osc3)), // Триугольник


  ]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

  let (_synth_state, _midi_conn) = init_synth_core()?;
  println!("SynthState готов");
  let modules = build_audio_modules(_synth_state.clone());
  let _module = modules[2].clone();
  let _ = init_audio_device();
  
     let (device, supported_config) = match init_audio_device() {
        Some(val) => val,
        None => return Ok(()), // Если не получилось, просто завершаемся молча
    };
    let config = supported_config.config();

    let stream = start_audio_stream(device, config, modules);
    stream.play().expect("Не удалось запустить поток");

    std::thread::park(); // Чтобы поток не завершился сразу

  loop {
    std::thread::sleep(std::time::Duration::from_millis(500));
  }
}