pub struct Volume {
    pub volume: f32
}
impl AudioModule for Volume {
    fn process(&mut self, output: &mut [f32]) {
        output *= volume;
    }
}

