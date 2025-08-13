use crate::audiomodules::glide::Glide;
use crate::audiomodules::modulator::{modulation, Modulator};
use crate::audiomodules::AudioModule;
use crate::synth_state::SynthState;
use std::f32::consts::PI;
use std::sync::atomic::Ordering;
use std::sync::Arc;

struct mini_oscilatorsa {
  phase: f32,
  frequenchy: f32,
}

impl mini_oscilatorsa {
  fn op() -> Self {
    Self {
      phase: 0.0,
      frequenchy: 0.0,
    }
  }
}

pub struct Oscillator {
  phase: f32,
  frequency: f32,
  sample_rate: f32,
  synthstate: Arc<SynthState>,
  id: usize,
  modulator: Modulator,
  glide: Glide,
  mini_osilators: [mini_oscilatorsa; 8],
}

impl Oscillator {
  pub fn new(id: usize, frequency: f32, sample_rate: f32, synthstate: Arc<SynthState>) -> Self {
    let glide_time = synthstate.glide_time.load(Ordering::Relaxed) as f32 / 127.0 * 0.5;
    Self {
      phase: 0.0,
      frequency,
      sample_rate,
      synthstate: synthstate.clone(),
      id,
      modulator: Modulator {
        nessesary_amplitude: 1.0,
        varying: 0.0,
        amp: 2.0,
      },
      glide: Glide::new(frequency, synthstate, sample_rate),
      mini_osilators: [
        mini_oscilatorsa::op(),
        mini_oscilatorsa::op(),
        mini_oscilatorsa::op(),
        mini_oscilatorsa::op(),
        mini_oscilatorsa::op(),
        mini_oscilatorsa::op(),
        mini_oscilatorsa::op(),
        mini_oscilatorsa::op(),
      ],
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

    let poli_moda = self.synthstate.poli_rezim.load(Ordering::Relaxed);

    let nazatie_knopkii = {
      let notas = self.synthstate.nazatie_knopki.lock().unwrap();
      notas.clone()
    };

    if nazatie_knopkii.is_empty() {
      return;
    }

    if poli_moda {
      for (osc_i, nota) in nazatie_knopkii.iter().take(8).enumerate() {
        let basa_nota = *nota as f32 + sdvig_oktov * 12.0 + nnno + micro_zdvig;
        self.mini_osilators[osc_i].frequenchy = midi_note_to_freq(basa_nota);

        let phase_increment = self.mini_osilators[osc_i].frequenchy / self.sample_rate;
        for sample in output.iter_mut() {
          self.mini_osilators[osc_i].phase += phase_increment;
          if self.mini_osilators[osc_i].phase > 1.0 {
            self.mini_osilators[osc_i].phase -= 1.0;
          }

          let v = match waveforma_index {
            0 => (self.mini_osilators[osc_i].phase * 2.0 * PI).sin(),
            1 => {
              if (self.mini_osilators[osc_i].phase * 2.0 * PI).sin() > 0.0 {
                1.0
              } else {
                -1.0
              }
            },
            2 => 2.0 * self.mini_osilators[osc_i].phase - 1.0,
            3 => 4.0 * (self.mini_osilators[osc_i].phase - 0.5).abs() - 1.0,
            _ => 0.0,
          };

          *sample += v * gromkost / nazatie_knopkii.len() as f32;
        }
      }
    } else {
      let midinota = self.synthstate.last_key.load(Ordering::Relaxed);
      let basa_nota = midinota as f32 + sdvig_oktov * 12.0 + nnno + micro_zdvig;
      let frequency_for_glide = midi_note_to_freq(basa_nota);

      // let vrema_glida = self.synthstate.glide_time.load(Ordering::Relaxed) as f32 / 127.0 * 0.5;
      // self.glide.set_glide_time(vrema_glida);
      self.glide.set_target(frequency_for_glide);

      let phase_increment = self.frequency / self.sample_rate;
      for sample in output.iter_mut() {
        self.frequency = self.glide.next() + modulation(&mut self.modulator);
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
}
