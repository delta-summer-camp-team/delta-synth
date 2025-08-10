pub mod oscillator;
pub mod gain;
pub mod delay;
pub mod chorus;

pub trait AudioModule: Send + Sync {
  fn process(&mut self, output: &mut [f32]);
}
