enum GateState {
    Attack,
    Decay,
    Sustain,
    Release,
    Idle,            //maybe not needed
}

pub struct SynthState {

    pub attack: u8,
    pub decay: u8,
    pub sustain: u8,
    pub release: u8,

    //private
    envelop: f32,
    gate_state: GateState,
}

impl SynthState {
    pub fn updateEnvelop(&mut self, dt: f32, gate_active: bool){ //dt = delta time
        match self.gate_state {
            GateState::Attack => {
                self.envelop += dt * (1/attack)
            }
        }
        if self.envelop >= 1.0 {
                    self.envelop = 1.0;
                    self.gate_state = GateState::Decay;
                }
        //TODO: Decay, etc.
    }
}

//todo: write code
