use std::sync::Arc;

use crate::{synth_state, Ordering};
use crate::audiomodules::AudioModule;
use crate::synth_state::SynthState;
const DT: f32 = 1.0/44100.0;

enum GateState {
    Attack,
    Decay,
    Sustain,
    Release,
    Idle,
}

pub struct AdvGate {

    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,

    envelop: f32,
    gate_state: GateState,
    synth_state: Arc<SynthState>,
    
}

impl AdvGate {
    fn new (attack: u8, decay: u8, sustain: u8, release: u8, envelop: f32, gate_state: GateState, synth_state: Arc<SynthState>) -> Self {
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
    
    fn get_envelop(&self) -> f32 {                              //GET ENVELOP
        return self.envelop
    }

    fn check_unpress(&mut self) {
            if self.synth_state.has_key_pressed.load(Ordering::Relaxed) == false {
                    self.gate_state = GateState::Release;
                }
        }

    fn update_envelop(&mut self){
        match self.gate_state {
            GateState::Idle    => {                             //IDLE
                if self.synth_state.has_key_pressed.load(Ordering::Relaxed) == true {
                    self.gate_state = GateState::Attack;
                }
            }
            GateState::Attack  => {                             //ATTACK
                self.envelop += DT / self.attack as f32;

                if self.envelop >= 1.0 {
                    self.envelop = 1.0;
                    self.gate_state = GateState::Decay;
                }
                self.check_unpress();
            }
            GateState::Decay   => {                             //DECAY
                self.envelop -= DT / self.decay as f32;

                if self.envelop <= (self.sustain as f32) {
                    self.envelop = self.sustain as f32;
                    self.gate_state = GateState::Sustain;
                }
                self.check_unpress();
            }
            GateState::Sustain => {                             //SUSTAIN
                self.check_unpress();
            }
            GateState::Release => {                             //RELEASE
                self.envelop -= DT / self.release as f32;

                if self.envelop <= 0.0 {
                    self.envelop = 0.0;
                    self.gate_state = GateState::Idle;
                }
            }
        }
    }
}


impl AudioModule for AdvGate {
    fn process(&mut self, output: &mut [f32]) {

        for sample in output.iter_mut() { 
            self.update_envelop();
            *sample = self.get_envelop() // *=
         }
    }
}