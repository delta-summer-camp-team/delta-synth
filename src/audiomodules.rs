pub mod chorus;
pub mod delay;
pub mod gain;
pub mod oscillator;

pub trait AudioModule: Send + Sync {
  fn process(&mut self, output: &mut [f32]);
}
