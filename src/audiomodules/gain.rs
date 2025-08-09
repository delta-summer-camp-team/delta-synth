use crate::audiomodules::AudioModule;

pub struct Gain{
    myltiply_by: f32,
}
impl Gain{
    pub fn new(myltiply_by: f32) -> Self{
        Self{
            myltiply_by,
        }        
    }
    fn sigmoid(x: f32) -> f32 {
        1.0 / (1.0 + (-x).exp())
    }
}
impl AudioModule for Delay {
    fn process(&mut self, input: &mut [f32]) {
        for sample in input.iter_mut(){
            let amplified = *sample * self.myltiply_by;
            *sample = 2.0 * Self::sigmoid(amplified) - 1.0;            
        }
    }
}