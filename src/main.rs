mod audiomodules;

use audiomodules::AudioModule;
use audiomodules::oscillator::Oscillator;
use std::sync::{Arc, Mutex, atomic::{Ordering},};

use anyhow::Result;

mod synth_state;
mod midi_service;

use crate::{audiomodules::{advanced_gate::{AdvGate, GateState}, reverb::ReverbEffect}, synth_state::SynthState};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, SupportedStreamConfig};



use cpal::{Stream, StreamConfig};
use cpal::traits::StreamTrait;



/// Инициализация аудиоустройства и конфигурации
fn init_audio_device() -> Option<(Device, SupportedStreamConfig)> {
  let host = cpal::default_host();
  let device = host.default_output_device()?;
  let config = device.default_output_config().ok()?;
  Some((device, config))
}
fn start_audio_stream(
    device: Device,
    config: StreamConfig,
    modules: Vec<Arc<Mutex<dyn AudioModule>>>,
) -> Stream {
    let stream = device
        .build_output_stream(
            &config,
            {
                let modules = modules.clone();
                move |data: &mut [f32], _| { 
                  for sample in data.iter_mut() {
                      *sample = 0.0;
                  }
                    for module in &modules {
                        if let Ok(mut m) = module.lock() {
                            m.process(data);
                        }
                    }
                }
            },
            move |_err| {
                // Ошибки можно обработать здесь (пока игнорируем)
            },
            None,
        )
        .expect("Не удалось создать аудиопоток");
    stream
}

fn build_audio_modules(synthstate: Arc<SynthState>) -> Vec<Arc<Mutex<dyn AudioModule>>> {
  let osc = Oscillator::new(0, 440.0, 44100.0,  synthstate.clone());
  let osc1 = Oscillator::new(1, 660.0, 44100.0,  synthstate.clone());
  let osc2 = Oscillator::new(2, 880.0, 44100.0,  synthstate.clone());
  let osc3 = Oscillator::new(3, 1320.0, 44100.0,  synthstate.clone());
  let gate = AdvGate::new(7.0,GateState::Idle,synthstate.clone());
  //let reverbeffect = ReverbEffect::new(0.5, 5.0, 44100);


  vec![
    Arc::new(Mutex::new(osc)), // Квадрат
    Arc::new(Mutex::new(osc1)), // син
    Arc::new(Mutex::new(osc2)), // пила
    Arc::new(Mutex::new(osc3)), // Триугольни
    Arc::new(Mutex::new(gate)),
    //Arc::new(Mutex::new(reverbeffect)),
    
  ]
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
  let synth_state = SynthState::new(4);
  let midi_con = midi_service::initiate_midi_connection(synth_state.clone());
    println!("SynthState готов");

  let modules = build_audio_modules(synth_state.clone());
     let (device, supported_config) = match init_audio_device() {
        Some(val) => val,
        None => return Ok(()), // Если не получилось, просто завершаемся молча
    };
    let config = supported_config.config();

    let stream = start_audio_stream(device, config, modules);
    stream.play().expect("Не удалось запустить поток");

    std::thread::park(); // Чтобы поток не завершился сразу
    drop(midi_con);

    Ok(())
}