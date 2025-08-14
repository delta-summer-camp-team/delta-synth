use crate::audiomodules::AudioModule;
use crate::synth_state::SynthState;
use std::sync::Arc;

pub struct LowPassFilter {
    synthstate: Arc<SynthState>,
    sample_rate: f32,

    // biquad coefficients (normalized so a0 = 1)
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,

    // DF2T state
    z1: f32, z2: f32,

    // cache to avoid recomputing every sample
    last_cutoff: f32,
    last_res_factor: f32,
}

impl LowPassFilter {
    pub fn new(synthstate: Arc<SynthState>, sample_rate: f32) -> Self {
        let mut s = Self {
            synthstate,
            sample_rate,
            b0: 0.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0,
            z1: 0.0, z2: 0.0,
            last_cutoff: f32::NAN,
            last_res_factor: f32::NAN,
        };
        s.update_coeffs();
        s
    }

    #[inline]
    fn update_coeffs(&mut self) {

        let cutoff = self.synthstate.lpf_cutoff.load(std::sync::atomic::Ordering::Relaxed) as f32 / 127.0 * (self.sample_rate / 2.0);
        let res_factor = self.synthstate.lpf_res_factor.load(std::sync::atomic::Ordering::Relaxed) as f32 / 127.0;
        let fs = self.sample_rate.max(1.0);
        let q  = res_factor;

        // keep cutoff strictly inside (0, fs/2)
        let f0 = cutoff.clamp(1e-3, 0.499 * fs);

        // Bilinear transform with pre-warping:
        // K = tan(pi * f0 / fs)
        let k = (std::f32::consts::PI * f0 / fs).tan();

        // Unnormalized RBJ-style coefficients for the LP derived from your ODE:
        let a0 = 1.0 + k / q + k * k;
        let a1 = 2.0 * (k * k - 1.0);
        let a2 = 1.0 - k / q + k * k;

        let b0 = k * k;
        let b1 = 2.0 * k * k;
        let b2 = k * k;

        // Normalize so a0 = 1
        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;

        self.last_cutoff = cutoff;
        self.last_res_factor = res_factor;
    }

    // Same signature you’re already using in main.
    #[inline]
    fn filter(&mut self, x: f32) -> f32 {
        // Recompute if parameters changed
        let cutoff = self.synthstate.lpf_cutoff.load(std::sync::atomic::Ordering::Relaxed) as f32 / 127.0 * (self.sample_rate / 2.0);
        let res_factor = self.synthstate.lpf_res_factor.load(std::sync::atomic::Ordering::Relaxed) as f32 / 127.0;
        if cutoff != self.last_cutoff || res_factor != self.last_res_factor {
            self.update_coeffs();
        }

        // Direct Form II Transposed sample processing
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x + self.z2 - self.a1 * y;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

impl AudioModule for LowPassFilter {
    fn process(&mut self, output: &mut [f32]) {
        // In-place: assumes `output` already contains the oscillator signal.
        for s in output.iter_mut() {
            *s = self.filter(*s);
        }
    }
}
