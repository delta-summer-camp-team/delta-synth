use std::sync::atomic::Ordering;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicI8};



pub struct SynthState {
    pub last_key: AtomicU8,
    pub has_key_pressed: AtomicBool,

    pub gromkost: Mutex<Vec<f32>>,

    pub waveformis: Vec<AtomicU8>,

    pub sdvig_oktov: Vec<AtomicI8>,
    pub nnno: Vec<AtomicI8>,
    pub micro_zdvig: Mutex<Vec<f32>>,

}

impl SynthState {
    pub fn new(kol_osc: usize) -> Arc<Self> {
        let state = Arc::new(Self {
            last_key: AtomicU8::new(0),
            has_key_pressed: AtomicBool::new(false),

            gromkost: Mutex::new(vec![0.25; kol_osc]),
            waveformis: (0..kol_osc).map(|_| AtomicU8::new(0)).collect(),
            sdvig_oktov: (0..kol_osc).map(|_| AtomicI8::new(0)).collect(),
            nnno: (0..kol_osc).map(|_| AtomicI8::new(0)).collect(),
            micro_zdvig: Mutex::new(vec![0.0; kol_osc]),
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