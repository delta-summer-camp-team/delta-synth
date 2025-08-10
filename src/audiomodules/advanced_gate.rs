use std::sync::Arc;

use crate::{Ordering};
use crate::audiomodules::AudioModule;
use crate::synth_state::SynthState;
const DT: f32 = 44100.0; // samples per second

enum GateState {
    Attack,
    Decay,
    Sustain,
    Release,
    Idle,
}

pub struct AdvGate {

    pub attack:  u8, //in milliseconds
    pub decay:   u8, //in milliseconds
    pub sustain: u8, // between 0 = 0.0 and 255 = 1.0
    pub release: u8, //in milliseconds

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
    fn check_press(&mut self) {
            if self.synth_state.has_key_pressed.load(Ordering::Relaxed) == true {
                    self.gate_state = GateState::Attack;
                }
        }

    fn update_envelop(&mut self){
        match self.gate_state {
            GateState::Idle    => {                             //IDLE
                self.check_press();
            }
            GateState::Attack  => {                             //ATTACK
                self.envelop += (self.attack as f32) / (1000.0*DT);

                if self.envelop >= 1.0 {
                    self.envelop = 1.0;
                    self.gate_state = GateState::Decay;
                }
                self.check_unpress();
            }
            GateState::Decay   => {                             //DECAY
                self.envelop -= (self.decay as f32) / (1000.0*DT);

                if self.envelop <= (self.sustain as f32) / 255.0 {
                    self.envelop = self.sustain as f32 / 255.0;
                    self.gate_state = GateState::Sustain;
                }
                self.check_unpress();
            }
            GateState::Sustain => {                             //SUSTAIN
                self.check_unpress();
            }
            GateState::Release => {                             //RELEASE
                self.envelop -= (self.release as f32) / (1000.0*DT);

                if self.envelop <= 0.0 {
                    self.envelop = 0.0;
                    self.gate_state = GateState::Idle;
                }
                self.check_press();
            }
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