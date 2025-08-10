#[derive(Default)]

pub struct LowPassFilter{
    pub cutoff: f32,
    pub res_factor: f32,
    sample_rate: u32,
    buffer1: f32, //do not need to be specified when initing it
    buffer2: f32,

}
impl LowPassFilter{
    pub fn new(cutoff: f32, res_factor: f32, sample_rate: f32) -> Self {
    Self { cutoff, res_factor, sample_rate, y1: 0.0, y2: 0.0 }
    }
    
    fn filter(&mut self, x:f32) -> f32{
        let dt:f32 = 1.0/self.sample_rate; 
        let Q_min = 0.707; 
        let Q_max = 1.0;  
        let Q:f32 = Q_min + self.res_factor.clamp(0.0, 1.0) * (Q_max - Q_min);
        let y:f32 =((2*Q+dt*self.cutoff) * self.buffer1 + Q * self.buffer2 - Q * self.cutoff*self.cutoff*dt*dt*x)/(Q + dt * self.cutoff + self.cutoff*self.cutoff); //discrete solution solution of a differential equation, don't ask
        //shift state
        self.buffer2 = self.buffer1;
        self.buffer1 = y;
        return y
    }
}
impl AudioModule for LowPassFilter{
    fn process(&mut self, output: &mut[f32]) {
        for sample in output.iter_mut(){
            *sample = self.filter(*sample);
        }
    }
}