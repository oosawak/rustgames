use wasm_bindgen::prelude::*;
use engine::{Engine, RendererConfig};
use game_logic::GameState;
use std::cell::RefCell;
use std::rc::Rc;

#[wasm_bindgen]
pub struct GameInstance {
    engine: Rc<RefCell<Option<Engine>>>,
    game_state: Rc<RefCell<GameState>>,
}

#[wasm_bindgen]
impl GameInstance {
    #[wasm_bindgen(constructor)]
    pub fn new(_canvas_id: &str) -> Result<GameInstance, JsValue> {
        // Set up logging
        #[cfg(feature = "console_error_panic_hook")]
        {
            let _ = console_error_panic_hook::set_once();
        }
        
        web_sys::console::log_1(&"Initializing game...".into());
        
        Ok(GameInstance {
            engine: Rc::new(RefCell::new(None)),
            game_state: Rc::new(RefCell::new(GameState::new())),
        })
    }
    
    pub fn init(&self) -> Result<(), JsValue> {
        web_sys::console::log_1(&"Game initialized".into());
        Ok(())
    }
    
    pub fn update(&self, delta_time: f32) -> Result<(), JsValue> {
        let mut game_state = self.game_state.borrow_mut();
        game_state.update(delta_time);
        Ok(())
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
}

#[wasm_bindgen(start)]
pub fn start() {
    web_sys::console::log_1(&"WASM Game Engine loaded!".into());
}
