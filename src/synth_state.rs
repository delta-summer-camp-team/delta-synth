use std::sync::atomic::{AtomicBool, AtomicU8};
use std::sync::Arc;

pub struct SynthState {
  pub last_key: AtomicU8,
  pub has_key_pressed: AtomicBool,
  pub delay_delay_time: AtomicU8,
  pub delay_feed_back: AtomicU8,
  pub delay_mix: AtomicU8,
  pub oscillator_wave_type: [AtomicU8; 4],
  pub oscillator_volume: [AtomicU8; 4],
  pub oscillator_frequency: [AtomicU8; 4],
  pub gain_multiply_by: AtomicU8,
  pub lpf_frequency: AtomicU8,
  pub gate_attack: AtomicU8,
  pub gate_decay: AtomicU8,
  pub gate_sustain: AtomicU8,
  pub gate_release: AtomicU8,
  pub reverb_decay_time: AtomicU8,
  pub reverb_dry_wet_mix: AtomicU8,
  pub glide_time: AtomicU8,
}

impl SynthState {
  pub fn new() -> Arc<Self> {
    Arc::new(Self {
      last_key: AtomicU8::new(0),
      has_key_pressed: AtomicBool::new(false),
      delay_delay_time: Default::default(),
      delay_feed_back: Default::default(),
      delay_mix: Default::default(),
      oscillator_wave_type: std::array::from_fn(|_| AtomicU8::new(0)),
      oscillator_volume: std::array::from_fn(|_| AtomicU8::new(0)),
      oscillator_frequency: std::array::from_fn(|_| AtomicU8::new(0)),
      gain_multiply_by: Default::default(),
      lpf_frequency: Default::default(),
      gate_attack: Default::default(),
      gate_decay: Default::default(),
      gate_sustain: Default::default(),
      gate_release: Default::default(),
      reverb_decay_time: Default::default(),
      reverb_dry_wet_mix: Default::default(),
      glide_time: Default::default(),
    })
  }
}
