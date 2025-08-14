use crate::{audiomodules::glide, synth_state::SynthState};
use std::sync::{atomic::Ordering, Arc};


const MAX: f32 = 0.8;
pub struct Glide {
  current_freq: f32,
  target_freq: f32,
  synthstate: Arc<SynthState>,
  sample_rate: f32,
}

impl Glide {
  pub fn new(start_freq: f32, synthstate: Arc<SynthState>, sample_rate: f32) -> Self {
    Self {
      current_freq: start_freq,
      target_freq: start_freq,
      synthstate,
      sample_rate,
    }
  }

  pub fn set_target(&mut self, freq: f32) {
    self.target_freq = freq;
  }


  pub fn next(&mut self) -> f32 {
    let glide_time = self.synthstate.glide_time.load(Ordering::Relaxed) as f32 / 270.0* MAX;
    if self.current_freq != self.target_freq {
      let step = (self.target_freq - self.current_freq) / (glide_time * self.sample_rate);
      self.current_freq += step;
    }
    self.current_freq
  }
}