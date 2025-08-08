mod gui;
mod gui_style;
mod rotary_knob;
mod styles;
mod app;

fn main() -> Result<(), eframe::Error> {
  gui::run()
}
