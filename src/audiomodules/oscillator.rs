use crate::audio_modules::AudioModule;
use std::f32::consts::PI;

pub struct Oscillator {
  pub phase: f32,
  pub frequency: f32,
  pub sample_rate: f32,
}

impl Oscillator {
  pub fn new(frequency: f32, sample_rate: f32) -> Self {
    Self {
      phase: 0.0,
      frequency,
      sample_rate,
    }
  }
}

impl AudioModule for Oscillator {
    fn process(&mut self, output: &mut [f32]) {
        let phase_increment = self.frequency / self.sample_rate;  
        for sample in output.iter_mut() { 
            self.phase += phase_increment; 
            if self.phase > 1.0 { 
                self.phase -= 1.0;
            }
            *sample = (self.phase * 2.0 * PI).sin();
            

        }
    }
}
