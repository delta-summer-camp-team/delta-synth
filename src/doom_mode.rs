// src/doom_mode.rs

use eframe::egui::{self, Color32, Key, Pos2, Rect, Shape, Stroke, Ui, Vec2};
use crate::app::{MyApp, rotary_knob::RotaryKnob};
use std::time::Instant;

// --- Enemy Struct ---
pub struct Enemy {
  pub pos: Vec2,   // world position
  pub health: f32,
}

// --- Doom Mode State ---
pub struct DoomState {
  pub health: f32,
  pub armor: f32,
  pub shoot_button_pressed: bool,
  pub open_door_button_pressed: bool,
  pub last_shot_time: Option<Instant>,
  pub player_pos: Vec2,
  pub player_dir: Vec2,
  pub camera_plane: Vec2,
  pub monster_animation_time: f32,
  pub enemies: Vec<Enemy>,
  pub door_open_amount: f32,
  pub map: Vec<Vec<u8>>,
}

impl Default for DoomState {
  fn default() -> Self {
    Self {
      health: 100.0,
      armor: 0.0,
      shoot_button_pressed: false,
      open_door_button_pressed: false,
      last_shot_time: None,
      player_pos: egui::vec2(1.5, 1.5),
      player_dir: egui::vec2(-1.0, 0.0),
      camera_plane: egui::vec2(0.0, 0.66),
      monster_animation_time: 0.0,
      enemies: vec![
        Enemy { pos: egui::vec2(3.5, 3.5), health: 100.0 },
        Enemy { pos: egui::vec2(5.5, 2.5), health: 100.0 },
      ],
      door_open_amount: 0.0,
      map: vec![
        vec![1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 1, 0, 1, 1, 0, 1],
        vec![1, 0, 0, 0, 0, 1, 0, 1],
        vec![1, 0, 1, 1, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 1, 0, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1],
      ],
    }
  }
}

// --- Helper: Rotate Vec2 by angle (radians) ---
fn rotate_vec(v: Vec2, angle: f32) -> Vec2 {
  let (sin_a, cos_a) = angle.sin_cos();
  Vec2::new(v.x * cos_a - v.y * sin_a, v.x * sin_a + v.y * cos_a)
}

// --- Helper: Draw Enemy (uses screen coordinates: Pos2) ---
fn draw_enemy(painter: &egui::Painter, pos: Pos2, size: Vec2, time: f32) {
  // bobbing offset
  let body_bob = (time * 5.0).sin() * 5.0;
  // center adjusted by bob
  let center = Pos2::new(pos.x, pos.y + body_bob);
  let body_rect = Rect::from_center_size(center, size);
  painter.rect_filled(body_rect, 5.0, Color32::from_rgb(139, 0, 0)); // Dark Red

  // Horns (as small rects)
  let horn_size = egui::vec2(size.x * 0.2, size.y * 0.3);
  let left_horn_pos = Pos2::new(center.x - size.x * 0.4, center.y - size.y * 0.5);
  let right_horn_pos = Pos2::new(center.x + size.x * 0.4, center.y - size.y * 0.5);
  painter.rect_filled(Rect::from_center_size(left_horn_pos, horn_size), 2.0, Color32::from_gray(200));
  painter.rect_filled(Rect::from_center_size(right_horn_pos, horn_size), 2.0, Color32::from_gray(200));

  // Eye (circle)
  let eye_pos = Pos2::new(center.x, center.y - size.y * 0.2);
  painter.circle_filled(eye_pos, size.x * 0.15, Color32::YELLOW);
}

// --- Doom Screen Drawing ---
pub fn draw_doom_screen(app: &mut MyApp, ctx: &egui::Context) {
  egui::CentralPanel::default().show(ctx, |ui| {
    let time = ui.input(|i| i.time);
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter();

    // --- Movement & Rotation ---
    ui.input(|i| {
      let speed = 0.1;
      let rot_speed = 0.05;

      if i.key_down(Key::W) {
        let new_pos = app.doom_state.player_pos + app.doom_state.player_dir * speed;
        let mx = new_pos.x.floor() as usize;
        let my = new_pos.y.floor() as usize;
        if my < app.doom_state.map.len() && mx < app.doom_state.map[0].len()
          && app.doom_state.map[my][mx] == 0
        {
          app.doom_state.player_pos = new_pos;
        }
      }
      if i.key_down(Key::S) {
        let new_pos = app.doom_state.player_pos - app.doom_state.player_dir * speed;
        let mx = new_pos.x.floor() as usize;
        let my = new_pos.y.floor() as usize;
        if my < app.doom_state.map.len() && mx < app.doom_state.map[0].len()
          && app.doom_state.map[my][mx] == 0
        {
          app.doom_state.player_pos = new_pos;
        }
      }
      if i.key_down(Key::A) {
        app.doom_state.player_dir = rotate_vec(app.doom_state.player_dir, rot_speed);
        app.doom_state.camera_plane = rotate_vec(app.doom_state.camera_plane, rot_speed);
      }
      if i.key_down(Key::D) {
        app.doom_state.player_dir = rotate_vec(app.doom_state.player_dir, -rot_speed);
        app.doom_state.camera_plane = rotate_vec(app.doom_state.camera_plane, -rot_speed);
      }
    });

    // --- Update Animations ---
    app.doom_state.monster_animation_time = time as f32;
    if app.doom_state.open_door_button_pressed {
      app.doom_state.door_open_amount = (app.doom_state.door_open_amount + 0.05).min(1.0);
    } else {
      app.doom_state.door_open_amount = (app.doom_state.door_open_amount - 0.05).max(0.0);
    }

    // --- Ceiling & Floor ---
    let ceiling_rect = Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.center().y));
    let floor_rect = Rect::from_min_max(egui::pos2(rect.min.x, rect.center().y), rect.max);
    painter.rect_filled(ceiling_rect, 0.0, Color32::from_rgb(0x5a, 0x4a, 0x3a));
    painter.rect_filled(floor_rect, 0.0, Color32::from_rgb(0x3a, 0x2a, 0x1a));

    // --- Raycasting (column by column) ---
    let width_f = rect.width();
    let height_f = rect.height();
    for x in 0..(width_f as i32) {
      let camera_x = 2.0 * x as f32 / width_f - 1.0;
      let ray_dir = app.doom_state.player_dir + app.doom_state.camera_plane * camera_x;

      let mut map_pos = egui::vec2(
        app.doom_state.player_pos.x.floor(),
        app.doom_state.player_pos.y.floor(),
      );

      let delta_dist = egui::vec2(
        if ray_dir.x == 0.0 { f32::INFINITY } else { (1.0 / ray_dir.x).abs() },
        if ray_dir.y == 0.0 { f32::INFINITY } else { (1.0 / ray_dir.y).abs() },
      );

      let mut step = egui::vec2(0.0, 0.0);
      let mut side_dist = egui::vec2(0.0, 0.0);
      let mut side = 0;

      if ray_dir.x < 0.0 {
        step.x = -1.0;
        side_dist.x = (app.doom_state.player_pos.x - map_pos.x) * delta_dist.x;
      } else {
        step.x = 1.0;
        side_dist.x = (map_pos.x + 1.0 - app.doom_state.player_pos.x) * delta_dist.x;
      }
      if ray_dir.y < 0.0 {
        step.y = -1.0;
        side_dist.y = (app.doom_state.player_pos.y - map_pos.y) * delta_dist.y;
      } else {
        step.y = 1.0;
        side_dist.y = (map_pos.y + 1.0 - app.doom_state.player_pos.y) * delta_dist.y;
      }

      let mut hit = false;
      while !hit {
        if side_dist.x < side_dist.y {
          side_dist.x += delta_dist.x;
          map_pos.x += step.x;
          side = 0;
        } else {
          side_dist.y += delta_dist.y;
          map_pos.y += step.y;
          side = 1;
        }

        let mx = map_pos.x as isize;
        let my = map_pos.y as isize;
        if my < 0 || mx < 0 {
          break;
        }
        let mxu = mx as usize;
        let myu = my as usize;
        if myu >= app.doom_state.map.len() || mxu >= app.doom_state.map[0].len() {
          break;
        }
        if app.doom_state.map[myu][mxu] > 0 {
          hit = true;
        }
      }
      if !hit { continue; }

      let perp_wall_dist = if side == 0 {
        (map_pos.x - app.doom_state.player_pos.x + (1.0 - step.x) / 2.0) / ray_dir.x
      } else {
        (map_pos.y - app.doom_state.player_pos.y + (1.0 - step.y) / 2.0) / ray_dir.y
      };

      // avoid crazy numbers
      if !perp_wall_dist.is_finite() || perp_wall_dist.abs() < 1e-6 { continue; }

      let line_height = (height_f / perp_wall_dist).abs();
      let draw_start = (-line_height / 2.0 + height_f / 2.0).max(0.0);
      let draw_end = (line_height / 2.0 + height_f / 2.0).min(height_f - 1.0);

      let wall_color = if side == 1 { Color32::from_gray(120) } else { Color32::from_gray(150) };
      painter.line_segment(
        [egui::pos2(x as f32, draw_start), egui::pos2(x as f32, draw_end)],
        Stroke::new(1.0, wall_color),
      );
    }

    // --- Enemies (sprites) ---
    for enemy in &app.doom_state.enemies {
      let sprite_pos = enemy.pos - app.doom_state.player_pos;
      let det = (app.doom_state.camera_plane.x * app.doom_state.player_dir.y
        - app.doom_state.player_dir.x * app.doom_state.camera_plane.y);
      if det.abs() < 1e-6 { continue; } // avoid divide by ~0
      let inv_det = 1.0 / det;

      let transform = egui::vec2(
        inv_det * (app.doom_state.player_dir.y * sprite_pos.x - app.doom_state.player_dir.x * sprite_pos.y),
        inv_det * (-app.doom_state.camera_plane.y * sprite_pos.x + app.doom_state.camera_plane.x * sprite_pos.y),
      );

      if transform.y > 0.0 {
        let sprite_screen_x = (rect.width() / 2.0) * (1.0 + transform.x / transform.y);
        let sprite_height = (rect.height() / transform.y).abs();
        let draw_start_y = (-sprite_height / 2.0 + rect.height() / 2.0).max(0.0);
        let draw_end_y = (sprite_height / 2.0 + rect.height() / 2.0).min(rect.height() - 1.0);
        let sprite_width = (rect.width() / transform.y).abs();
        let draw_start_x = -sprite_width / 2.0 + sprite_screen_x;
        let draw_end_x = sprite_width / 2.0 + sprite_screen_x;

        if draw_start_x < rect.width() && draw_end_x > 0.0 {
          let enemy_pos = Pos2::new(sprite_screen_x, (draw_start_y + draw_end_y) / 2.0);
          let enemy_size = egui::vec2(sprite_width, sprite_height);
          draw_enemy(painter, enemy_pos, enemy_size, app.doom_state.monster_animation_time);
        }
      }
    }

    // --- Weapon & Muzzle Flash ---
    let weapon_pos = egui::pos2(rect.center().x, rect.max.y - 60.0);
    let weapon_size = egui::vec2(100.0, 120.0);
    let weapon_rect = Rect::from_center_size(weapon_pos, weapon_size);
    painter.rect_filled(weapon_rect, 0.0, Color32::from_gray(50));
    painter.rect_stroke(weapon_rect, 0.0, Stroke::new(2.0, Color32::from_gray(140)));

    if let Some(shot_time) = app.doom_state.last_shot_time {
      if shot_time.elapsed().as_millis() < 100 {
        let flash_size = 60.0;
        let flash_pos = Pos2::new(weapon_rect.center().x, weapon_rect.center().y - weapon_rect.height() / 2.0 - 20.0);
        let flash_shape = Shape::circle_filled(flash_pos, flash_size, Color32::from_rgba_unmultiplied(255, 255, 0, 200));
        painter.add(flash_shape);
      }
    }

    // --- UI Controls Overlay ---
    egui::Area::new("doom_ui_controls".into())
      .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(10.0, -10.0))
      .show(ctx, |ui| {
        ui.horizontal(|ui| {
          ui.add(RotaryKnob::new(&mut app.doom_state.health, 0.0, 100.0).with_label("Health").with_size(100.0));
          ui.add(RotaryKnob::new(&mut app.doom_state.armor, 0.0, 100.0).with_label("Armor").with_size(100.0));
          ui.add_space(50.0);
          ui.vertical(|ui| {
            if app.styled_button(ui, "SHOOT", app.doom_state.shoot_button_pressed).clicked() {
              app.doom_state.last_shot_time = Some(Instant::now());
            }
            if app.styled_button(ui, "OPEN DOOR", app.doom_state.open_door_button_pressed).clicked() {
              app.doom_state.open_door_button_pressed = !app.doom_state.open_door_button_pressed;
            }
          });
        });
      });

    egui::Area::new("doom_back_button".into())
      .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
      .show(ctx, |ui| {
        if ui.button("↩ Back").clicked() {
          app.app_state = super::app::AppState::StartScreen;
        }
      });
  });
}
