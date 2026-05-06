use game_logic::{GameState, puzzle::PuzzleState};
use std::sync::{Arc, Mutex};
use std::ptr;
use log::info;

// グローバルゲーム状態（スレッドセーフ）
static mut GAME_STATE: Option<Arc<Mutex<GameState>>> = None;

// C互換 ゲーム状態構造体
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GameStateFFI {
    pub score: u32,
    pub moves: u32,
    pub time_elapsed: f32,
    pub is_won: bool,
    pub puzzle_move_count: u32,
}

impl From<&GameState> for GameStateFFI {
    fn from(state: &GameState) -> Self {
        GameStateFFI {
            score: state.score,
            moves: state.moves,
            time_elapsed: state.time_elapsed,
            is_won: state.puzzle.is_won(),
            puzzle_move_count: state.puzzle.move_count,
        }
    }
}

/// ゲーム初期化
#[no_mangle]
pub extern "C" fn game_initialize(width: u32, height: u32) -> i32 {
    unsafe {
        if GAME_STATE.is_some() {
            return -1; // Already initialized
        }
        
        GAME_STATE = Some(Arc::new(Mutex::new(GameState::new())));
        info!("Game initialized with size {}x{}", width, height);
        0
    }
}

/// ゲーム更新
#[no_mangle]
pub extern "C" fn game_update(delta_time: f32) -> i32 {
    unsafe {
        match &GAME_STATE {
            Some(state) => {
                let mut game = state.lock().unwrap();
                game.update(delta_time);
                0
            }
            None => -1, // Not initialized
        }
    }
}

/// ゲーム状態取得
#[no_mangle]
pub extern "C" fn game_get_state() -> GameStateFFI {
    unsafe {
        match &GAME_STATE {
            Some(state) => {
                let game = state.lock().unwrap();
                GameStateFFI::from(&*game)
            }
            None => GameStateFFI {
                score: 0,
                moves: 0,
                time_elapsed: 0.0,
                is_won: false,
                puzzle_move_count: 0,
            },
        }
    }
}

/// 立方体を移動
#[no_mangle]
pub extern "C" fn game_move_cube(cube_id: u32, x: i32, y: i32, z: i32) -> i32 {
    unsafe {
        match &GAME_STATE {
            Some(state) => {
                let mut game = state.lock().unwrap();
                let result = game.puzzle.move_cube(cube_id, (x, y, z));
                if result {
                    game.moves += 1;
                    0
                } else {
                    -1 // Invalid move
                }
            }
            None => -1, // Not initialized
        }
    }
}

/// パズルリセット
#[no_mangle]
pub extern "C" fn game_reset() -> i32 {
    unsafe {
        match &GAME_STATE {
            Some(state) => {
                let mut game = state.lock().unwrap();
                game.reset();
                0
            }
            None => -1, // Not initialized
        }
    }
}

/// ゲーム終了
#[no_mangle]
pub extern "C" fn game_cleanup() -> i32 {
    unsafe {
        GAME_STATE = None;
        info!("Game cleaned up");
        0
    }
}

/// スコア加算
#[no_mangle]
pub extern "C" fn game_add_score(points: u32) -> i32 {
    unsafe {
        match &GAME_STATE {
            Some(state) => {
                let mut game = state.lock().unwrap();
                game.score += points;
                0
            }
            None => -1, // Not initialized
        }
    }
}

/// パーティクルエミット
#[no_mangle]
pub extern "C" fn game_emit_particles(x: f32, y: f32, z: f32, count: u32) -> i32 {
    unsafe {
        match &GAME_STATE {
            Some(state) => {
                let mut game = state.lock().unwrap();
                let position = cgmath::Vector3::new(x, y, z);
                game.particles.emit_burst(
                    position,
                    count as usize,
                    2.0,
                    1.0,
                    (1.0, 1.0, 0.0, 1.0),
                );
                0
            }
            None => -1, // Not initialized
        }
    }
}

// Version info
#[no_mangle]
pub extern "C" fn game_get_version() -> *const u8 {
    b"Rust Game Engine v0.1.0\0".as_ptr()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let result = unsafe { game_initialize(1280, 720) };
        assert_eq!(result, 0);
        let result = unsafe { game_cleanup() };
        assert_eq!(result, 0);
    }
}
