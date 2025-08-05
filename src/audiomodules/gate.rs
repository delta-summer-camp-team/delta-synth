use crate::audio_modules::AudioModule;
use std::sync::{Arc, Mutex};

pub struct Gate {
  synth_state: Arc<Mutex<SynthState>>,
}

impl Gate {
  pub fn new(synth_state: Arc<Mutex<SynthState>>) -> Self {
    Self { synth_state }
  }
}

impl AudioModule for Gate {
  fn process(&mut self, buffer: &mut [f32]) {
    let state = self.synth_state.lock().unwrap();
    if state.pressed_keys.is_empty() {
      for sample in buffer.iter_mut() {
        *sample = 0.0;
      }
    }
  }
}
