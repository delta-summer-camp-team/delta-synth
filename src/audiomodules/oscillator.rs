use crate::audio_modules::AudioModule;
use std::f32::consts::PI;

pub enum Waveforma {
  Sine,
  Quadrat,
  Saw,
  Triugolnik,
}


pub struct Oscillator {
  pub phase: f32,
  pub frequency: f32,
  pub sample_rate: f32,
  pub waveforma: Waveforma,
  pub amplituda: f32
}

impl Oscillator {
  pub fn new(frequency: f32, sample_rate: f32, waveforma:Waveforma, amplituda:f32) -> Self {
    Self {
      phase: 0.0,
      frequency,
      sample_rate,
      waveforma,
      amplituda
    }
  }
}


pub fn midi_note_to_freq(note: u8) -> f32 {
  let nota = note as f32;
  440.0 * 2.0_f32.powf(nota/12.0) 
}

impl AudioModule for Oscillator {
    fn process(&mut self, output: &mut [f32]) {
        let phase_increment = self.frequency / self.sample_rate;
        for sample in output.iter_mut() { 
            self.phase += phase_increment; 
            if self.phase > 1.0 { 
                self.phase -= 1.0;
            }
            let v = match self.waveforma {
                Waveforma::Sine => (self.phase * 2.0 * PI).sin(),          
                Waveforma::Quadrat => if (self.phase * 2.0 * PI).sin() > 0.0 {
                  1.0
                }else{
                 -1.0
                }
                Waveforma::Saw => 2.0 * self.phase - 1.0,
                Waveforma::Triugolnik => 4.0 * (self.phase - 0.5).abs() - 1.0
                };
            *sample = v * self.amplituda;
            }

        }
    }