pub mod chorus;
pub mod delay;
pub mod gain;
pub mod oscillator;
pub mod advanced_gate;
pub mod low_pass_filter;

pub trait AudioModule: Send + Sync {
  fn process(&mut self, output: &mut [f32]);
}
