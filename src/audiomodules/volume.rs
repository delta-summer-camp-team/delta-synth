use crate::synth_state::SynthState;
use std::sync::Arc;
use crate::audiomodules::AudioModule;


pub struct Volume {
    synthstate: Arc<SynthState>,
}
impl AudioModule for Volume {
    fn process(&mut self, output: &mut [f32]) {
        let volume = self.synthstate.volume_volume.load(std::sync::atomic::Ordering::Relaxed) as f32 / 127.0;
        for sample in output.iter_mut() {
            *sample *= volume;
        }
    }
}


