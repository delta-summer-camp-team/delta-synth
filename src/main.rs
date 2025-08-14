mod audiomodules;

use audiomodules::AudioModule;
use audiomodules::oscillator::Oscillator;
use audiomodules::low_pass_filter::LowPassFilter;
use std::sync::{Arc, Mutex, atomic::{Ordering},};

use anyhow::Result;
use midir::{MidiInput, MidiInputConnection};

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
pub fn init_synth_core() -> Result<(Arc<SynthState>, MidiInputConnection<()>)> {
  let synth_state = SynthState::new(4);

  let midi_in = MidiInput::new("midir reading input")?;
  let in_ports = midi_in.ports();

  let in_port = &in_ports[0];
  println!("Подключение к порту: {}", midi_in.port_name(in_port)?);

  let synth_state_clone = Arc::clone(&synth_state);
  let conn_in = midi_in.connect(
    in_port,
    "midir-read-input",
    move |_, message, _| {
      if let Some(&status) = message.first() {
        if status & 0xF0 == 0x90 && message.len() >= 2 {
          let key = message[1];
          synth_state_clone.last_key.store(key, Ordering::Relaxed);
          synth_state_clone
            .has_key_pressed
            .store(true, Ordering::Relaxed);
          println!("Нажата клавиша: {}", key);
        } else if status & 0xF0 == 0x80 && message.len() >= 2 {
          synth_state_clone
            .has_key_pressed
            .store(false, Ordering::Relaxed);
          println!("Клавиша отпущена");
        }
      }
    },
    (),
  )?;

  Ok((synth_state, conn_in))
}

fn build_audio_modules(synthstate: Arc<SynthState>) -> Vec<Arc<Mutex<dyn AudioModule>>> {
  let osc = Oscillator::new(0, 440.0, 44100.0,  synthstate.clone());
  let osc1 = Oscillator::new(1, 660.0, 44100.0,  synthstate.clone());
  let osc2 = Oscillator::new(2, 880.0, 44100.0,  synthstate.clone());
  let osc3 = Oscillator::new(3, 1320.0, 44100.0,  synthstate.clone());
  //let reverbeffect = ReverbEffect
  let lpfgate = AdvGate::new(7,7,255,7,1.0,GateState::Idle,synthstate.clone());
  let lpf: LowPassFilter = LowPassFilter::new(1760.0 , 0.707 , 44100.0, lpfgate);
  let gate = AdvGate::new(7,7,255,7,1.0,GateState::Idle,synthstate.clone());

  vec![
    Arc::new(Mutex::new(osc)), // Квадрат
    Arc::new(Mutex::new(osc1)), // син
    Arc::new(Mutex::new(osc2)), // пила
    Arc::new(Mutex::new(osc3)), // Триугольни
    //Arc::new(Mutex::new(reverbeffect)),
    Arc::new(Mutex::new(lpf)), // low pass filter
    Arc::new(Mutex::new(gate)), // normal gate

  ]
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

  let (_synth_state, _midi_conn) = init_synth_core()?;
  println!("SynthState готов");
  let modules = build_audio_modules(_synth_state.clone());
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