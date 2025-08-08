use crate::gui_style::GUIStyle;
use crate::keyboard::Keyboard;
use eframe::egui;
use eframe::egui::{
  Button, Color32, Label, Rect, Response, RichText, Sense, TextStyle, TextureHandle,
  Ui, Vec2, Widget,
};
use midir::{MidiInput, MidiOutput, MidiOutputConnection};
use std::sync::mpsc::{channel, Receiver, Sender};

// --- AppState Enum ---
// Manages the current view of the application.
#[derive(Clone, Copy, Debug, PartialEq)]
enum AppState {
  StartScreen,
  MainApp,
  Diagnostics,
}

mod rotary_knob {
  use super::*;

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

      painter.circle(center, radius - 2.0, visuals.bg_fill, visuals.fg_stroke);

      let normalized_value = (*value - min) / (max - min);
      let angle = (normalized_value * std::f32::consts::TAU) - std::f32::consts::PI;
      let pointer = Vec2::angled(angle) * radius * 0.7;
      painter.line_segment([center, center + pointer], visuals.fg_stroke);

      if show_value {
        let val_str = format!("{:.2}", *value);
        let font = TextStyle::Small.resolve(ui.style());
        painter.text(
          center,
          egui::Align2::CENTER_CENTER,
          val_str,
          font,
          visuals.text_color(),
        );
      }

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

use rotary_knob::RotaryKnob;

pub struct MyApp {
  knob1: f32,
  knob2: f32,
  slider_vals: [f32; 4],
  midi_out: Option<MidiOutputConnection>,
  midi_in: Option<midir::MidiInputConnection<()>>,
  midi_status: String,
  button1_pressed: bool,
  button3_pressed: bool,
  logo_texture: Option<TextureHandle>,
  current_style: GUIStyle,
  app_state: AppState, // New state field
  is_fullscreen: bool,
  keyboard: Keyboard,
  midi_rx: Receiver<Vec<u8>>,
  midi_tx: Sender<Vec<u8>>,
}

impl Default for MyApp {
  fn default() -> Self {
    let (midi_tx, midi_rx) = channel();
    Self {
      knob1: 0.0,
      knob2: 0.0,
      slider_vals: [0.0; 4],
      midi_out: None,
      midi_in: None,
      midi_status: "Initializing MIDI...".to_string(),
      button1_pressed: false,
      button3_pressed: false,
      logo_texture: None,
      current_style: GUIStyle::DarkMode, // Default style
      app_state: AppState::StartScreen,  // Start with the selection screen
      is_fullscreen: true,
      keyboard: Keyboard::new(),
      midi_rx,
      midi_tx,
    }
  }
}

impl MyApp {
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
    } else {
      let out_port = &out_ports[0];
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

    let midi_in = MidiInput::new("egui-midi-synth-in").unwrap();
    let in_ports = midi_in.ports();
    if in_ports.is_empty() {
      self.midi_status = format!("{} No MIDI input ports found.", self.midi_status);
    } else {
      let in_port = &in_ports[0];
      let port_name = midi_in.port_name(in_port).unwrap();
      let tx = self.midi_tx.clone();
      let conn_in = midi_in
        .connect(
          in_port,
          "egui-midi-input",
          move |_, message, _| {
            tx.send(message.to_vec()).unwrap();
          },
          (),
        )
        .unwrap();
      self.midi_in = Some(conn_in);
      self.midi_status = format!("{} ✅ Connected to MIDI In: {}", self.midi_status, port_name);
    }
  }

  fn send_cc(&mut self, controller: u8, value: f32) {
    if let Some(ref mut conn) = self.midi_out {
      let midi_value = (value.clamp(0.0, 1.0) * 127.0).round() as u8;
      let _ = conn.send(&[0xB0, controller, midi_value]);
    }
  }

  // --- UI drawing functions for different states ---

  fn draw_start_screen(&mut self, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.2);
        ui.add_space(50.0);
        ui.heading("Choose a Style");
        ui.add_space(20.0);

        if ui
          .add(
            Button::new("Orange Mode")
              .min_size(Vec2::new(200.0, 50.0))
              .rounding(10.0),
          )
          .clicked()
        {
          self.current_style = GUIStyle::OrangeMode;
          self.app_state = AppState::MainApp;
        }
        ui.add_space(10.0);
        if ui
          .add(
            Button::new("Dark Mode")
              .min_size(Vec2::new(200.0, 50.0))
              .rounding(10.0),
          )
          .clicked()
        {
          self.current_style = GUIStyle::DarkMode;
          self.app_state = AppState::MainApp;
        }
        ui.add_space(10.0);
        if ui
          .add(
            Button::new("Turquoise Mode")
              .min_size(Vec2::new(200.0, 50.0))
              .rounding(10.0),
          )
          .clicked()
        {
          self.current_style = GUIStyle::TurquoiseMode;
          self.app_state = AppState::MainApp;
        }
        ui.add_space(10.0);
        if ui
          .add(
            Button::new("Diagnostics")
              .min_size(Vec2::new(200.0, 50.0))
              .rounding(10.0),
          )
          .clicked()
        {
          self.app_state = AppState::Diagnostics;
        }
      });
    });
  }

  fn draw_main_app(&mut self, ctx: &egui::Context) {
    let mut cc_to_send: Vec<(u8, f32)> = Vec::new();

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
      ui.horizontal(|ui| {
        if ui.button("↩ Back").clicked() {
          self.app_state = AppState::StartScreen;
        }

        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
          if ui.button("Toggle Fullscreen").clicked() {
            self.is_fullscreen = !self.is_fullscreen;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
              self.is_fullscreen,
            ));
          }
          if let Some(texture) = &self.logo_texture {
            let img = egui::Image::new(texture);
            let sized_img = img.fit_to_exact_size(Vec2::new(1000.0, 200.0));
            ui.add(sized_img);
          }
          ui.label(&self.midi_status);
        });
      });
      ui.add_space(50.0);
      ui.columns(3, |columns| {
        columns[0].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
          if ui.add(
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
          if ui.add(
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


    for (controller, value) in cc_to_send {
      self.send_cc(controller, value);
    }
  }

  fn draw_diagnostics_screen(&mut self, ctx: &egui::Context) {
    let mut cc_to_send: Vec<(u8, f32)> = Vec::new();

    egui::TopBottomPanel::top("top_panel")
      .max_height(ctx.screen_rect().height() / 2.0)
      .show(ctx, |ui| {
        ui.vertical_centered(|ui| {
          ui.add_space(5.0);
          if ui.button("↩ Back to Style Selection").clicked() {
            self.app_state = AppState::StartScreen;
          }
          if ui.button("Toggle Fullscreen").clicked() {
            self.is_fullscreen = !self.is_fullscreen;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
              self.is_fullscreen,
            ));
          }
          ui.add_space(10.0);

          // Add the keyboard here
          self.keyboard.ui(ui);

          ui.label(&self.midi_status);
          ui.add_space(5.0);
        });
        ui.add_space(50.0);
        ui.columns(3, |columns| {
          columns[0].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(
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
            if ui.add(
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

    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
      ui.add_space(10.0);
      ui.columns(3, |columns| {
        columns[0].vertical_centered(|ui| {
          if self.styled_button(ui, "BUTTON 1", self.button1_pressed).clicked() {
            self.button1_pressed = !self.button1_pressed;
            let value_to_send = if self.button1_pressed { 1.0 } else { 0.0 };
            cc_to_send.push((20, value_to_send));
          }
          ui.add_space(5.0);
          if self.styled_button(ui, "BUTTON 2", false).clicked() {
            cc_to_send.push((21, 1.0));
          }
        });

        columns[1].vertical_centered(|ui| {
          egui::ScrollArea::horizontal()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
              ui.horizontal(|ui| {
                for (i, val) in self.slider_vals.iter_mut().enumerate() {
                  ui.vertical(|ui| {
                    ui.label(format!("S{}", i + 1));
                    let slider = egui::Slider::new(val, -0.5..=0.5)
                      .vertical()
                      .text("");
                    if ui.add_sized([200.0, 1000.0], slider).changed() {
                      cc_to_send.push((1 + i as u8, *val + 0.5));
                    }
                  });
                  ui.add_space(15.0);
                }
              });
            });
        });

        columns[2].vertical_centered(|ui| {
          if self.styled_button(ui, "BUTTON 3", self.button3_pressed).clicked() {
            self.button3_pressed = !self.button3_pressed;
            let value_to_send = if self.button3_pressed { 1.0 } else { 0.0 };
            cc_to_send.push((22, value_to_send));
          }
          ui.add_space(5.0);
          if self.styled_button(ui, "BUTTON 4", false).clicked() {
            cc_to_send.push((23, 1.0));
          }
        });
      });
      ui.add_space(10.0);
    });

    for (controller, value) in cc_to_send {
      self.send_cc(controller, value);
    }
  }

  fn styled_button(&self, ui: &mut Ui, text: &str, pressed: bool) -> Response {
    let visuals = self.current_style.get_visuals();
    let (fill_color, text_color) = if pressed {
      (
        visuals.widgets.active.bg_fill,
        visuals.override_text_color.unwrap_or(Color32::BLACK),
      )
    } else {
      (
        visuals.widgets.inactive.bg_fill,
        visuals.override_text_color.unwrap_or(Color32::BLACK),
      )
    };

    ui.add(
      Button::new(
        RichText::new(text)
          .heading()
          .monospace()
          .color(text_color),
      )
        .min_size(Vec2::new(240.0, 80.0))
        .rounding(egui::Rounding::same(20.0))
        .fill(fill_color),
    )
  }
}

impl eframe::App for MyApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Handle incoming MIDI messages
    for msg in self.midi_rx.try_iter() {
      if let [status, key, _] = msg.as_slice() {
        // Key on
        if (status & 0xF0) == 0x90 {
          self.keyboard.add_key(*key);
        }
        // Key off
        if (status & 0xF0) == 0x80 {
          self.keyboard.remove_key(*key);
        }
      }
    }

    // Load logo texture once
    if self.logo_texture.is_none() {
      let image_bytes = include_bytes!("../logo.png");
      let image = image::load_from_memory(image_bytes).expect("Failed to load logo image");
      let size = [image.width() as _, image.height() as _];
      let image_buffer = image.to_rgba8();
      let pixels = image_buffer.as_flat_samples();
      let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

      self.logo_texture = Some(ctx.load_texture("logo", color_image, Default::default()));
    }

    // Set the visuals for the entire application based on the current style
    ctx.set_visuals(self.current_style.get_visuals());

    // Render UI based on the current application state
    match self.app_state {
      AppState::StartScreen => self.draw_start_screen(ctx),
      AppState::MainApp => self.draw_main_app(ctx),
      AppState::Diagnostics => self.draw_diagnostics_screen(ctx),
    }

    // Request a repaint to ensure the UI is updated
    ctx.request_repaint();
  }
}

pub fn run() -> Result<(), eframe::Error> {
  let mut app = MyApp::default();
  app.setup_midi();

  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default(),
    ..Default::default()
  };
  eframe::run_native(
    "Rust Synthesizer",
    options,
    Box::new(|_cc| Box::new(app)),
  )
}