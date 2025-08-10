pub struct low_pass_filter{
    pub cutoff: f32,
    pub res_factor: f32,
    #[optional(default = 0)]
    buffer1: f32,
    #[optional(default = 0)]
    buffer2: f32,

}

impl AudioModule for low_pass_filter{

    fn y(&mut self, x:f32){
        let dt = 1/sample_rate; 
        return ((2*self.res_factor+dt*self.cutoff) * self.buffer1 + self.res_factor* y(n-2) - self.res_factor *self.cutoff*self.cutoff*dt*dt*x(n))/(self.res_factor + dt * self.cutoff + self.cutoff*self.cutoff); //discrete solution solution of a differential equation, don't ask
    }
    fn process(&mut self, output: &mut[f32]) {
        for freq in output.iter_mut(){
            self.buffer2 = freq;
            freq = y(freq);
            self.buffer1 = freq;
        }
    }
}