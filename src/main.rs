mod audio_modules;
mod audiomodules;

use audio_modules::AudioModule;
use audiomodules::oscillator::Oscillator;

use std::sync::{Arc, Mutex};

fn main() {
  let modules = build_audio_modules();
  let module = modules[0].clone();

}

fn build_audio_modules() -> Vec<Arc<Mutex<dyn AudioModule>>> {
  let osc = Oscillator::new(440.0, 44100.0);

    vec![
        Arc::new(Mutex::new(osc)),
    ]
}