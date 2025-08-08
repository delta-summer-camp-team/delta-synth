use crate::audiomodules::AudioModule;
use crate::synth_state::SynthState;
const DT: f64 = 1.0/44100.0;

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
    pub sustain: u8,        //=1
    pub release: u8,

    envelop: f32,
    gate_state: GateState,
    
}

impl AdvGate {
    
    
    fn get_envelop(&self) -> f32 {                          //used to get envelop
        self.envelop
    }

    fn check_unpress(&mut self) {                            //checks if key is unpressed
            if SynthState.has_key_pressed.load(Ordering::Relaxed) == false {
                    self.gate_state = GateState::Release;
                }
        }

    fn update_envelop(&mut self){
        Self {
            attack: todo!(),
            decay: todo!(),
            sustain: todo!(),
            release: todo!(),
            envelop: todo!(),
            gate_state: todo!(),
        };
        match self.gate_state {
            GateState::Idle    => {                             //IDLE
                if SynthState.has_key_pressed.load(Ordering::Relaxed) == true {
                    self.gate_state = GateState::Attack;
                }
            }
            GateState::Attack  => {                             //ATTACK
                self.envelop += DT * (1.0/attack);

                if self.envelop >= 1.0 {
                    self.envelop = 1.0;
                    self.gate_state = GateState::Decay;
                }
                self.check_unpress(); //catches preemptive unpress
            }
            GateState::Decay   => {                             //DECAY
                self.envelop -= DT * (1.0/self.decay);

                if self.envelop <= sustain {
                    self.envelop = sustain;
                    self.gate_state = GateState::Sustain;
                }
                self.check_unpress();
            }
            GateState::Sustain => {                             //SUSTAIN
                self.check_unpress();
            }
            GateState::Release => {                             //RELEASE
                self.envelop -= DT * (1.0/release);

                if self.envelop <= 0 {
                    self.envelop = 0;
                    self.gate_state = GateState::Idle;
                }
            }

        }
        
    }
}


impl AudioModule for AdvGate {
    fn process(&mut self, output: &mut [f32]) {

    }
}