use crate::audiomodules::AudioModule;
use crate::synth_state::SynthState;
use std::sync::Arc;

const MAX:f32 =3.0;
pub struct Gain {
  synthstate: Arc<SynthState>,
}

impl Gain {
  pub fn new(synthstate:Arc<SynthState>) -> Self {
    Self { synthstate, }
  }
  fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
  }
}
impl AudioModule for Gain {
  fn process(&mut self, input: &mut [f32]) {
    for sample in input.iter_mut() {
      let multiply_by=(self.synthstate.gain_multiply_by.load(std::sync::atomic::Ordering::Relaxed) as f32)/127.0*MAX;
      let amplified = *sample * multiply_by;
      *sample = 2.0 * Self::sigmoid(amplified) - 1.0;
    }
  }
}
