// Entry point for penpen project
pub mod penpen;
pub mod penpen2;
pub mod math;
pub mod gpu;
pub mod geometry;
pub mod audio_tool;
pub mod constants;
pub mod shader;
pub mod maze;
pub mod particle;
pub mod theme;
pub mod engine;
pub mod scene;
pub mod storage;
pub mod font;
pub mod input;
pub mod enemy;
pub mod camera;
pub mod audio;
pub mod game;

use wasm_bindgen::prelude::*;
use std::cell::RefCell;

// ── PenPen Glide 3D ─────────────────────────────────────────────────────────
thread_local! {
    static PENPEN: RefCell<Option<penpen::PenPenGame>> = RefCell::new(None);
}

#[wasm_bindgen]
pub async fn init_penpen(canvas_id: &str) {
    console_error_panic_hook::set_once();
    let game = penpen::PenPenGame::new(canvas_id).await.unwrap();
    PENPEN.with(|s| *s.borrow_mut() = Some(game));
}

#[wasm_bindgen]
pub async fn init_penpen_demo(canvas_id: &str) {
    console_error_panic_hook::set_once();
    let game = penpen::PenPenGame::new_demo(canvas_id).await.unwrap();
    PENPEN.with(|s| *s.borrow_mut() = Some(game));
}

#[wasm_bindgen]
pub fn tick_penpen(ts: f64) {
    PENPEN.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.tick(ts);
            let aspect = g.gpu.width as f32 / g.gpu.height as f32;
            let (verts, idxs) = penpen::build_penpen_scene(g);
            let uni = penpen::build_penpen_uni(g, aspect);
            g.gpu.render(&verts, &idxs, &uni);
        }
    });
}

#[wasm_bindgen] pub fn resize_penpen(w: u32, h: u32) { PENPEN.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.gpu.resize(w, h); }); }
#[wasm_bindgen] pub fn start_penpen() { PENPEN.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.start(); }); }
#[wasm_bindgen] pub fn move_penpen(dx: f32) { PENPEN.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.set_move_input(dx); }); }
#[wasm_bindgen] pub fn jump_penpen(on: bool) { PENPEN.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.set_jump_input(on); }); }
#[wasm_bindgen] pub fn act_penpen(key: i32) { PENPEN.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.act(key); }); }
#[wasm_bindgen] pub fn scene_penpen() -> u8 { PENPEN.with(|s| s.borrow().as_ref().map(|g| g.scene as u8).unwrap_or(0)) }
#[wasm_bindgen] pub fn score_penpen() -> u32 { PENPEN.with(|s| s.borrow().as_ref().map(|g| g.score).unwrap_or(0)) }
#[wasm_bindgen] pub fn speed_penpen() -> f32 { PENPEN.with(|s| s.borrow().as_ref().map(|g| g.player_speed).unwrap_or(0.0)) }
#[wasm_bindgen] pub fn hp_penpen() -> i32 { PENPEN.with(|s| s.borrow().as_ref().map(|g| g.player_hp).unwrap_or(0)) }
#[wasm_bindgen] pub fn max_hp_penpen() -> i32 { PENPEN.with(|s| s.borrow().as_ref().map(|g| g.player_max_hp).unwrap_or(5)) }
#[wasm_bindgen] pub fn level_penpen() -> u32 { PENPEN.with(|s| s.borrow().as_ref().map(|g| g.level).unwrap_or(1)) }
#[wasm_bindgen] pub fn fish_count_penpen() -> u32 { PENPEN.with(|s| s.borrow().as_ref().map(|g| g.fish_collected).unwrap_or(0)) }
#[wasm_bindgen] pub fn audio_event_penpen() -> u8 { PENPEN.with(|s| if let Some(g) = s.borrow_mut().as_mut() { let ev = g.audio_event; g.audio_event = 0; ev } else { 0 }) }
#[wasm_bindgen] pub fn sound_def_penpen(event: u8) -> String { penpen::sound_def(event) }
#[wasm_bindgen] pub fn progress_penpen() -> f32 { PENPEN.with(|s| s.borrow().as_ref().map(|g| (g.player_z / g.track_length).clamp(0.0, 1.0)).unwrap_or(0.0)) }
#[wasm_bindgen] pub fn switch_camera_penpen() { PENPEN.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.camera_mode = (g.camera_mode + 1) % 3; }); }
#[wasm_bindgen] pub fn camera_name_penpen() -> String { PENPEN.with(|s| s.borrow().as_ref().map(|g| match g.camera_mode { 1 => "TOP".to_string(), 2 => "FPS".to_string(), _ => "TPS".to_string() }).unwrap_or_else(|| "TPS".to_string())) }
#[wasm_bindgen] pub fn next_level_penpen() { PENPEN.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.next_level(); }); }
#[wasm_bindgen] pub fn reset_game_penpen() { PENPEN.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.reset_game(); }); }

// ─── PenPen Glide 2 ──────────────────────────────────────────────────────────
thread_local! {
    static PENPEN2: RefCell<Option<penpen2::PenPenGame>> = RefCell::new(None);
}
#[wasm_bindgen] pub async fn init_penpen2(canvas_id: &str) { console_error_panic_hook::set_once(); let game = penpen2::PenPenGame::new(canvas_id).await.unwrap(); PENPEN2.with(|s| *s.borrow_mut() = Some(game)); }
#[wasm_bindgen] pub fn tick_penpen2(ts: f64) { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.tick(ts); let aspect = g.gpu.width as f32 / g.gpu.height as f32; let (verts, idxs) = penpen2::build_penpen_scene(g); let uni = penpen2::build_penpen_uni(g, aspect); g.gpu.render(&verts, &idxs, &uni); }); }
#[wasm_bindgen] pub fn resize_penpen2(w: u32, h: u32) { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.gpu.resize(w, h); }); }
#[wasm_bindgen] pub fn start_penpen2() { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.start(); }); }
#[wasm_bindgen] pub fn move_penpen2(dx: f32) { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.set_move_input(dx); }); }
#[wasm_bindgen] pub fn jump_penpen2(on: bool) { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.set_jump_input(on); }); }
#[wasm_bindgen] pub fn pull_penpen2(on: bool) { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.set_pull_input(on); }); }
#[wasm_bindgen] pub fn set_accel_input_penpen2(on: bool) { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.set_accel_input(on); }); }
#[wasm_bindgen] pub fn set_brake_input_penpen2(on: bool) { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.set_brake_input(on); }); }
#[wasm_bindgen] pub fn act_penpen2(key: i32) { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.act(key); }); }
#[wasm_bindgen] pub fn pull_dist_penpen2() -> f32 { PENPEN2.with(|s| s.borrow().as_ref().map(|g| g.pull_dist).unwrap_or(0.0)) }
#[wasm_bindgen] pub fn scene_penpen2() -> u8 { PENPEN2.with(|s| s.borrow().as_ref().map(|g| g.scene as u8).unwrap_or(0)) }
#[wasm_bindgen] pub fn score_penpen2() -> u32 { PENPEN2.with(|s| s.borrow().as_ref().map(|g| g.score).unwrap_or(0)) }
#[wasm_bindgen] pub fn speed_penpen2() -> f32 { PENPEN2.with(|s| s.borrow().as_ref().map(|g| g.player_speed).unwrap_or(0.0)) }
#[wasm_bindgen] pub fn hp_penpen2() -> i32 { PENPEN2.with(|s| s.borrow().as_ref().map(|g| g.player_hp).unwrap_or(0)) }
#[wasm_bindgen] pub fn max_hp_penpen2() -> i32 { PENPEN2.with(|s| s.borrow().as_ref().map(|g| g.player_max_hp).unwrap_or(5)) }
#[wasm_bindgen] pub fn level_penpen2() -> u32 { PENPEN2.with(|s| s.borrow().as_ref().map(|g| g.level).unwrap_or(1)) }
#[wasm_bindgen] pub fn fish_count_penpen2() -> u32 { PENPEN2.with(|s| s.borrow().as_ref().map(|g| g.fish_collected).unwrap_or(0)) }
#[wasm_bindgen] pub fn audio_event_penpen2() -> u8 { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { let ev = g.audio_event; g.audio_event = 0; ev } else { 0 }) }
#[wasm_bindgen] pub fn sound_def_penpen2(event: u8) -> String { penpen2::sound_def(event) }
#[wasm_bindgen] pub fn progress_penpen2() -> f32 { PENPEN2.with(|s| s.borrow().as_ref().map(|g| (g.player_z / g.track_length).clamp(0.0, 1.0)).unwrap_or(0.0)) }
#[wasm_bindgen] pub fn switch_camera_penpen2() { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.camera_mode = (g.camera_mode + 1) % 3; }); }
#[wasm_bindgen] pub fn camera_name_penpen2() -> String { PENPEN2.with(|s| s.borrow().as_ref().map(|g| match g.camera_mode { 1 => "TOP".to_string(), 2 => "FPS".to_string(), _ => "TPS".to_string() }).unwrap_or_else(|| "TPS".to_string())) }
#[wasm_bindgen] pub fn next_level_penpen2() { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.next_level(); }); }
#[wasm_bindgen] pub fn reset_game_penpen2() { PENPEN2.with(|s| if let Some(g) = s.borrow_mut().as_mut() { g.reset_game(); }); }
