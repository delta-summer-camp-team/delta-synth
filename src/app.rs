// src/app.rs
use crate::doom_mode::DoomState;
use crate::gui_style::GUIStyle;
use crate::keyboard::Keyboard;
use eframe::egui;
use eframe::egui::{
  Button, Color32, Label, Pos2, Rect, Response, RichText, Sense, Shape, TextStyle, TextureHandle,
  Ui, Vec2, Widget,
};
use midir::{MidiInput, MidiInputPort, MidiOutput, MidiOutputConnection, MidiOutputPort};
use std::sync::mpsc::{channel, Receiver, Sender};

// --- AppState Enum ---
// Manages the current view of the application.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AppState {
  StartScreen,
  MainApp,
  Diagnostics,
  DoomMode,
}

// --- Rotary Knob Widget ---
// This is a custom widget for the circular knobs.
pub mod rotary_knob {
  use super::*;

  pub struct RotaryKnob<'a> {
    pub value: &'a mut f32,
    pub min: f32,
    pub max: f32,
    pub size: f32,
    pub label: Option<&'a str>,
    pub show_value: bool,
    pub texture: Option<&'a TextureHandle>,
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
        texture: None,
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

    pub fn with_texture(mut self, texture: &'a TextureHandle) -> Self {
      self.texture = Some(texture);
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
        texture,
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

      if let Some(tex) = texture {
        // If a texture is provided, draw it clipped to a circle
        let circle = Shape::circle_filled(center, radius, Color32::WHITE);
        let mut mesh = egui::Mesh::with_texture(tex.id());
        mesh.add_rect_with_uv(
          rect,
          Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
          Color32::WHITE,
        );
        let clip_shape = Shape::Vec(vec![circle, Shape::Mesh(mesh)]);
        painter.add(clip_shape);
      } else {
        // Otherwise, draw a simple circle
        painter.circle(center, radius - 2.0, visuals.bg_fill, visuals.fg_stroke);
      }

      // Draw the pointer line
      let normalized_value = (*value - min) / (max - min);
      let angle = (normalized_value * std::f32::consts::TAU) - std::f32::consts::PI;
      let pointer = Vec2::angled(angle) * radius * 0.7;
      painter.line_segment([center, center + pointer], visuals.fg_stroke);

      // Draw the numeric value inside the knob
      if show_value {
        let val_str = format!("{:.0}", *value * 100.0); // Show as percentage
        let font = TextStyle::Small.resolve(ui.style());
        painter.text(
          center,
          egui::Align2::CENTER_CENTER,
          val_str,
          font,
          visuals.text_color(),
        );
      }

      // Draw the label below the knob
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

// --- Main Application Struct ---
pub struct MyApp {
  pub knobs: [f32; 20],
  pub sliders: [f32; 20],
  pub midi_out: Option<MidiOutputConnection>,
  pub midi_in: Option<midir::MidiInputConnection<()>>,
  pub midi_status: String,
  pub button1_pressed: bool,
  pub button3_pressed: bool,
  pub logo_texture: Option<TextureHandle>,
  pub antonui_texture: Option<TextureHandle>,
  pub current_style: GUIStyle,
  pub app_state: AppState,
  pub is_fullscreen: bool,
  pub keyboard: Keyboard,
  pub midi_rx: Receiver<Vec<u8>>,
  pub midi_tx: Sender<Vec<u8>>,
  // MIDI Port selection
  pub midi_out_ports: Vec<(String, MidiOutputPort)>,
  pub selected_midi_out_port: Option<usize>,
  pub midi_in_ports: Vec<(String, MidiInputPort)>,
  pub selected_midi_in_port: Option<usize>,
  // Doom mode specific state
  pub doom_state: DoomState,
}

impl Default for MyApp {
  fn default() -> Self {
    let (midi_tx, midi_rx) = channel();
    Self {
      knobs: [0.0; 20],
      sliders: [0.0; 20],
      midi_out: None,
      midi_in: None,
      midi_status: "Initializing MIDI...".to_string(),
      button1_pressed: false,
      button3_pressed: false,
      logo_texture: None,
      antonui_texture: None,
      current_style: GUIStyle::DarkMode,
      app_state: AppState::StartScreen,
      is_fullscreen: true,
      keyboard: Keyboard::new(),
      midi_rx,
      midi_tx,
      midi_out_ports: Vec::new(),
      selected_midi_out_port: None,
      midi_in_ports: Vec::new(),
      selected_midi_in_port: None,
      doom_state: DoomState::default(),
    }
  }
}

impl MyApp {

  // --- MIDI Setup and Communication ---
  fn setup_midi(&mut self) {
    // --- MIDI Output ---
    let midi_out = match MidiOutput::new("egui-midi-synth") {
      Ok(m) => m,
      Err(e) => {
        self.midi_status = format!("❌ MIDI Output Init Error: {}", e);
        return;
      }
    };

    self.midi_out_ports = midi_out
      .ports()
      .iter()
      .map(|p| (midi_out.port_name(p).unwrap_or_default(), p.clone()))
      .collect();

    if self.midi_out_ports.is_empty() {
      self.midi_status = "⚠️ No MIDI output ports found.".to_string();
    } else {
      self.midi_status = format!("Found {} MIDI output ports.", self.midi_out_ports.len());
      self.connect_midi_out(0); // Automatically connect to the first output port
    }

    // --- MIDI Input ---
    let midi_in = match MidiInput::new("egui-midi-synth-in") {
      Ok(m) => m,
      Err(e) => {
        self.midi_status = format!("{} ❌ MIDI Input Init Error: {}", self.midi_status, e);
        return;
      }
    };

    self.midi_in_ports = midi_in
      .ports()
      .iter()
      .map(|p| (midi_in.port_name(p).unwrap_or_default(), p.clone()))
      .collect();

    if self.midi_in_ports.is_empty() {
      self.midi_status = format!("{} ⚠️ No MIDI input ports found.", self.midi_status);
    } else {
      self.midi_status = format!(
        "{} Found {} MIDI input ports.",
        self.midi_status,
        self.midi_in_ports.len()
      );
      self.connect_midi_in(0); // Automatically connect to the first input port
    }
  }

  fn connect_midi_out(&mut self, port_index: usize) {
    if let Some((port_name, port)) = self.midi_out_ports.get(port_index) {
      println!("Attempting to connect to MIDI output port: {}", port_name);
      let midi_out = MidiOutput::new("egui-midi-synth-connect").unwrap();
      match midi_out.connect(port, "egui-midi-output") {
        Ok(conn) => {
          self.midi_out = Some(conn);
          self.selected_midi_out_port = Some(port_index);
          self.midi_status = format!("✅ Connected to Output: {}", port_name);
          println!("Successfully connected to MIDI output port: {}", port_name);
        }
        Err(e) => {
          self.midi_status = format!("❌ MIDI Output Connect Error: {}", e);
          println!("Failed to connect to MIDI output port: {}. Error: {}", port_name, e);
        }
      }
    }
  }

  fn connect_midi_in(&mut self, port_index: usize) {
    if let Some((port_name, port)) = self.midi_in_ports.get(port_index) {
      println!("Attempting to connect to MIDI input port: {}", port_name);
      self.midi_in = None; // Drop the old connection

      let midi_in = MidiInput::new("egui-midi-synth-in-connect").unwrap();
      let tx = self.midi_tx.clone();
      let port_clone = port.clone();
      let conn_in = midi_in.connect(
        &port_clone,
        "egui-midi-input",
        move |_, message, _| {
          println!("MIDI in: {:?}", message);
          tx.send(message.to_vec()).unwrap();
        },
        (),
      );

      match conn_in {
        Ok(conn) => {
          self.midi_in = Some(conn);
          self.selected_midi_in_port = Some(port_index);
          self.midi_status = format!("{} ✅ Connected to Input: {}", self.midi_status, port_name);
          println!("Successfully connected to MIDI input port: {}", port_name);
        }
        Err(e) => {
          self.midi_status =
            format!("{} ❌ MIDI Input Connect Error: {}", self.midi_status, e);
          println!("Failed to connect to MIDI input port: {}. Error: {}", port_name, e);
        }
      }
    }
  }

  fn send_cc(&mut self, controller: u8, value: f32) {
    if let Some(ref mut conn) = self.midi_out {
      let midi_value = (value.clamp(0.0, 1.0) * 127.0).round() as u8;
      let msg = [0xB0, controller, midi_value];
      println!("MIDI out: {:?}", msg);
      let _ = conn.send(&msg);
    }
  }

  // --- UI Drawing Functions for Different States ---

  fn draw_start_screen(&mut self, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.1);
        if let Some(texture) = &self.logo_texture {
          let img = egui::Image::new(egui::ImageSource::Texture(
            egui::load::SizedTexture::new(texture.id(), texture.size_vec2()),
          ));
          let sized_img = img.fit_to_exact_size(Vec2::new(1000.0, 200.0));
          ui.add(sized_img);
        }
        ui.add_space(50.0);
        ui.heading("Choose a Style");
        ui.add_space(20.0);

        if ui
          .add(Button::new("Orange Mode").min_size(Vec2::new(200.0, 50.0)).rounding(10.0))
          .clicked()
        {
          self.current_style = GUIStyle::OrangeMode;
          self.app_state = AppState::MainApp;
        }
        ui.add_space(10.0);
        if ui
          .add(Button::new("Dark Mode").min_size(Vec2::new(200.0, 50.0)).rounding(10.0))
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
          .add(Button::new("Anton Mode").min_size(Vec2::new(200.0, 50.0)).rounding(10.0))
          .clicked()
        {
          self.current_style = GUIStyle::AntonMode;
          self.app_state = AppState::MainApp;
        }
        ui.add_space(10.0);
        if ui
          .add(Button::new("Doom Mode").min_size(Vec2::new(200.0, 50.0)).rounding(10.0))
          .clicked()
        {
          self.current_style = GUIStyle::DoomMode;
          self.app_state = AppState::DoomMode;
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
        ui.label(&self.midi_status);
        if ui.button("Toggle Fullscreen").clicked() {
          self.is_fullscreen = !self.is_fullscreen;
          ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
        }
      });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
      ui.columns(5, |columns| {
        // --- Column 1 ---
        columns[0].vertical(|ui| {
          for i in 0..4 {
            if ui
              .add(
                RotaryKnob::new(&mut self.knobs[i], 0.0, 1.0)
                  .with_label(&format!("K{}", i + 1))
                  .with_size(187.5),
              )
              .changed()
            {
              cc_to_send.push((i as u8, self.knobs[i]));
            }
            ui.add_space(5.0);
          }
        });

        // --- Column 2 ---
        columns[1].vertical(|ui| {
          for i in 0..8 {
            ui.label(format!("Slider {}", i + 1));
            let slider = egui::Slider::new(&mut self.sliders[i], 0.0..=1.0);
            if ui.add_sized(Vec2::new(240.0, 30.0), slider).changed() {
              cc_to_send.push((11 + i as u8, self.sliders[i]));
            }
            if i % 2 != 0 {
              ui.add_space(95.0); // Spacing to align with knobs
            }
          }
        });

        // --- Column 3 ---
        columns[2].vertical_centered(|ui| {
          if let Some(texture) = &self.logo_texture {
            ui.image(egui::ImageSource::Texture(
              egui::load::SizedTexture::new(texture.id(), texture.size_vec2() / 8.0),
            ));
          }

          for i in 4..6 {
            if ui
              .add(
                RotaryKnob::new(&mut self.knobs[i], 0.0, 1.0)
                  .with_label(&format!("K{}", i + 1))
                  .with_size(250.0),
              )
              .changed()
            {
              cc_to_send.push((i as u8, self.knobs[i]));
            }
            ui.add_space(5.0);
          }
        });

        // --- Column 4 ---
        columns[3].vertical(|ui| {
          for i in 8..16 {
            ui.label(format!("Slider {}", i + 1));
            let slider = egui::Slider::new(&mut self.sliders[i], 0.0..=1.0);
            if ui.add_sized(Vec2::new(240.0, 30.0), slider).changed() {
              cc_to_send.push((33 + i as u8, self.sliders[i]));
            }
            if i % 2 != 0 {
              ui.add_space(95.0); // Spacing to align with knobs
            }
          }
        });

        // --- Column 5 ---
        columns[4].vertical(|ui| {
          ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.set_height(ui.available_height() / 3.0);
            for i in 8..12 {
              if ui
                .add(
                  RotaryKnob::new(&mut self.knobs[i], 0.0, 1.0)
                    .with_label(&format!("K{}", i + 1))
                    .with_size(187.5),
                )
                .changed()
              {
                cc_to_send.push((i as u8, self.knobs[i]));
              }
              ui.add_space(5.0);
            };
            ui.label("Slider 12");
            let slider = egui::Slider::new(&mut self.sliders[11], 0.0..=1.0);
            if ui.add_sized(Vec2::new(360.0, 30.0), slider).changed() {
              cc_to_send.push((31, self.sliders[11]));
            }
          });
          for i in 10..12 {
            if ui
              .add(
                RotaryKnob::new(&mut self.knobs[i], 0.0, 1.0)
                  .with_label(&format!("K{}", i + 1))
                  .with_size(156.25),
              )
              .changed()
            {
              cc_to_send.push((i as u8, self.knobs[i]));
            }
            ui.add_space(5.0);
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
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
          }
          ui.add_space(10.0);
          self.keyboard.ui(ui);
          ui.label(&self.midi_status);

          // --- MIDI Port Selection ---
          ui.add_space(10.0);
          ui.heading("MIDI Connections");

          // Output Port Selector
          let selected_out_text = if let Some(index) = self.selected_midi_out_port {
            self.midi_out_ports.get(index).map_or("None", |(name, _)| name)
          } else {
            "None"
          };

          let mut selected_out_port_index = None;
          egui::ComboBox::from_label("Output Port")
            .selected_text(selected_out_text)
            .show_ui(ui, |ui| {
              for (i, (name, _)) in self.midi_out_ports.iter().enumerate() {
                if ui.selectable_label(self.selected_midi_out_port == Some(i), name).clicked() {
                  selected_out_port_index = Some(i);
                }
              }
            });

          if let Some(index) = selected_out_port_index {
            self.connect_midi_out(index);
          }

          // Input Port Selector
          let selected_in_text = if let Some(index) = self.selected_midi_in_port {
            self.midi_in_ports.get(index).map_or("None", |(name, _)| name)
          } else {
            "None"
          };

          let mut selected_in_port_index = None;
          egui::ComboBox::from_label("Input Port")
            .selected_text(selected_in_text)
            .show_ui(ui, |ui| {
              for (i, (name, _)) in self.midi_in_ports.iter().enumerate() {
                if ui.selectable_label(self.selected_midi_in_port == Some(i), name).clicked() {
                  selected_in_port_index = Some(i);
                }
              }
            });

          if let Some(index) = selected_in_port_index {
            self.connect_midi_in(index);
          }

          ui.add_space(5.0);
        });
        ui.add_space(50.0);
        ui.columns(2, |columns| {
          columns[0].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
              .add(
                RotaryKnob::new(&mut self.knobs[0], 0.0, 1.0)
                  .with_label("CUTOFF")
                  .with_size(200.0)
                  .show_value(true),
              )
              .changed()
            {
              cc_to_send.push((10, self.knobs[0]));
            }
          });
          columns[1].with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            if ui
              .add(
                RotaryKnob::new(&mut self.knobs[1], 0.0, 1.0)
                  .with_label("RESONANCE")
                  .with_size(200.0)
                  .show_value(true),
              )
              .changed()
            {
              cc_to_send.push((11, self.knobs[1]));
            }
          });
        });
      });

    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
      ui.add_space(10.0);
      ui.columns(3, |columns| {
        columns[0].vertical_centered(|ui| {
          if self
            .styled_button(ui, "BUTTON 1", self.button1_pressed)
            .clicked()
          {
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
                for (i, val) in self.sliders.iter_mut().take(4).enumerate() {
                  ui.vertical(|ui| {
                    ui.label(format!("S{}", i + 1));
                    let slider =
                      egui::Slider::new(val, -0.5..=0.5).vertical().text("");
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
          if self
            .styled_button(ui, "BUTTON 3", self.button3_pressed)
            .clicked()
          {
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

  pub fn styled_button(&self, ui: &mut Ui, text: &str, pressed: bool) -> Response {
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
      Button::new(RichText::new(text).heading().monospace().color(text_color))
        .min_size(Vec2::new(240.0, 80.0))
        .rounding(egui::Rounding::same(20.0))
        .fill(fill_color),
    )
  }
}

// --- eframe App Implementation ---
impl eframe::App for MyApp {

  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // Handle incoming MIDI messages
    for msg in self.midi_rx.try_iter() {
      // Forward the MIDI message directly to the output
      if let Some(ref mut conn) = self.midi_out {
        let _ = conn.send(&msg);
      }

      if let [status, key, _] = msg.as_slice() {
        if (status & 0xF0) == 0x90 {
          self.keyboard.add_key(*key);
        }
        if (status & 0xF0) == 0x80 {
          self.keyboard.remove_key(*key);
        }
      }
    }

    // Load logo and antonui textures once
    if self.logo_texture.is_none() {
      let image_bytes = include_bytes!("../logo.png");
      let image = image::load_from_memory(image_bytes).expect("Failed to load logo image");
      let size = [image.width() as _, image.height() as _];
      let image_buffer = image.to_rgba8();
      let pixels = image_buffer.as_flat_samples();
      let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
      self.logo_texture = Some(ctx.load_texture("logo", color_image, Default::default()));
    }

    if self.antonui_texture.is_none() {
      let image_bytes = include_bytes!("../antonui.jpg");
      let image = image::load_from_memory(image_bytes).expect("Failed to load antonui image");
      let size = [image.width() as _, image.height() as _];
      let image_buffer = image.to_rgba8();
      let pixels = image_buffer.as_flat_samples();
      let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
      self.antonui_texture =
        Some(ctx.load_texture("antonui", color_image, Default::default()));

    }

    // Set the visuals for the entire application
    ctx.set_visuals(self.current_style.get_visuals());

    // Render UI based on the current application state
    match self.app_state {
      AppState::StartScreen => self.draw_start_screen(ctx),
      AppState::MainApp => self.draw_main_app(ctx),
      AppState::Diagnostics => self.draw_diagnostics_screen(ctx),
      AppState::DoomMode => crate::doom_mode::draw_doom_screen(self, ctx),
    }

    // Request a repaint to ensure animations and updates are smooth
    ctx.request_repaint();
  }
}

// --- Main Function ---
pub fn run() -> Result<(), eframe::Error> {

  let mut app = MyApp::default();
  app.setup_midi();
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
      .with_resizable(true)
      .with_inner_size(egui::vec2(1920.0, 1080.0)),
    ..Default::default()
  };
  eframe::run_native(
    "Rust Synthesizer",
    options,
    Box::new(|_cc| Box::new(app)),
  )
}
