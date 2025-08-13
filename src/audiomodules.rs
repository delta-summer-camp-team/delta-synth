pub mod advanced_gate;
pub mod chorus;
pub mod delay;
pub mod gain;
pub mod glide;
pub mod low_pass_filter;
pub mod oscillator;
pub mod reverb;
pub mod spread;
pub trait AudioModule: Send + Sync {
  fn process(&mut self, output: &mut [f32]);
}
