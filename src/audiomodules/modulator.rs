use std::f32::consts::PI;
use std::sync::Arc;
use crate::synth_state::SynthState;
use crate::audiomodules::AudioModule;

pub struct Modulator {
  pub synthstate: Arc<SynthState>,
  pub varying: f32,
  pub freq: f32,
  pub step: f32
}
 
pub fn modulation(modulator: &mut Modulator) -> f32 {
  let nessesary_amplitude = modulator.synthstate.modulator_nessesary_amplitude.load(std::sync::atomic::Ordering::Relaxed) as f32 / 127.0 * 10.0;
  if modulator.varying <= 1.0 {
    modulator.varying -= 1.0
  }
  else{
  modulator.varying += modulator.step;
  }
  if modulator.nessesary_amplitude <= 0.0 {
     return 0.0;
  } 
  return modulator.nessesary_amplitude*(PI*modulator.varying*modulator.freq).sin();
} 
