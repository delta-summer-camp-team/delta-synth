mod gui;
mod rotary_knob;

// The main function now simply calls the public `run` function from the `gui` module.
fn main() -> Result<(), eframe::Error> {gui::run()
}