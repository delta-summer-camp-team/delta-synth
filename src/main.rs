mod app;
mod gui_style;
mod rotary_knob;
mod styles;

fn main() -> Result<(), eframe::Error> {
  app::run()
}