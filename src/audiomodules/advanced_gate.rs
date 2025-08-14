use crate::synth_state::SynthState;
use crate::{audiomodules::AudioModule, Ordering};
use std::sync::Arc;


const SR: f32 = 44100.0; // sample rate per second
const MAX: f32 = 4500.0;


pub enum GateState {
  Attack,
  Decay,
  Sustain,
  Release,
  Idle,
}

pub struct AdvGate {

  envelop: f32,
  gate_state: GateState,
  synth_state: Arc<SynthState>,
}

impl AdvGate {
  pub fn new(
    envelop: f32,
    gate_state: GateState,
    synth_state: Arc<SynthState>,
  ) -> Self {
    Self {
      envelop,
      gate_state,
      synth_state,
    }
  }

  fn get_envelop(&self) -> f32 {
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

     let decay = (self.synth_state.gate_decay.load(std::sync::atomic::Ordering::Relaxed) as f32)/127.0*MAX;
    let attack = (self.synth_state.gate_attack.load(std::sync::atomic::Ordering::Relaxed) as f32)/127.0*MAX;
    let sustain = (self.synth_state.gate_sustain.load(std::sync::atomic::Ordering::Relaxed) as f32)/127.0;
    let release = (self.synth_state.gate_release.load(std::sync::atomic::Ordering::Relaxed) as f32)/127.0*MAX;

    match self.gate_state {
      GateState::Idle => {
        //IDLE
        self.envelop = 0.0;
        self.check_press();
      },
      GateState::Attack => 'block: {
        //ATTACK
        if attack == 0.0 {
          self.gate_state = GateState::Decay;
          break 'block;
        }
        self.envelop += 1.0 / (attack as f32 * 0.02 * SR);

        if self.envelop >= 1.0 {
          self.envelop = 1.0;
          self.gate_state = GateState::Decay;
        }
        self.check_unpress();
      },
      GateState::Decay => 'block: {
        //DECAY
        if decay == 0.0 {
          self.gate_state = GateState::Sustain;
          break 'block;
        }
        self.envelop -= 1.0 / (decay  * 0.02 * SR);

        if self.envelop <= (sustain ) / 255.0 {
          self.gate_state = GateState::Sustain;
        }
        self.check_unpress();
      },
      GateState::Sustain => {
        //SUSTAIN
        self.envelop = sustain / 255.0;
        self.check_unpress();
      },
      GateState::Release => 'block: {
        //RELEASE
        if release == 0.0 {
          self.gate_state = GateState::Idle;
          break 'block;
        }
        self.envelop -= 1.0 / (release * 0.02 * SR);

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

