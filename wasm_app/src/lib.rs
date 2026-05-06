use wasm_bindgen::prelude::*;
use game_logic::GameState;
use std::cell::RefCell;
use std::rc::Rc;

#[wasm_bindgen]
pub struct GameInstance {
    game_state: Rc<RefCell<GameState>>,
}

#[wasm_bindgen]
impl GameInstance {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GameInstance {
        #[cfg(feature = "console_error_panic_hook")]
        {
            let _ = console_error_panic_hook::set_once();
        }
        
        web_sys::console::log_1(&"🎮 Game initialized!".into());
        
        GameInstance {
            game_state: Rc::new(RefCell::new(GameState::new())),
        }
    }
    
    pub fn update(&self, delta_time: f32) {
        let mut game_state = self.game_state.borrow_mut();
        game_state.update(delta_time);
    }
    
    pub fn get_score(&self) -> u32 {
        self.game_state.borrow().score
    }
    
    pub fn get_moves(&self) -> u32 {
        self.game_state.borrow().moves
    }
    
    pub fn get_time(&self) -> f32 {
        self.game_state.borrow().time_elapsed
    }

    pub fn move_cube(&self, x: i32, y: i32, z: i32) -> bool {
        let mut game_state = self.game_state.borrow_mut();
        game_state.move_cube(0, (x, y, z))
    }

    pub fn reset(&self) {
        let mut game_state = self.game_state.borrow_mut();
        game_state.puzzle.reset();
        game_state.score = 0;
        game_state.moves = 0;
        game_state.time_elapsed = 0.0;
    }

    pub fn is_won(&self) -> bool {
        self.game_state.borrow().puzzle.is_won()
    }

    pub fn get_cube_position(&self) -> String {
        let game_state = self.game_state.borrow();
        if let Some(cube) = game_state.puzzle.cubes.get(&0) {
            format!("({}, {}, {})", cube.position.0, cube.position.1, cube.position.2)
        } else {
            "不明".to_string()
        }
    }

    pub fn get_goal_position(&self) -> String {
        let game_state = self.game_state.borrow();
        if let Some(goal) = game_state.puzzle.goal_positions.get(&0) {
            format!("({}, {}, {})", goal.0, goal.1, goal.2)
        } else {
            "未設定".to_string()
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    web_sys::console::log_1(&"✅ WASM Game Engine loaded!".into());
}
