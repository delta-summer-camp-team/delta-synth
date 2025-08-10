mod app;
mod gui_style;
mod keyboard;
mod rotary_knob;
mod styles;
mod doom_mode;
mod midi_output; // renamed module, file = src/midi_output.rs

use std::thread;

fn main() -> Result<(), eframe::Error> {
  // Spawn a thread so the MIDI code doesn't block the GUI event loop.
  thread::spawn(|| {
    if let Err(e) = run_midi() {
      eprintln!("MIDI thread error: {}", e);
    }
  });

  // Start your eframe app (keeps running until GUI exits)
  app::run()
}

fn run_midi() -> Result<(), Box<dyn std::error::Error>> {
  let mut conn_out = midi_output::open_midi_output()?;

  println!("Sending a test note...");

  // Note On
  conn_out.send(&[0x90, 60, 64])?;
  std::thread::sleep(std::time::Duration::from_millis(500));
  // Note Off
  conn_out.send(&[0x80, 60, 64])?;

  println!("Note sent! Press Enter to exit MIDI thread.");
  let mut exit_input = String::new();
  std::io::stdin().read_line(&mut exit_input)?; // keeps the thread alive until Enter

  Ok(())
}