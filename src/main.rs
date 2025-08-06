mod gui;
mod rotary_knob;

// The main function now simply calls the public `run` function from the `gui` module.
fn main() -> Result<(), eframe::Error> {
  // This executes the GUI code located in `gui.rs`.
  gui::run()
}
