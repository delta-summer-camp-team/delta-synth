pub struct low_pass_filter{
    pub cutoff: f32,
    pub res_factor: f32,
}

impl AudioModule for low_pass_filter{
    fn diff_equation_ddy_component(y: &f32, dy: &f32, x: &f32, cutoff: &f32, res_factor: &f32){
        let Q: f32 = 0.707;
        - cutoff/Q *dy - cutoff.pow(2) * y + cutoff.pow(2)* x
    }
}