// main.rs

// The rotary_knob module is unchanged.
mod rotary_knob {
  use eframe::egui::{
    Label, Rect, Response, RichText, Sense, TextStyle, Ui, Vec2,
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
          let angle = delta.y.atan2(delta.x);
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

use egui::{Button, Color32, RichText, TextureHandle, Vec2};
use midir::{MidiOutput, MidiOutputConnection};

struct MyApp {
  knob1: f32,
  knob2: f32,
  slider_vals: [f32; 4],
  midi_out: Option<MidiOutputConnection>,
  midi_status: String,
  button1_pressed: bool,
  button3_pressed: bool,
  logo_texture: Option<TextureHandle>,
}

impl Default for MyApp {
  fn default() -> Self {
    Self {
      knob1: 0.0,
      knob2: 0.0,
      slider_vals: [0.0; 4],
      midi_out: None,
      midi_status: "Initializing MIDI...".to_string(),
      button1_pressed: false,
      button3_pressed: false,
      logo_texture: None,
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

  // This function is unchanged. It takes the value and scales it for MIDI.
  fn send_cc(&mut self, controller: u8, value: f32) {
    if let Some(ref mut conn) = self.midi_out {
      // Note: The value is clamped to [0.0, 1.0] here before sending.
      // A value of 2.0 will be treated as 1.0.
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
fn styled_button(ui: &mut egui::Ui, text: &str, pressed: bool) -> egui::Response {
  let fill_color = if pressed {
    Color32::from_rgb(0xb0, 0x70, 0x00) // Darker orange when pressed
  } else {
    Color32::from_rgb(0xf3, 0xa3, 0x09) // Normal orange
  };

  ui.add(
    Button::new(
      RichText::new(text)
        .heading()
        .monospace()
        .color(Color32::BLACK),
    )
      .min_size(Vec2::new(240.0, 80.0))
      .rounding(egui::Rounding::same(20.0))
      .fill(fill_color),
  )
}


impl eframe::App for MyApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // This logic for loading the logo is unchanged.
    if self.logo_texture.is_none() {
      let image_bytes = include_bytes!("../logo.png");
      let image = image::load_from_memory(image_bytes).expect("Failed to load logo image");
      let size = [image.width() as _, image.height() as _];
      let image_buffer = image.to_rgba8();
      let pixels = image_buffer.as_flat_samples();
      let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

      self.logo_texture = Some(ctx.load_texture("logo", color_image, Default::default()));
    }

    let mut cc_to_send: Vec<(u8, f32)> = Vec::new();

    ctx.set_visuals(egui::Visuals::dark());

    // Top panel with knobs is unchanged.
    egui::TopBottomPanel::top("top_panel")
      .max_height(ctx.screen_rect().height() / 2.0)
      .show(ctx, |ui| {
        ui.vertical_centered(|ui| {
          ui.add_space(5.0);
          if let Some(texture) = &self.logo_texture {
            let img = egui::Image::new(texture);
            let sized_img = img.fit_to_exact_size(egui::Vec2::new(1000.0, 200.0));
            ui.add(sized_img);
          }
          ui.label(&self.midi_status);
          ui.add_space(5.0);
        });
        ui.add_space(50.0);
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

    // Bottom panel with sliders and buttons.
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
      ui.add_space(10.0);
      ui.columns(3, |columns| {
        // Left buttons are unchanged.
        columns[0].vertical_centered(|ui| {
          if styled_button(ui, "BUTTON 1", self.button1_pressed).clicked() {
            self.button1_pressed = !self.button1_pressed;
            let value_to_send = if self.button1_pressed { 1.0 } else { 0.0 };
            cc_to_send.push((20, value_to_send));
          }
          ui.add_space(5.0);
          if styled_button(ui, "BUTTON 2", false).clicked() {
            cc_to_send.push((21, 1.0));
          }
        });

        // Center sliders - THIS IS WHERE THE CHANGE IS
        columns[1].horizontal_centered(|ui| {
          ui.add_space (75.0);
          for (i, val) in self.slider_vals.iter_mut().enumerate() {
            let slider = egui::Slider::new(val, -1.0..=1.0)
              .vertical()
              .text("");
            if ui.add_sized([192.0, 600.0], slider).changed() {
              // --- MODIFICATION START ---
              // Determine the value to send. For the first slider (i=0),
              // we add 1.0 to shift its range from [-1.0, 1.0] to [0.0, 2.0].
              // For all other sliders, we use the original value.
              let value_to_send = if i == 0 { *val + 1.0 } else { *val };

              cc_to_send.push((1 + i as u8, value_to_send));
              // --- MODIFICATION END ---
            }
          }
        });

        // Right buttons are unchanged.
        columns[2].vertical_centered(|ui| {
          if styled_button(ui, "BUTTON 3", self.button3_pressed).clicked() {
            self.button3_pressed = !self.button3_pressed;
            let value_to_send = if self.button3_pressed { 1.0 } else { 0.0 };
            cc_to_send.push((22, value_to_send));
          }
          ui.add_space(5.0);
          if styled_button(ui, "BUTTON 4", false).clicked() {
            cc_to_send.push((23, 1.0));
          }
        });
      });
      ui.add_space(10.0);
    });

    egui::CentralPanel::default().show(ctx, |_ui| {});

    // Send all collected MIDI messages.
    for (controller, value) in cc_to_send {
      self.send_cc(controller, value);
    }
  }
}
