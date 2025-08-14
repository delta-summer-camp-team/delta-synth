use crate::synth_state::SynthState;
use std::sync::Arc;


pub struct Volume {
    synthstate: Arc<SynthState>,
}
impl AudioModule for Volume {
    fn process(synthstate: Arc<SynthState>, output: &mut [f32]) {
        let volume = synthstate.volume_volume.load(std::sync::atomic::Ordering::Relaxed) as f32 / 127.0;
        output *= volume;
    }
}

