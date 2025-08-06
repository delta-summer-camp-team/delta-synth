mod gui;
mod midi_service;

use std::thread;
use std::time::Duration;

// This is a placeholder for your MIDI service.
// In a real application, this would contain your loop for handling MIDI messages.
fn midi_service() {
  loop {
    println!("MIDI service is running in the background...");
    // Add a small delay to prevent this loop from consuming 100% CPU.
    thread::sleep(Duration::from_secs(2));
  }
}

fn main() -> Result<(), eframe::Error> {
  // Spawn the midi_service in a new thread.
  // The `move` keyword gives the new thread ownership of any variables it uses.
  // The main thread will not wait for this to finish.
  thread::spawn(move || {
    midi_service();
  });

  // Run the GUI on the main thread. This function will block the main thread
  // until the application window is closed.
  gui::run()
}