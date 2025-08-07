// main.rs

mod rotary_knob {
  use eframe::egui::{
    self, Align2, Color32, Label, Rect, Response, RichText, Sense, Stroke, TextStyle, Ui, Vec2,
    Widget,
  };

  pub struct RotaryKnob<'a> {
    value: &'a mut f32,
    min: f32,
    max: f32,
    size: f32,
    label: Option<&'a str>,
    show_value: bool,
  }

  impl<'a> RotaryKnob<'a> {
    pub fn new(value: &'a mut f32, min: f32, max: f32) -> Self {
      Self {
        value,
        min,
        max,
        size: 100.0,
        label: None,
        show_value: true,
      }
    }

    pub fn with_label(mut self, label: &'a str) -> Self {
      self.label = Some(label);
      self
    }

    pub fn with_size(mut self, size: f32) -> Self {
      self.size = size;
      self
    }

    pub fn show_value(mut self, show: bool) -> Self {
      self.show_value = show;
      self
    }
  }

  impl<'a> Widget for RotaryKnob<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
      let RotaryKnob {
        value,
        min,
        max,
        size,
        label,
        show_value,
      } = self;

      let desired_size = Vec2::splat(size);
      let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::drag());
      let center = rect.center();
      let radius = size * 0.5;

      // Handle circular drag input
      if response.dragged() {
        if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
          let delta = pointer_pos - center;
          let mut angle = delta.y.atan2(delta.x);
          let t = (angle / std::f32::consts::TAU) + 0.5;
          *value = (min + t * (max - min)).clamp(min, max);
          response.mark_changed();
        }
      }

      let painter = ui.painter();
      let visuals = ui.style().interact(&response);

      // Draw knob circle
      painter.circle(center, radius - 2.0, visuals.bg_fill, visuals.fg_stroke);

      // Draw pointer
      let normalized_value = (*value - min) / (max - min);
      let angle = (normalized_value * std::f32::consts::TAU) - std::f32::consts::PI;
      let pointer = Vec2::angled(angle) * radius * 0.7;
      painter.line_segment([center, center + pointer], visuals.fg_stroke);

      // Draw value text inside the knob
      if show_value {
        let val_str = format!("{:.2}", *value);
        let font = TextStyle::Small.resolve(ui.style());
        painter.text(
          center,
          eframe::egui::Align2::CENTER_CENTER,
          val_str,
          font,
          visuals.text_color(),
        );
      }

      // Draw label below knob
      if let Some(label) = label {
        let label_pos = center + Vec2::Y * (radius + 5.0);
        let label_rect = Rect::from_center_size(label_pos, Vec2::new(size, 10.0));
        ui.put(
          label_rect,
          Label::new(RichText::new(label).text_style(TextStyle::Body)).wrap(false),
        );
      }

      response
    }
  }
}

use eframe::egui;
use rotary_knob::RotaryKnob;
use std::io::{stdin, stdout, Write};

use egui::{Button, Color32, RichText, Vec2};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

struct MyApp {
  knob1: f32,
  knob2: f32,
  slider_vals: [f32; 4],
  midi_out: Option<MidiOutputConnection>,
  midi_status: String,
}

impl Default for MyApp {
  fn default() -> Self {
    Self {
      knob1: 0.0,
      knob2: 0.0,
      slider_vals: [0.0; 4],
      midi_out: None,
      midi_status: "Initializing MIDI...".to_string(),
    }
  }
}

impl MyApp {
  // Sets up the MIDI output connection with user selection.
  fn setup_midi(&mut self) {
    let midi_out = match MidiOutput::new("egui-midi-synth") {
      Ok(m) => m,
      Err(e) => {
        self.midi_status = format!("❌ MIDI Init Error: {}", e);
        return;
      }
    };

    let out_ports = midi_out.ports();
    if out_ports.is_empty() {
      self.midi_status = "⚠️ No MIDI output ports found.".to_string();
      return;
    }

    let out_port = match out_ports.len() {
      1 => {
        println!(
          "Choosing the only available output port: {}",
          midi_out.port_name(&out_ports[0]).unwrap()
        );
        &out_ports[0]
      }
      _ => {
        println!("\nAvailable MIDI output ports:");
        for (i, p) in out_ports.iter().enumerate() {
          println!("{}: {}", i, midi_out.port_name(p).unwrap());
        }
        print!("Please select a port number: ");
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        match input.trim().parse::<usize>() {
          Ok(choice) if choice < out_ports.len() => &out_ports[choice],
          _ => {
            self.midi_status = "❌ Invalid port selection.".to_string();
            return;
          }
        }
      }
    };

    let port_name = midi_out.port_name(out_port).unwrap();
    match midi_out.connect(out_port, "egui-midi-output") {
      Ok(conn) => {
        self.midi_out = Some(conn);
        self.midi_status = format!("✅ Connected to: {}", port_name);
      }
      Err(e) => {
        self.midi_status = format!("❌ MIDI Connect Error: {}", e);
      }
    }
  }

  fn send_cc(&mut self, controller: u8, value: f32) {
    if let Some(ref mut conn) = self.midi_out {
      let midi_value = (value.clamp(0.0, 1.0) * 127.0).round() as u8;
      let _ = conn.send(&[0xB0, controller, midi_value]);
    }
  }
}

pub fn run() -> Result<(), eframe::Error> {
  let mut app = MyApp::default();
  app.setup_midi();

  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
    ..Default::default()
  };
  eframe::run_native(
    "Rust Synthesizer",
    options,
    Box::new(|_cc| Box::new(app)),
  )
}

/// A helper function to create the styled buttons consistently.
fn styled_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
  ui.add(
    Button::new(
      RichText::new(text)
        .heading()
        .monospace()
        .color(Color32::BLACK),
    )
      .min_size(Vec2::new(120.0, 40.0))
      .fill(Color32::from_rgb(0xf3, 0xa3, 0x09)),
  )
}


impl eframe::App for MyApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    let mut cc_to_send: Vec<(u8, f32)> = Vec::new();

    ctx.set_visuals(egui::Visuals::dark());

    // LAYOUT FIX: Set a maximum height for the top panel to prevent overlap.
    egui::TopBottomPanel::top("top_panel")
      .max_height(ctx.screen_rect().height() / 2.0)
      .show(ctx, |ui| {
        ui.vertical_centered(|ui| {
          ui.add_space(5.0);
          ui.label(
            RichText::new("RUST SYNTHESIZER")
              .monospace()
              .heading()
              .size(28.0)
              .color(Color32::from_rgb(255, 204, 0)),
          );
          ui.label(&self.midi_status);
          ui.add_space(5.0);
        });

        ui.columns(3, |columns| {
          columns[0].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
              .add(
                RotaryKnob::new(&mut self.knob1, 0.0, 1.0)
                  .with_label("CUTOFF")
                  .with_size(200.0)
                  .show_value(true),
              )
              .changed()
            {
              cc_to_send.push((10, self.knob1));
            }
          });

          columns[2].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            if ui
              .add(
                RotaryKnob::new(&mut self.knob2, 0.0, 1.0)
                  .with_label("RESONANCE")
                  .with_size(200.0)
                  .show_value(true),
              )
              .changed()
            {
              cc_to_send.push((11, self.knob2));
            }
          });
        });
      });

    // LAYOUT FIX: Revert to using a bottom panel for the lower controls.
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
      ui.add_space(10.0);
      ui.columns(3, |columns| {
        // Left buttons
        columns[0].vertical_centered(|ui| {
          if styled_button(ui, "BUTTON 1").clicked() {
            cc_to_send.push((20, 1.0));
          }
          ui.add_space(5.0);
          if styled_button(ui, "BUTTON 2").clicked() {
            cc_to_send.push((21, 1.0));
          }
        });

        // Center sliders
        columns[1].horizontal_centered(|ui| {
          for (i, val) in self.slider_vals.iter_mut().enumerate() {
            let slider = egui::Slider::new(val, 0.0..=1.0)
              .vertical()
              .text("");
            if ui.add_sized([48.0, 300.0], slider).changed() {
              cc_to_send.push((30 + i as u8, *val));
            }
          }
        });

        // Right buttons
        columns[2].vertical_centered(|ui| {
          if styled_button(ui, "BUTTON 3").clicked() {
            cc_to_send.push((22, 1.0));
          }
          ui.add_space(5.0);
          if styled_button(ui, "BUTTON 4").clicked() {
            cc_to_send.push((23, 1.0));
          }
        });
      });
      ui.add_space(10.0);
    });

    // The CentralPanel now just fills the space between the top and bottom panels.
    egui::CentralPanel::default().show(ctx, |_ui| {});

    for (controller, value) in cc_to_send {
      self.send_cc(controller, value);
    }
  }
}
