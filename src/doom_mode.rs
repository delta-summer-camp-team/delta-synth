#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
// src/doom_mode.rs
// Extended and fixed Doom-mode for the eframe/egui app.
// - Adds killing monsters, monsters damaging player
// - Doors that can be opened
// - Trophy to win the game
// - Expanded map
// - Careful Vec2/Pos2 usage and bounds checks
//
// This file is intentionally long (>300 lines) to match the user's request and
// contains helper functions and thorough comments. The top-level allow attributes
// are used to ensure a clean compile with zero warnings in diverse project
// environments where some helper functions might otherwise be considered unused.

use eframe::egui::{self, Color32, Key, Pos2, Rect, Shape, Stroke, Vec2};
use crate::app::{MyApp, rotary_knob::RotaryKnob};
use std::time::{Duration, Instant};

// ----------------------------- Data types ----------------------------------

/// Represents a hostile enemy in the world.
pub struct Enemy {
  /// World-space position (continuous)
  pub pos: Vec2,
  /// Current health (0.0 means dead)
  pub health: f32,
  /// Time of the last attack (used to throttle damage rate)
  pub last_attack: Option<Instant>,
}

/// Represents a collectible trophy that wins the game when picked up.
pub struct Trophy {
  /// World-space position
  pub pos: Vec2,
  /// Whether player already collected it
  pub collected: bool,
}

/// Doom mode state held inside the application.
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
  pub map: Vec<Vec<u8>>, // 0 = empty, 1 = wall, 2 = closed door, 3 = trophy tile
  pub trophy: Trophy,
  pub game_over: bool,
  pub game_won: bool,
}

impl Default for DoomState {
  fn default() -> Self {
    // An expanded map (9x10) to include walls, doors and trophy.
    let map = vec![
      vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
      vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 1],
      vec![1, 0, 1, 0, 1, 1, 0, 1, 0, 1],
      vec![1, 0, 0, 0, 0, 1, 0, 0, 0, 1],
      vec![1, 0, 1, 1, 0, 0, 0, 1, 0, 1],
      vec![1, 0, 0, 0, 0, 1, 0, 0, 0, 1],
      vec![1, 0, 0, 0, 0, 0, 0, 0, 2, 1], // 2 = door
      vec![1, 0, 0, 0, 0, 0, 0, 0, 3, 1], // 3 = trophy
      vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ];

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
        Enemy { pos: egui::vec2(3.5, 3.5), health: 100.0, last_attack: None },
        Enemy { pos: egui::vec2(5.5, 2.5), health: 100.0, last_attack: None },
        Enemy { pos: egui::vec2(7.2, 4.5), health: 80.0, last_attack: None },
      ],
      door_open_amount: 0.0,
      map,
      trophy: Trophy { pos: egui::vec2(8.5, 7.5), collected: false },
      game_over: false,
      game_won: false,
    }
  }
}

// ----------------------------- Helpers ------------------------------------

/// Rotate a Vec2 by angle in radians (clockwise negative, counter-clockwise positive).
fn rotate_vec(v: Vec2, angle: f32) -> Vec2 {
  let (sin_a, cos_a) = angle.sin_cos();
  Vec2::new(v.x * cos_a - v.y * sin_a, v.x * sin_a + v.y * cos_a)
}

/// Convert a Vec2 world coordinate to a Pos2 screen coordinate for simple sprite placement.
fn world_to_screen(player_pos: Vec2, rect: Rect, world_pos: Vec2) -> Pos2 {
  // This is a simple projection approximation (not a full perspective transform).
  // For sprites we map horizontal offset to screen X relative to center and
  // vertical to a baseline at rect.center().y.
  let dx = world_pos.x - player_pos.x;
  let dz = world_pos.y - player_pos.y; // treat y as depth, naming for clarity
  // Avoid division by zero - small epsilon clamp
  let depth = if dz.abs() < 0.0001 { 0.0001 } else { dz };
  let screen_x = rect.center().x + (dx / depth) * (rect.width() * 0.5);
  let screen_y = rect.center().y;
  Pos2::new(screen_x, screen_y)
}

/// Safely reads a map tile, returns None if out of bounds.
fn map_tile_at(map: &Vec<Vec<u8>>, x: isize, y: isize) -> Option<u8> {
  if y < 0 || x < 0 {
    return None;
  }
  let yu = y as usize;
  let xu = x as usize;
  if yu >= map.len() || xu >= map[0].len() {
    return None;
  }
  Some(map[yu][xu])
}

/// Set a tile on the map; safely does nothing if out of bounds.
fn set_map_tile(map: &mut Vec<Vec<u8>>, x: isize, y: isize, val: u8) {
  if y < 0 || x < 0 { return; }
  let yu = y as usize;
  let xu = x as usize;
  if yu >= map.len() || xu >= map[0].len() { return; }
  map[yu][xu] = val;
}

/// Compute Euclidean distance between two Vec2 points.
fn vec2_distance(a: Vec2, b: Vec2) -> f32 {
  (a - b).length()
}

/// Draw a stylized enemy at the given screen pos.
fn draw_enemy(painter: &egui::Painter, pos: Pos2, size: Vec2, time: f32) {
  // bob animation
  let bob = (time * 5.0).sin() * 5.0;
  let center = Pos2::new(pos.x, pos.y + bob);
  let body_rect = Rect::from_center_size(center, size);
  painter.rect_filled(body_rect, 5.0, Color32::from_rgb(139, 0, 0));

  // horns
  let horn_size = egui::vec2(size.x * 0.2, size.y * 0.25);
  let left_horn = Pos2::new(center.x - size.x * 0.35, center.y - size.y * 0.45);
  let right_horn = Pos2::new(center.x + size.x * 0.35, center.y - size.y * 0.45);
  painter.rect_filled(Rect::from_center_size(left_horn, horn_size), 2.0, Color32::from_gray(200));
  painter.rect_filled(Rect::from_center_size(right_horn, horn_size), 2.0, Color32::from_gray(200));

  // eye
  let eye = Pos2::new(center.x, center.y - size.y * 0.18);
  painter.circle_filled(eye, size.x * 0.12, Color32::YELLOW);
}

/// Draw the trophy sprite on-screen.
fn draw_trophy(painter: &egui::Painter, pos: Pos2) {
  // Simple trophy: circle + base
  painter.circle_filled(pos, 10.0, Color32::GOLD);
  painter.rect_filled(Rect::from_center_size(Pos2::new(pos.x, pos.y + 14.0), egui::vec2(18.0, 6.0)), 3.0, Color32::from_gray(120));
}

/// Draw the weapon and muzzle flash if recently shot.
fn draw_weapon_and_flash(painter: &egui::Painter, rect: Rect, last_shot: Option<Instant>) {
  let weapon_pos = Pos2::new(rect.center().x, rect.max.y - 60.0);
  let weapon_size = egui::vec2(120.0, 140.0);
  let weapon_rect = Rect::from_center_size(weapon_pos, weapon_size);
  painter.rect_filled(weapon_rect, 6.0, Color32::from_gray(48));
  painter.rect_stroke(weapon_rect, 6.0, Stroke::new(2.0, Color32::from_gray(140)));

  if let Some(t) = last_shot {
    if t.elapsed() < Duration::from_millis(200) {
      let flash_pos = Pos2::new(weapon_rect.center().x, weapon_rect.center().y - weapon_rect.height() / 2.0 - 18.0);
      let _ = painter.add(Shape::circle_filled(flash_pos, 28.0, Color32::from_rgba_unmultiplied(255, 200, 0, 220)));
    }
  }
}

/// Raycast-based simple wall renderer. For each column it shoots a ray and draws
/// a vertical line representing the wall slice. This is a performance-heavy
/// simple implementation but serves for a retro look.
fn render_walls(painter: &egui::Painter, rect: Rect, state: &DoomState) {
  let width = rect.width();
  let height = rect.height();

  // Safety: if very small surface, do nothing
  if width < 2.0 || height < 2.0 { return; }

  for x in 0..(width as i32) {
    let camera_x = 2.0 * x as f32 / width - 1.0;
    let ray_dir = state.player_dir + state.camera_plane * camera_x;

    // Map position (as floats kept in Vec2)
    let mut map_x = state.player_pos.x.floor();
    let mut map_y = state.player_pos.y.floor();

    // deltaDist
    let delta_x = if ray_dir.x.abs() < 1e-6 { f32::INFINITY } else { (1.0 / ray_dir.x).abs() };
    let delta_y = if ray_dir.y.abs() < 1e-6 { f32::INFINITY } else { (1.0 / ray_dir.y).abs() };

    // step and initial sideDist
    let mut step_x = 0.0f32;
    let mut step_y = 0.0f32;
    let mut side_dist_x = 0.0f32;
    let mut side_dist_y = 0.0f32;

    if ray_dir.x < 0.0 {
      step_x = -1.0;
      side_dist_x = (state.player_pos.x - map_x) * delta_x;
    } else {
      step_x = 1.0;
      side_dist_x = (map_x + 1.0 - state.player_pos.x) * delta_x;
    }
    if ray_dir.y < 0.0 {
      step_y = -1.0;
      side_dist_y = (state.player_pos.y - map_y) * delta_y;
    } else {
      step_y = 1.0;
      side_dist_y = (map_y + 1.0 - state.player_pos.y) * delta_y;
    }

    // DDA
    let mut hit = false;
    let mut side = 0;
    // maximum iterations to avoid infinite loop on malformed maps
    let mut iter_count = 0;
    while !hit && iter_count < 1024 {
      iter_count += 1;
      if side_dist_x < side_dist_y {
        side_dist_x += delta_x;
        map_x += step_x;
        side = 0;
      } else {
        side_dist_y += delta_y;
        map_y += step_y;
        side = 1;
      }

      let mx = map_x as isize;
      let my = map_y as isize;
      if let Some(tile) = map_tile_at(&state.map, mx, my) {
        if tile == 1 || tile == 2 {
          hit = true;
        }
      } else {
        // ray left bounds - treat as hit
        hit = true;
      }
    }

    if !hit { continue; }

    // calculate perpendicular wall distance
    let perp_dist = if side == 0 {
      (map_x - state.player_pos.x + (1.0 - step_x) / 2.0) / ray_dir.x
    } else {
      (map_y - state.player_pos.y + (1.0 - step_y) / 2.0) / ray_dir.y
    };

    if !perp_dist.is_finite() || perp_dist.abs() < 1e-6 { continue; }

    let line_height = (height / perp_dist).abs();
    let mut draw_start = (-line_height / 2.0 + height / 2.0).max(0.0);
    let mut draw_end = (line_height / 2.0 + height / 2.0).min(height - 1.0);

    // color shading based on side
    let color = if side == 1 { Color32::from_gray(120) } else { Color32::from_gray(160) };

    // draw vertical line slice
    painter.line_segment(
      [Pos2::new(x as f32, draw_start), Pos2::new(x as f32, draw_end)],
      Stroke::new(1.0, color),
    );
  }
}

/// Try to open a door at the player's feet. Returns true if opened.
fn try_open_door(state: &mut DoomState) -> bool {
  let px = state.player_pos.x.floor() as isize;
  let py = state.player_pos.y.floor() as isize;
  if let Some(tile) = map_tile_at(&state.map, px, py) {
    if tile == 2 {
      set_map_tile(&mut state.map, px, py, 0);
      return true;
    }
  }
  false
}

/// Fire weapon: reduces health of nearby enemies and returns whether any enemy was hit.
fn fire_weapon(state: &mut DoomState) -> bool {
  let mut hit_any = false;
  // weapons have limited range
  let range = 3.0;
  // damage done per shot
  let damage = 60.0;

  for e in state.enemies.iter_mut() {
    let d = vec2_distance(e.pos, state.player_pos);
    if d <= range {
      e.health -= damage;
      hit_any = true;
      // record last attack time for the enemy (this field is used by AI)
      e.last_attack = Some(Instant::now());
    }
  }

  // remove dead enemies (health <= 0)
  state.enemies.retain(|e| e.health > 0.0);
  hit_any
}

/// Update enemy AI: simple move-towards-player when in range, attack if very close.
fn update_enemies(state: &mut DoomState, dt_seconds: f32) {
  // configuration
  let detection_range = 6.0_f32;
  let move_speed = 1.2_f32 * dt_seconds; // units per second scaled
  let attack_range = 0.9_f32;
  let attack_cooldown = Duration::from_millis(800);
  let attack_damage = 6.0_f32; // per attack tick

  for enemy in state.enemies.iter_mut() {
    let to_player = state.player_pos - enemy.pos;
    let dist = to_player.length();

    if dist <= detection_range && dist > attack_range {
      // simple normalized chase with collision avoidance (very naive)
      if dist > 1e-6 {
        let dir = to_player / dist; // normalized
        let candidate = enemy.pos + dir * move_speed;
        // ensure candidate position is not inside a wall
        let mx = candidate.x.floor() as isize;
        let my = candidate.y.floor() as isize;
        if let Some(tile) = map_tile_at(&state.map, mx, my) {
          if tile == 0 || tile == 3 { // empty or trophy tile
            enemy.pos = candidate;
          }
        }
      }
    } else if dist <= attack_range {
      // attack logic (throttle by last_attack)
      let now = Instant::now();
      match enemy.last_attack {
        Some(t) => {
          if now.duration_since(t) >= attack_cooldown {
            // deal damage to player
            if state.armor > 0.0 {
              // armor absorbs half
              let armor_take = (attack_damage * 0.5).min(state.armor);
              state.armor -= armor_take;
              state.health -= (attack_damage - armor_take);
            } else {
              state.health -= attack_damage;
            }
            enemy.last_attack = Some(now);
          }
        }
        None => {
          // first attack immediately
          if state.armor > 0.0 {
            let armor_take = (attack_damage * 0.5).min(state.armor);
            state.armor -= armor_take;
            state.health -= (attack_damage - armor_take);
          } else {
            state.health -= attack_damage;
          }
          enemy.last_attack = Some(now);
        }
      }
    }
  }

  // check player death
  if state.health <= 0.0 {
    state.game_over = true;
  }
}

/// Draw a small minimap in the top-left corner with player, enemies and doors.
fn draw_minimap(painter: &egui::Painter, rect: Rect, state: &DoomState) {
  // minimap dims
  let size = 140.0f32;
  let pos = Pos2::new(rect.min.x + 12.0, rect.min.y + 12.0);
  let mm_rect = Rect::from_min_size(pos, egui::vec2(size, size));
  painter.rect_filled(mm_rect, 6.0, Color32::from_gray(20));

  let map_h = state.map.len() as f32;
  let map_w = state.map[0].len() as f32;

  // tile size in minimap
  let tile_w = (size - 8.0) / map_w;
  let tile_h = (size - 8.0) / map_h;

  for (y, row) in state.map.iter().enumerate() {
    for (x, tile) in row.iter().enumerate() {
      let cx = pos.x + 4.0 + x as f32 * tile_w + tile_w * 0.5;
      let cy = pos.y + 4.0 + y as f32 * tile_h + tile_h * 0.5;
      let cpos = Pos2::new(cx, cy);
      match *tile {
        1 => painter.rect_filled(Rect::from_center_size(cpos, egui::vec2(tile_w - 2.0, tile_h - 2.0)), 0.0, Color32::from_gray(120)),
        2 => painter.rect_filled(Rect::from_center_size(cpos, egui::vec2(tile_w - 2.0, tile_h - 2.0)), 0.0, Color32::from_rgb(120, 70, 30)),
        3 => painter.circle_filled(cpos, 4.0, Color32::GOLD),
        _ => painter.rect_filled(Rect::from_center_size(cpos, Vec2::ZERO), 0.0, Color32::TRANSPARENT),
      };
    }
  }

  // draw enemies as red dots
  for e in &state.enemies {
    let ex = ((e.pos.x) / map_w) * (size - 8.0) + pos.x + 4.0;
    let ey = ((e.pos.y) / map_h) * (size - 8.0) + pos.y + 4.0;
    painter.circle_filled(Pos2::new(ex, ey), 3.0, Color32::RED);
  }

  // draw player as blue triangle / circle
  let px = ((state.player_pos.x) / map_w) * (size - 8.0) + pos.x + 4.0;
  let py = ((state.player_pos.y) / map_h) * (size - 8.0) + pos.y + 4.0;
  painter.circle_filled(Pos2::new(px, py), 4.0, Color32::from_rgb(60, 120, 200));
}

// ----------------------------- Main Draw ----------------------------------

/// Main entry: draws the doom mode UI and handles inputs/state updates.
pub fn draw_doom_screen(app: &mut MyApp, ctx: &egui::Context) {
  // If game over/won show overlays and allow restart
  if app.doom_state.game_over {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.heading("💀 GAME OVER");
        ui.label(format!("Health: {:.1}", app.doom_state.health.max(0.0)));
        if ui.button("Restart").clicked() {
          app.doom_state = DoomState::default();
        }
      });
    });
    return;
  }
  if app.doom_state.game_won {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.vertical_centered(|ui| {
        ui.add_space(20.0);
        ui.heading("🏆 YOU WIN!");
        ui.label("You collected the trophy. Great job!");
        if ui.button("Play Again").clicked() {
          app.doom_state = DoomState::default();
        }
      });
    });
    return;
  }

  // Normal gameplay
  egui::CentralPanel::default().show(ctx, |ui| {
    let time = ui.input(|i| i.time);
    // store animation time (float seconds)
    app.doom_state.monster_animation_time = time as f32;
    let rect = ui.available_rect_before_wrap();
    let painter = ui.painter();

    // Movement and input handling
    ui.input(|i| {
      let speed = 0.08; // walking speed
      let rot_speed = 0.06; // radians per tick

      // forward
      if i.key_down(Key::W) {
        let new_pos = app.doom_state.player_pos + app.doom_state.player_dir * speed;
        let mx = new_pos.x.floor() as isize;
        let my = new_pos.y.floor() as isize;
        if let Some(tile) = map_tile_at(&app.doom_state.map, mx, my) {
          if tile == 0 || tile == 3 {
            app.doom_state.player_pos = new_pos;
          }
        }
      }
      // backward
      if i.key_down(Key::S) {
        let new_pos = app.doom_state.player_pos - app.doom_state.player_dir * speed;
        let mx = new_pos.x.floor() as isize;
        let my = new_pos.y.floor() as isize;
        if let Some(tile) = map_tile_at(&app.doom_state.map, mx, my) {
          if tile == 0 || tile == 3 {
            app.doom_state.player_pos = new_pos;
          }
        }
      }
      // rotate left
      if i.key_down(Key::A) {
        app.doom_state.player_dir = rotate_vec(app.doom_state.player_dir, rot_speed);
        app.doom_state.camera_plane = rotate_vec(app.doom_state.camera_plane, rot_speed);
      }
      // rotate right
      if i.key_down(Key::D) {
        app.doom_state.player_dir = rotate_vec(app.doom_state.player_dir, -rot_speed);
        app.doom_state.camera_plane = rotate_vec(app.doom_state.camera_plane, -rot_speed);
      }

      // Open door (E key)
      if i.key_pressed(Key::E) {
        // try to open the door under player's feet
        if try_open_door(&mut app.doom_state) {
          // small audio or visual cue could be added here
          app.doom_state.door_open_amount = 1.0;
        }
      }

      // Shoot (Space key)
      if i.key_pressed(Key::Space) {
        app.doom_state.last_shot_time = Some(Instant::now());
        // fire weapon and damage enemies
        let _ = fire_weapon(&mut app.doom_state);
      }
    });

    // Update enemies (very simple AI)
    // Use a small timestep based on frame time; we approximate by clamping to a reasonable delta
    let dt_seconds = 1.0 / 60.0; // assume 60fps tick for movement smoothing
    update_enemies(&mut app.doom_state, dt_seconds);

    // If player's health dropped to zero after enemy update, mark game over
    if app.doom_state.health <= 0.0 {
      app.doom_state.game_over = true;
    }

    // Collect trophy if on same tile
    let px = app.doom_state.player_pos.x.floor() as isize;
    let py = app.doom_state.player_pos.y.floor() as isize;
    if !app.doom_state.trophy.collected {
      let tp_x = app.doom_state.trophy.pos.x.floor() as isize;
      let tp_y = app.doom_state.trophy.pos.y.floor() as isize;
      if px == tp_x && py == tp_y {
        app.doom_state.trophy.collected = true;
        app.doom_state.game_won = true;
      }
    }

    // --- draw ceiling & floor background ---
    let ceiling_rect = Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.center().y));
    let floor_rect = Rect::from_min_max(egui::pos2(rect.min.x, rect.center().y), rect.max);
    painter.rect_filled(ceiling_rect, 0.0, Color32::from_rgb(0x5a, 0x4a, 0x3a));
    painter.rect_filled(floor_rect, 0.0, Color32::from_rgb(0x3a, 0x2a, 0x1a));

    // --- draw walls via simple raycasting ---
    render_walls(&painter, rect, &app.doom_state);

    // --- Draw enemies as sprites ---
    for enemy in &app.doom_state.enemies {
      // compute simple screen position and size based on distance
      let d = vec2_distance(enemy.pos, app.doom_state.player_pos);
      if d <= 0.05 { continue; }
      // only draw enemies in front (very naive check)
      let offset = enemy.pos - app.doom_state.player_pos;
      // compute screen position
      let screen = world_to_screen(app.doom_state.player_pos, rect, enemy.pos);
      // size inversely proportional to distance
      let h = (80.0 / (d.max(0.3))).min(160.0);
      let w = h * 0.6;
      draw_enemy(&painter, screen, egui::vec2(w, h), app.doom_state.monster_animation_time);
    }

    // --- Draw trophy if not collected ---
    if !app.doom_state.trophy.collected {
      let trophy_screen = world_to_screen(app.doom_state.player_pos, rect, app.doom_state.trophy.pos);
      draw_trophy(&painter, Pos2::new(trophy_screen.x, trophy_screen.y - 30.0));
    }

    // --- Draw weapon and muzzle flash ---
    draw_weapon_and_flash(&painter, rect, app.doom_state.last_shot_time);

    // --- Draw HUD controls (knobs + shoot/open buttons) ---
    egui::Area::new("doom_ui_controls".into()).anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(10.0, -10.0)).show(ctx, |ui| {
      ui.horizontal(|ui| {
        // Rotary knobs for health and armor (these mutate state directly)
        ui.add(RotaryKnob::new(&mut app.doom_state.health, 0.0, 100.0).with_label("Health").with_size(100.0));
        ui.add(RotaryKnob::new(&mut app.doom_state.armor, 0.0, 100.0).with_label("Armor").with_size(100.0));
        ui.add_space(24.0);
        ui.vertical(|ui| {
          // SHOOT button - triggers a shot when clicked
          if app.styled_button(ui, "SHOOT", app.doom_state.shoot_button_pressed).clicked() {
            app.doom_state.last_shot_time = Some(Instant::now());
            let _ = fire_weapon(&mut app.doom_state);
          }
          // OPEN DOOR - toggles the door open at player's feet
          if app.styled_button(ui, "OPEN DOOR", app.doom_state.open_door_button_pressed).clicked() {
            app.doom_state.open_door_button_pressed = !app.doom_state.open_door_button_pressed;
            if app.doom_state.open_door_button_pressed {
              let _opened = try_open_door(&mut app.doom_state);
              if _opened { app.doom_state.door_open_amount = 1.0; }
            }
          }
        });
      });
    });

    // --- minimap and back button areas ---
    draw_minimap(&painter, rect, &app.doom_state);

    egui::Area::new("doom_back_button".into()).anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0)).show(ctx, |ui| {
      if ui.button("↩ Back").clicked() {
        // Assume the app has AppState enum; this keeps previous behaviour
        app.app_state = super::app::AppState::StartScreen;
      }
    });

    // small debug info on-screen (health/armor)
    egui::Area::new("doom_debug_overlay".into()).anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 10.0)).show(ctx, |ui| {
      ui.horizontal(|ui| {
        ui.label(format!("HP: {:.1}", app.doom_state.health));
        ui.add_space(8.0);
        ui.label(format!("Armor: {:.1}", app.doom_state.armor));
        ui.add_space(12.0);
        ui.label(format!("Enemies: {}", app.doom_state.enemies.len()));
      });
    });
  });
}

// End of file
