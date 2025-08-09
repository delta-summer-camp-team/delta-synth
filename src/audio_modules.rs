
pub trait AudioModule: Send + Sync {
  fn process(&mut self, output: &mut [f32]);
}
