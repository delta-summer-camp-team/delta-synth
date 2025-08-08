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

    //private
    envelop: f32,
    gate_state: GateState,
}

impl AdvGate {
    pub fn updateEnvelop(&mut self, dt: f32){ //dt = delta time

        fn checkUnpress(){ //checks if key is unpressed
            if SynthState.has_key_pressed.load(Ordering::Relaxed) = false {
                    self.gate_state = GateState::Release;
                }
        }

        match self.gate_state {
            GateState::Idle => {
                if SynthState.has_key_pressed.load(Ordering::Relaxed) = true {
                    self.gate_state = GateState::Attack;
                }
            }
            GateState::Attack => {                              //ATTACK
                self.envelop += dt * (1.0/attack);

                if self.envelop >= 1.0 {
                    self.envelop = 1.0;
                    self.gate_state = GateState::Decay;
                }
                checkUnpress(); //catches preemptive unpress
            }
            GateState::Decay => {                               //DECAY
                self.envelop -= dt * (1.0/decay);

                if self.envelop <= sustain {
                    self.envelop = sustain;
                    self.gate_state = GateState::Sustain;
                }
                checkUnpress();
            }
            GateState::Sustain => {                             //SUSTAIN
                checkUnpress()
            }
            GateState::Release => {                             //RELEASE
                self.envelop -= dt * (1.0/release);

                if self.envelop <= 0 {
                    self.envelop = 0;
                    self.gate_state = GateState::Idle;
                }
            }

        }
        
    }
}
