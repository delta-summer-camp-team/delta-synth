enum GateState {
    Attack,
    Decay,
    Sustain,
    Release,
    Idle,
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

//todo: write code