use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, SupportedStreamConfig};
use std::sync::{
  Arc,
  atomic::{AtomicBool, AtomicU8, Ordering},
};

use anyhow::Result;
use midir::{MidiInput, MidiInputConnection};

mod synth_state;                           // подключаем модуль
use crate::synth_state::SynthState;        // импортируем структуру

/// Инициализация аудиоустройства и конфигурации
fn init_audio_device() -> Option<(Device, SupportedStreamConfig)> {
    let host = cpal::default_host();
    let device = host.default_output_device()?;
    let config = device.default_output_config().ok()?;
    Some((device, config))
}

pub fn init_synth_core() -> Result<(Arc<SynthState>, MidiInputConnection<()>)> {
  let synth_state = SynthState::new();

  let midi_in = MidiInput::new("midir reading input")?;
  let in_ports = midi_in.ports();

  if in_ports.is_empty() {
    anyhow::bail!("Нет доступных MIDI-портов");
  }

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
          synth_state_clone.has_key_pressed.store(true, Ordering::Relaxed);
          println!("Нажата клавиша: {}", key);
        } else if status & 0xF0 == 0x80 && message.len() >= 2 {
          synth_state_clone.has_key_pressed.store(false, Ordering::Relaxed);
          println!("Клавиша отпущена");
        }
      }
    },
    (),
  )?;

  Ok((synth_state, conn_in))
}

fn main() {
  let _ = init_audio_device();
  let (synth_state, _midi_conn) = init_synth_core()?;
  println!("SynthState готов");

  loop {
    std::thread::sleep(std::time::Duration::from_millis(500));
  }
}
