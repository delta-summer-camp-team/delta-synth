use crate::audiomodules::{delay, AudioModule};
use crate::synth_state::SynthState;
use std::sync::Arc;

const MAX_DELAY_SEC: f32 = 1.5; 

pub struct Delay {
  buffer: Vec<f32>,
  write_pos: usize,
  sample_rate: f32,

  synthstate: Arc<SynthState>,
}

impl Delay {
  pub fn new(sample_rate: f32, max_delay_sec: f32, synthstate:Arc<SynthState>) -> Self {
    let buffer_len = (sample_rate * max_delay_sec).ceil() as usize + 1;
    Self {
      buffer: vec![0.0; buffer_len],
      write_pos: 0,
      sample_rate,
      synthstate,
    }
  }
}


impl AudioModule for Delay {
  fn process(&mut self, input: &mut [f32]) {
    let mut delay_time = (self.synthstate.delay_delay_time.load(std::sync::atomic::Ordering::Relaxed) as f32)/127.0*MAX_DELAY_SEC;
    let feedback = (self.synthstate.delay_feedback.load(std::sync::atomic::Ordering::Relaxed) as f32) /127.0;
    let mix = (self.synthstate.delay_mix.load(std::sync::atomic::Ordering::Relaxed) as f32)/127.0;
    
    if delay_time==0.0{
      delay_time=0.01;
    }

    let delayed_sample = delay_time * self.sample_rate;
    let buffer_len = self.buffer.len();

    for sample in input.iter_mut() {
      let input_sample = *sample;

      // Применяем микс
      let output = input_sample * (1.0 - mix) + delayed_sample * mix;

      // Записываем обратно в input (in-place обработка)
      *sample = output;

      // Обновляем буфер с учётом обратной связи
      self.buffer[self.write_pos] = input_sample + delayed_sample * feedback;

      // Продвигаем позицию записи
      self.write_pos = (self.write_pos + 1) % buffer_len;
    }
  }
}

