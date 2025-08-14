use crate::audiomodules::glide::Glide;
use crate::audiomodules::AudioModule;
use crate::synth_state::SynthState;
use std::f32::consts::PI;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct Oscillator {
  phase: f32,
  glide: Glide,
  sample_rate: f32,
  synthstate: Arc<SynthState>,
  id: usize,
}

impl Oscillator {
  pub fn new(id: usize, start_freq: f32, sample_rate: f32, synthstate: Arc<SynthState>) -> Self {
    Self {
      phase: 0.0,
      glide: Glide::new(start_freq, 0.1, sample_rate),
      sample_rate,
      synthstate,
      id,
    }
  }
}

pub fn midi_note_to_freq(note: f32) -> f32 {
  if note <= 0.0 {
    return 0.0;
  }
  return 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
}

impl AudioModule for Oscillator {
  fn process(&mut self, output: &mut [f32]) {
    let midinota = self.synthstate.last_key.load(Ordering::Relaxed);

    let gromkost = {
      let vol = self.synthstate.gromkost.lock().unwrap();
      vol[self.id]
    };

    let sdvig_oktov = self.synthstate.sdvig_oktov[self.id].load(Ordering::Relaxed) as f32;
    let nnno = self.synthstate.nnno[self.id].load(Ordering::Relaxed) as f32;

    let micro_zdvig = {
      let micros = self.synthstate.micro_zdvig.lock().unwrap();
      micros[self.id]
    };
    let waveforma_index = self.synthstate.waveformis[self.id].load(Ordering::Relaxed);

    let basa_nota = midinota as f32 + sdvig_oktov * 12.0 + nnno + micro_zdvig;

    //целевая частота
    let target_freq = midi_note_to_freq(basa_nota);
    //перед в glide
    self.glide.set_target(target_freq);
    //генер плавн частоты
    for sample in output.iter_mut() {
      let freq = self.glide.next();
      let phase_increment = freq / self.sample_rate;

      self.phase += phase_increment;
      if self.phase > 1.0 {
        self.phase -= 1.0;
      }
      let v = match waveforma_index {
        0 => (self.phase * 2.0 * PI).sin(),
        1 => {
          if (self.phase * 2.0 * PI).sin() > 0.0 {
            1.0
          } else {
            -1.0
          }
        },
        2 => 2.0 * self.phase - 1.0,
        3 => 4.0 * (self.phase - 0.5).abs() - 1.0,
        _ => 0.0,
      };
      *sample += v * gromkost;
    }
  }
}
