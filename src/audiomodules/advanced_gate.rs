use crate::synth_state::SynthState;
use crate::{audiomodules::AudioModule, Ordering};
use std::sync::{Arc, Mutex};
const SR: f32 = 44100.0; // sample rate per second


pub enum GateState {
  Attack,
  Decay,
  Sustain,
  Release,
  Idle,
}

pub struct AdvGate {
  attack: u8,  //in 20 milliseconds, 0 = 0ms, 225 = 4500ms
  decay: u8,   //in 20 milliseconds, 0 = 0ms, 225 = 4500ms
  sustain: u8, //between 0 = 0.0 and 255 = 1.0
  release: u8, //in 20 milliseconds, 0 = 0ms, 225 = 4500ms

  envelop: f32,
  gate_state: GateState,
  synth_state: Arc<SynthState>,
}

impl AdvGate {
  pub fn new(
    attack: u8,
    decay: u8,
    sustain: u8,
    release: u8,
    envelop: f32,
    gate_state: GateState,
    synth_state: Arc<SynthState>,
  ) -> Self {
    Self {
      attack,
      decay,
      sustain,
      release,
      envelop,
      gate_state,
      synth_state,
    }
  }

  pub fn get_envelop(&self) -> f32 {
    //GET ENVELOP
    return self.envelop;
  }

  fn check_unpress(&mut self) {
    if self.synth_state.has_key_pressed.load(Ordering::Relaxed) == false {
      self.gate_state = GateState::Release;
    }
  }
  fn check_press(&mut self) {
    if self.synth_state.has_key_pressed.load(Ordering::Relaxed) == true {
      self.gate_state = GateState::Attack;
    }
  }

  fn update_envelop(&mut self) {
    match self.gate_state {
      GateState::Idle => {
        //IDLE
        self.envelop = 0.0;
        self.check_press();
      },
      GateState::Attack => 'block: {
        //ATTACK
        if self.attack == 0 {
          self.gate_state = GateState::Decay;
          break 'block;
        }
        self.envelop += 1.0 / (self.attack as f32 * 0.02 * SR);

        if self.envelop >= 1.0 {
          self.envelop = 1.0;
          self.gate_state = GateState::Decay;
        }
        self.check_unpress();
      },
      GateState::Decay => 'block: {
        //DECAY
        if self.decay == 0 {
          self.gate_state = GateState::Sustain;
          break 'block;
        }
        self.envelop -= 1.0 / (self.decay as f32 * 0.02 * SR);

        if self.envelop <= (self.sustain as f32) / 255.0 {
          self.gate_state = GateState::Sustain;
        }
        self.check_unpress();
      },
      GateState::Sustain => {
        //SUSTAIN
        self.envelop = self.sustain as f32 / 255.0;
        self.check_unpress();
      },
      GateState::Release => 'block: {
        //RELEASE
        if self.release == 0 {
          self.gate_state = GateState::Idle;
          break 'block;
        }
        self.envelop -= 1.0 / (self.release as f32 * 0.02 * SR);

        if self.envelop <= 0.0 {
          self.envelop = 0.0;
          self.gate_state = GateState::Idle;
        }
        self.check_press();
      },
    }
  }
}

impl AudioModule for AdvGate {
  fn process(&mut self, output: &mut [f32]) {
      for sample in output.iter_mut() {
        self.update_envelop();
        *sample *= self.get_envelop();
      }
  }
}
