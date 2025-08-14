use crate::audiomodules::AudioModule;
use std::f32::consts::TAU;
use crate::synth_state::SynthState;
use std::sync::Arc;

const MAX_LFO_FREQ: f32=5.0;
const MAX_VAR_SEC: f32=0.01;
const MAX_DELAY_SEC: f32=0.05;
pub struct Chorus {
  sample_rate: f32,
  // максимальная задержка в сэмплах (позволяет избежать переполнения)
  max_delay_samples: usize,
  buffer: Vec<f32>,
  write_pos: usize,
  lfo_phase: f32, // текущее значение фазы LFO

  synthstate: Arc<SynthState>,           // 0..1 (0 = только сухой, 1 = только эффект)
}

impl Chorus {
  pub fn new(sample_rate: f32, max_delay_ms: f32,  lfo_phase: f32, synthstate:Arc<SynthState>) -> Self {
    let max_delay_samples = (sample_rate * (max_delay_ms / 1000.0)).ceil() as usize;
    Self {
      sample_rate,
      lfo_phase,
      max_delay_samples,
      buffer: vec![0.0; max_delay_samples + 2], // +2 safety for interpolation
      write_pos: 0,
      synthstate,
    }
  }

  fn push_to_buffer(&mut self, x: f32) {
    self.buffer[self.write_pos] = x;
    self.write_pos += 1;
    if self.write_pos >= self.buffer.len() {
      self.write_pos = 0;
    }
  }

  fn read_fractional(&self, delay_samples: f32) -> f32 {
    // read_pos = write_pos - delay_samples (wrap)
    let buf_len = self.buffer.len() as isize;
    let write = self.write_pos as isize;
    // compute fractional index
    let read_pos = (write as f32) - delay_samples;
    // wrap to [0, buf_len)
    let mut i = read_pos.floor() as isize;
    let frac = read_pos - (i as f32);
    while i < 0 {
      i += buf_len;
    }
    while i >= buf_len {
      i -= buf_len;
    }
    let i_next = (i + 1) % (buf_len as isize);

    let a = self.buffer[i as usize];
    let b = self.buffer[i_next as usize];
    // linear interpolation
    a + frac * (b - a)
  }
}

impl AudioModule for Chorus {
  fn process(&mut self, input: &mut [f32]) {
    for sample in input.iter_mut() {

      let lfo_freq= (self.synthstate.chorus_lfo_freq.load(std::sync::atomic::Ordering::Relaxed) as f32) /127.0*MAX_LFO_FREQ;
      let base_delay_sec= (self.synthstate.chorus_base_delay_sec.load(std::sync::atomic::Ordering::Relaxed) as f32) /127.0*MAX_DELAY_SEC;
      let variation_sec= (self.synthstate.chorus_variation_sec.load(std::sync::atomic::Ordering::Relaxed) as f32) /127.0*MAX_VAR_SEC;
      let feedback= (self.synthstate.chorus_feedback.load(std::sync::atomic::Ordering::Relaxed) as f32) /127.0;
      let mix= (self.synthstate.chorus_mix.load(std::sync::atomic::Ordering::Relaxed) as f32) /127.0;
      let lfo = (self.lfo_phase).sin(); // -1..1
      let current_delay_sec = base_delay_sec + lfo * variation_sec;
      let current_delay_samples = current_delay_sec * self.sample_rate;

      // 2) read delayed sample (fractional)
      let delayed = self.read_fractional(current_delay_samples);

      // 3) output = dry*(1-mix) + wet*mix
      let wet = delayed;
      *sample = (1.0 - mix) * *sample + mix * wet;

      // 4) push input + feedback*delayed into buffer (feedback)
      let to_write = *sample + delayed * feedback;
      self.push_to_buffer(to_write);

      // 5) advance LFO
      self.lfo_phase += TAU * lfo_freq / self.sample_rate;
      if self.lfo_phase > TAU {
        self.lfo_phase -= TAU;
      }
    }
  }
}
