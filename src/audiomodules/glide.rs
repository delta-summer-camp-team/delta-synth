pub struct Glide {
  current_freq: f32,
  target_freq: f32,
  glide_time: f32,
  sample_rate: f32,
}

impl Glide {
  pub fn new(start_freq: f32, glide_time: f32, sample_rate: f32) -> Self {
    Self {
      current_freq: start_freq,
      target_freq: start_freq,
      glide_time,
      sample_rate,
    }
  }

  pub fn set_target(&mut self, freq: f32) {
    self.target_freq = freq;
  }

  pub fn set_glide_time(&mut self, time: f32) {
    self.glide_time = time;
  }

  pub fn next(&mut self) -> f32 {

    if self.glide_time <= 0.0 {
      self.current_freq = self.target_freq;
      return self.current_freq;
    }
    if self.current_freq != self.target_freq {
      let step = (self.target_freq - self.current_freq) / (self.glide_time * self.sample_rate);
      self.current_freq += step;
    }
    self.current_freq
  }
}
