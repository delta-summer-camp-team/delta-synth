use std::f32::consts::PI;

pub struct Modulator {
  pub nessesary_amplitude: f32, 
  pub varying: f32,
  pub freq: f32,
  pub step: f32
}
 
pub fn modulation(modulator: &mut Modulator) -> f32 {
  modulator.varying += modulator.step;
  if modulator.nessesary_amplitude <= 0.0 {
     return 0.0;
  } 
  return modulator.nessesary_amplitude*(PI*modulator.varying*modulator.freq).sin();
} 
