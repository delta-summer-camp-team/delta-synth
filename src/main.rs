mod audio_modules;
mod audiomodules;

use audio_modules::AudioModule;
use audiomodules::oscillator::Oscillator;
use std::sync::{Arc, Mutex};
use crate::audiomodules::oscillator::Waveforma;

fn main() {

  let modules = build_audio_modules();
  let module = modules[0].clone();

}


fn build_audio_modules() -> Vec<Arc<Mutex<dyn AudioModule>>> {
  let osc = Oscillator::new(440.0, 44100.0, Waveforma::Quadrat, 0.5);
  let osc1 = Oscillator::new(440.0, 44100.0, Waveforma::Sine, 0.5);
  let osc2 = Oscillator::new(440.0, 44100.0, Waveforma::Saw, 0.5);
  let osc3 = Oscillator::new(440.0, 44100.0, Waveforma::Triugolnik, 0.5);


    vec![
        Arc::new(Mutex::new(osc)), // Квадрат
        Arc::new(Mutex::new(osc1)), // син
        Arc::new(Mutex::new(osc2)),// пила
        Arc::new(Mutex::new(osc3)),// Триугольник


    ]
}