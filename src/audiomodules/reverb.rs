use crate::audiomodules::AudioModule;

pub struct ReverbEffect {
  dry_wet_mix: f32,
  decay_time: f32,
  pre_delay: DelayLine,
  early_reflections: EarlyReflections,
  late_reflections: LateReflections,
}

struct DelayLine {
  buffer: Vec<f32>,
  write_head: usize,
}

impl DelayLine {
  fn process_sample(&mut self, input: f32) -> f32 {
    let read_head = (self.write_head + 1) % self.buffer.len();
    let output = self.buffer[read_head];
    self.buffer[self.write_head] = input;
    self.write_head = (self.write_head + 1) % self.buffer.len();
    output
  }

  fn read_at_offset(&self, offset: usize) -> f32 {
    let len = self.buffer.len();
    self.buffer[(self.write_head + len - (offset % len)) % len]
  }
}

struct EarlyReflections {
  delay_line: DelayLine,
}

impl EarlyReflections {
  const TAPS: &'static [(usize, f32)] = &[
    (2345, 0.78), (3456, 0.65), (4567, 0.55), (5678, 0.45)
  ];

  fn process_sample(&mut self, input: f32) -> f32 {
    self.delay_line.process_sample(input);
    let mut output = 0.0;
    for (delay, gain) in Self::TAPS {
      output += self.delay_line.read_at_offset(*delay) * gain;
    }
    output
  }
}

struct CombFilter {
  delay_line: DelayLine,
  feedback: f32,
  delay_samples: usize,
}

impl CombFilter {
  fn process_sample(&mut self, input: f32) -> f32 {
    let delayed = self.delay_line.read_at_offset(self.delay_samples);
    let new_value = input + delayed * self.feedback;
    self.delay_line.process_sample(new_value);
    delayed
  }
}

struct AllPassFilter {
  delay_line: DelayLine,
  gain: f32,
  delay_samples: usize,
}

impl AllPassFilter {
  fn process_sample(&mut self, input: f32) -> f32 {
    let delayed = self.delay_line.read_at_offset(self.delay_samples);
    let output = -input * self.gain + delayed;
    let new_value_for_buffer = output * self.gain + input;
    self.delay_line.process_sample(new_value_for_buffer);
    output
  }
}

struct LateReflections {
  comb_filters: Vec<CombFilter>,
  all_pass_filters: Vec<AllPassFilter>,
}

impl LateReflections {
  fn new(decay_time: f32, sample_rate: usize) -> Self {
    let comb_delays = [1557, 1617, 1491, 1422];
    let all_pass_delays = [225, 556];

    let comb_filters = comb_delays
      .iter()
      .map(|&delay| {
        let delay_seconds = delay as f32 / sample_rate as f32;
        let feedback = (10.0_f32).powf((-3.0 * delay_seconds) / decay_time);
        CombFilter {
          delay_line: DelayLine {
            buffer: vec![0.0; delay],
            write_head: 0,
          },
          feedback,
          delay_samples: delay,
        }
      })
      .collect();

    let all_pass_filters = all_pass_delays
      .iter()
      .map(|&delay| AllPassFilter {
        delay_line: DelayLine {
          buffer: vec![0.0; delay],
          write_head: 0,
        },
        gain: 0.7,
        delay_samples: delay,
      })
      .collect();

    Self {
      comb_filters,
      all_pass_filters,
    }
  }

  fn process_sample(&mut self, input: f32) -> f32 {
    let mut comb_output = 0.0;
    for filter in &mut self.comb_filters {
      comb_output += filter.process_sample(input);
    }
    let mut final_output = comb_output;
    for filter in &mut self.all_pass_filters {
      final_output = filter.process_sample(final_output);
    }
    final_output
  }
}

impl AudioModule for ReverbEffect {
  fn process(&mut self, output: &mut [f32]) {
    for sample in output.iter_mut() {
      let dry_sample = *sample;

      let delayed_sample = self.pre_delay.process_sample(dry_sample);

      let early = self.early_reflections.process_sample(delayed_sample);
      let late = self.late_reflections.process_sample(delayed_sample);

      let wet_sample = early + late;

      *sample = (dry_sample * (1.0 - self.dry_wet_mix)) + (wet_sample * self.dry_wet_mix);
    }
  }
}

impl ReverbEffect {
  pub fn new(dry_wet_mix: f32, decay_time: f32, sample_rate: usize) -> Self {
    let pre_delay_samples = (0.05 * sample_rate as f32) as usize;
    let early_reflections_buffer_len = 6000;

    Self {
      dry_wet_mix,
      decay_time,
      pre_delay: DelayLine {
        buffer: vec![0.0; pre_delay_samples],
        write_head: 0,
      },
      early_reflections: EarlyReflections {
        delay_line: DelayLine {
          buffer: vec![0.0; early_reflections_buffer_len],
          write_head: 0,
        },
      },
      late_reflections: LateReflections::new(decay_time, sample_rate),
    }
  }
}