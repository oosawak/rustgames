// エントリーポイント: モジュール宣言と #[wasm_bindgen] エクスポート関数のみを含む薄いファイル

pub mod constants;
pub mod shader;
pub mod math;
pub mod particle;
pub mod maze;
pub mod geometry;
pub mod gpu;
pub mod game;
pub mod audio;
pub mod camera;
pub mod engine;
pub mod input;
pub mod enemy;
pub mod theme;
pub mod scene;

use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use game::GameState;

thread_local! {
    static STATE: RefCell<Option<GameState>> = RefCell::new(None);
}

#[wasm_bindgen]
pub async fn init_maze3d(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let s = GameState::new(canvas_id).await.map_err(|e| JsValue::from_str(&e))?;
    STATE.with(|st| *st.borrow_mut() = Some(s));
    Ok(())
}

#[wasm_bindgen] pub fn tick_maze3d(ts:f64){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.tick(ts);}}); }
#[wasm_bindgen] pub fn move_maze3d(a:i32){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.act(a);}}); }
#[wasm_bindgen] pub fn next_level_maze3d(){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.next_level();}}); }
#[wasm_bindgen] pub fn reset_maze3d(){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.reset();}}); }
#[wasm_bindgen] pub fn start_game_maze3d(){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.start_game();}}); }
#[wasm_bindgen] pub fn scene_maze3d() -> u8 { STATE.with(|s| s.borrow().as_ref().map(|g| g.scene.as_u8()).unwrap_or(0)) }
#[wasm_bindgen] pub fn steps_maze3d()->u32       { STATE.with(|s|s.borrow().as_ref().map(|g|g.steps).unwrap_or(0)) }
#[wasm_bindgen] pub fn total_steps_maze3d()->u32 { STATE.with(|s|s.borrow().as_ref().map(|g|g.total_steps).unwrap_or(0)) }
#[wasm_bindgen] pub fn level_maze3d()->u32       { STATE.with(|s|s.borrow().as_ref().map(|g|g.level).unwrap_or(1)) }
#[wasm_bindgen] pub fn level_clear_maze3d()->bool{ STATE.with(|s|s.borrow().as_ref().map(|g|g.level_clear).unwrap_or(false)) }
#[wasm_bindgen] pub fn warp_maze3d()->f32        { STATE.with(|s|s.borrow().as_ref().map(|g|g.warp_amount()).unwrap_or(0.0)) }
#[wasm_bindgen] pub fn warp_done_maze3d()->bool  { STATE.with(|s|s.borrow().as_ref().map(|g|g.level_clear && g.warp_timer>=1.5).unwrap_or(false)) }

/// 音声イベントフラグ (0=なし 1=足音 2=壁衝突 3=レベルクリア 4=ゴール付近)
#[wasm_bindgen]
pub fn audio_event_maze3d() -> u8 {
    STATE.with(|s| s.borrow_mut().as_mut().map(|g| g.audio.consume() as u8).unwrap_or(0))
}

/// 足音の左右パリティ (true=左足)
#[wasm_bindgen]
pub fn audio_step_parity_maze3d() -> bool {
    STATE.with(|s| s.borrow().as_ref().map(|g| g.audio.step_parity).unwrap_or(false))
}

#[wasm_bindgen]
pub fn game_over_maze3d() -> bool {
    STATE.with(|s| s.borrow().as_ref().map(|g| g.game_over).unwrap_or(false))
}

#[wasm_bindgen]
pub fn theme_name_maze3d() -> String {
    STATE.with(|s| s.borrow().as_ref()
        .map(|g| crate::theme::get_theme(g.level).name.to_string())
        .unwrap_or_default())
}

#[wasm_bindgen]
pub fn enemy_x_maze3d() -> f32 {
    STATE.with(|s| s.borrow().as_ref().map(|g| g.enemy.vis_x).unwrap_or(-1.0))
}

#[wasm_bindgen]
pub fn enemy_z_maze3d() -> f32 {
    STATE.with(|s| s.borrow().as_ref().map(|g| g.enemy.vis_z).unwrap_or(-1.0))
}

// ── ミニマップ用エクスポート ───────────────────────────────────────────────────
// 迷路セルフラグ (N=1,E=2,S=4,W=8) を MAZE_W×MAZE_H の平坦なVec<u8>で返す
#[wasm_bindgen] pub fn maze_data_maze3d() -> Vec<u8> {
    STATE.with(|s| s.borrow().as_ref().map(|g| g.maze.cells.to_vec()).unwrap_or_default())
}
#[wasm_bindgen] pub fn player_x_maze3d()      -> u32 { STATE.with(|s|s.borrow().as_ref().map(|g|g.px as u32).unwrap_or(0)) }
#[wasm_bindgen] pub fn player_z_maze3d()      -> u32 { STATE.with(|s|s.borrow().as_ref().map(|g|g.pz as u32).unwrap_or(0)) }
#[wasm_bindgen] pub fn player_facing_maze3d() -> u8  { STATE.with(|s|s.borrow().as_ref().map(|g|g.facing).unwrap_or(4)) }
