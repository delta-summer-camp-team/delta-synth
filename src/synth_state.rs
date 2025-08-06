use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8};


pub struct SynthState {
    pub last_key: AtomicU8,
    pub has_key_pressed: AtomicBool,
}

impl SynthState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            last_key: AtomicU8::new(0),
            has_key_pressed: AtomicBool::new(false),
        })
    }
}