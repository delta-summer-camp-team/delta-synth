use std::sync::atomic::Ordering;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicI8};



pub struct SynthState {
    pub last_key: AtomicU8,
    pub has_key_pressed: AtomicBool,
    pub nazatie_knopki: Mutex<Vec<u8>>,

    pub poli_rezim : AtomicBool,

    pub gromkost: Mutex<Vec<f32>>,

    pub waveformis: Vec<AtomicU8>,

    pub sdvig_oktov: Vec<AtomicI8>,
    pub nnno: Vec<AtomicI8>,
    pub micro_zdvig: Mutex<Vec<f32>>,

    pub delay_delay_time: AtomicU8,
    pub delay_feedback: AtomicU8,
    pub delay_mix: AtomicU8,
    pub gain_multiply_by: AtomicU8,
    pub lpf_cutoff: AtomicU8,
    pub lpf_res_factor: AtomicU8,
    pub gate_attack: AtomicU8,
    pub gate_decay: AtomicU8,
    pub gate_sustain: AtomicU8,
    pub gate_release: AtomicU8,
    pub reverb_decay_time: AtomicU8,
    pub reverb_dry_wet_mix: AtomicU8,
    pub glide_time: AtomicU8,
    pub chorus_lfo_freq: AtomicU8,
    pub chorus_base_delay_sec: AtomicU8,
    pub chorus_variation_sec: AtomicU8,
    pub chorus_feedback: AtomicU8,
    pub chorus_mix: AtomicU8,
    pub volume_volume: AtomicU8,
}

impl SynthState {
    pub fn new(kol_osc: usize) -> Arc<Self> {
        let state = Arc::new(Self {
            last_key: AtomicU8::new(0),
            has_key_pressed: AtomicBool::new(false),
            nazatie_knopki: Mutex::new(Vec::new()),
            poli_rezim: AtomicBool::new(false),

            gromkost: Mutex::new(vec![0.25; kol_osc]),
            waveformis: (0..kol_osc).map(|_| AtomicU8::new(0)).collect(),
            sdvig_oktov: (0..kol_osc).map(|_| AtomicI8::new(0)).collect(),
            nnno: (0..kol_osc).map(|_| AtomicI8::new(0)).collect(),
            micro_zdvig: Mutex::new(vec![0.0; kol_osc]),

            delay_delay_time: AtomicU8::new(32),
            delay_feedback: AtomicU8::new(38),
            delay_mix: AtomicU8::new(32),
            gain_multiply_by: AtomicU8::new(64),
            lpf_cutoff: AtomicU8::new(127),
            lpf_res_factor: AtomicU8::new(32),
            gate_attack: AtomicU8::new(3),
            gate_decay: AtomicU8::new(32),
            gate_sustain: AtomicU8::new(100),
            gate_release: AtomicU8::new(32),
            reverb_decay_time: AtomicU8::new(38),
            reverb_dry_wet_mix: AtomicU8::new(32),
            glide_time: AtomicU8::new(6),
            chorus_lfo_freq: AtomicU8::new(13),
            volume_volume: AtomicU8::new(127),
            chorus_base_delay_sec: AtomicU8::new(6),
            chorus_variation_sec: AtomicU8::new(3),
            chorus_feedback: AtomicU8::new(32),
            chorus_mix: AtomicU8::new(32),
        });


        if kol_osc > 0 { state.sdvig_oktov[0].store(0, Ordering::Relaxed); }
        if kol_osc > 1 { 
            state.sdvig_oktov[1].store(1, Ordering::Relaxed); 
            state.waveformis[1].store(1, Ordering::Relaxed);
        }
        if kol_osc > 2 {
            state.nnno[2].store(7, Ordering::Relaxed);   
            state.waveformis[2].store(2, Ordering::Relaxed); 
        }
        if kol_osc > 3 {
            state.micro_zdvig.lock().unwrap()[3] = 0.1;
            state.waveformis[3].store(3, Ordering::Relaxed);
        }

        state
    }
}