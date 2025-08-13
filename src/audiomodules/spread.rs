use crate::audiomodules::oscillator::Oscillator;
use crate::audiomodules::AudioModule;
use crate::synth_state::SynthState;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct Spread {
  oscillators: Vec<Oscillator>,
  voices: usize,
  synthstate: Arc<SynthState>,
}

impl Spread {
  pub fn set_spread_mode(&self, mode: bool) {
    self.synthstate.spread_mode.store(mode, Ordering::Relaxed);
  }
  pub fn new(voices: usize, sample_rate: f32, synthstate: Arc<SynthState>) -> Self {
    let mut oscillators = Vec::new();
    for i in 0..voices {
      oscillators.push(Oscillator::new(
        i,
        440.0,
        sample_rate,
        Arc::clone(&synthstate),
      ));
    }
    Self {
      oscillators,
      voices,
      synthstate,
    }
  }

  fn update_micro_shifts(&mut self) {
    let spread_enabled = self.synthstate.spread_mode.load(Ordering::Relaxed);
    let mut micros = self.synthstate.micro_zdvig.lock().unwrap();

    for osc in &self.oscillators {
      micros[osc.id()] = if spread_enabled {
        osc.id() as f32 
      } else {
        0.0
      };
    }
  }
}

impl AudioModule for Spread {
  fn process(&mut self, output: &mut [f32]) {
    self.update_micro_shifts();
    let mut temp = vec![0.0; output.len()];

    for osc in &mut self.oscillators {
      osc.process(&mut temp);
      for (o, t) in output.iter_mut().zip(temp.iter()) {
        *o += *t / self.voices as f32;
      }
      temp.fill(0.0);
    }
  }
}
