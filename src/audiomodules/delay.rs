use crate::audiomodules::{delay, AudioModule};
use crate::synth_state::SynthState;
use std::sync::Arc;


const MAX_DELAY_SEC: f32 = 1.5; 

pub struct Delay {
  buffer: Vec<f32>,
  write_pos: usize,
  sample_rate: f32,
  max_delay_samples: usize,
  synthstate: Arc<SynthState>,
}

impl Delay {
  pub fn new(sample_rate: f32, max_delay_sec: f32, synthstate: Arc<SynthState>) -> Self {
    let max_delay_samples = (sample_rate * max_delay_sec).ceil() as usize;
    // +1 to simplify interpolation at wrap
    let buffer = vec![0.0; max_delay_samples + 1];
    Self {
      buffer,
      write_pos: 0,
      sample_rate,
      max_delay_samples,
      synthstate,
    }
  }

  #[inline]
  fn wrap(&self, idx: isize) -> usize {
    let len = self.buffer.len() as isize;
    let m = ((idx % len) + len) % len;
    m as usize
  }
}

impl AudioModule for Delay {
  fn process(&mut self, input: &mut [f32]) {
    use std::sync::atomic::Ordering::Relaxed;

    // Normalized 0..1 controls from SynthState (0..127 MIDI)
    let mut t = self.synthstate.delay_delay_time.load(Relaxed) as f32 / 127.0;
    let mut fb = self.synthstate.delay_feedback.load(Relaxed)   as f32 / 127.0;
    let mut mix = self.synthstate.delay_mix.load(Relaxed)       as f32 / 127.0;

    // Clamp to sane ranges
    mix = mix.clamp(0.0, 1.0);
    fb  = fb.clamp(0.0, 0.98); // avoid runaway
    // map t to samples and ensure >= 1 sample
    let mut d_samp = (t * self.max_delay_samples as f32).max(1.0);
    // never exceed buffer
    let max_readable = (self.max_delay_samples - 1) as f32;
    if d_samp > max_readable { d_samp = max_readable; }

    for s in input.iter_mut() {
      let x = *s;

      // fractional delay read (linear interpolation)
      let d     = d_samp;
      let i0    = d.floor() as isize;
      let frac  = d - i0 as f32;

      let rp0 = self.wrap(self.write_pos as isize - i0);
      let rp1 = self.wrap(self.write_pos as isize - i0 - 1);

      let y0 = self.buffer[rp0];
      let y1 = self.buffer[rp1];
      let y  = y0 * (1.0 - frac) + y1 * frac; // delayed sample

      // dry/wet mix
      let out = x * (1.0 - mix) + y * mix;
      *s = out;

      // write input + feedback*delayed
      self.buffer[self.write_pos] = x + y * fb;

      // advance write pointer
      self.write_pos += 1;
      if self.write_pos >= self.buffer.len() {
        self.write_pos = 0;
      }
    }
  }
}
