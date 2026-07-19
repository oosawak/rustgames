// Roguelike Dungeon - 独立した WASM エントリーポイント

mod state;
pub use state::RoguelikeGame;

use wasm_bindgen::prelude::*;
use std::cell::RefCell;

thread_local! {
    static ROGUELIKE_STATE: RefCell<Option<RoguelikeGame>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn init_roguelike() {
    console_error_panic_hook::set_once();
    let game = RoguelikeGame::new();
    ROGUELIKE_STATE.with(|s| *s.borrow_mut() = Some(game));
}

#[wasm_bindgen]
pub fn start_game_roguelike() {
    ROGUELIKE_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.start_game();
        }
    });
}

#[wasm_bindgen]
pub fn move_roguelike(action: i32) {
    ROGUELIKE_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.move_player(action);
        }
    });
}


#[wasm_bindgen]
pub fn tick_roguelike(ts: f64) {
    ROGUELIKE_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.tick(ts);
        }
    });
}

#[wasm_bindgen]
pub fn scene_roguelike() -> u8 {
    ROGUELIKE_STATE.with(|s| s.borrow().as_ref().map(|g| g.scene.as_u8()).unwrap_or(0))
}

#[wasm_bindgen]
pub fn hp_roguelike() -> u32 {
    ROGUELIKE_STATE.with(|s| s.borrow().as_ref().map(|g| g.hp).unwrap_or(0))
}

#[wasm_bindgen]
pub fn max_hp_roguelike() -> u32 {
    ROGUELIKE_STATE.with(|s| s.borrow().as_ref().map(|g| g.max_hp).unwrap_or(0))
}

#[wasm_bindgen]
pub fn mp_roguelike() -> u32 {
    ROGUELIKE_STATE.with(|s| s.borrow().as_ref().map(|g| g.mp).unwrap_or(0))
}

#[wasm_bindgen]
pub fn max_mp_roguelike() -> u32 {
    ROGUELIKE_STATE.with(|s| s.borrow().as_ref().map(|g| g.max_mp).unwrap_or(0))
}

#[wasm_bindgen]
pub fn level_roguelike() -> u32 {
    ROGUELIKE_STATE.with(|s| s.borrow().as_ref().map(|g| g.level).unwrap_or(0))
}

#[wasm_bindgen]
pub fn depth_roguelike() -> u32 {
    ROGUELIKE_STATE.with(|s| s.borrow().as_ref().map(|g| g.depth).unwrap_or(0))
}

#[wasm_bindgen]
pub fn map_width_roguelike() -> i32 {
    ROGUELIKE_STATE.with(|s| s.borrow().as_ref().map(|g| g.map_width).unwrap_or(0))
}

#[wasm_bindgen]
pub fn map_height_roguelike() -> i32 {
    ROGUELIKE_STATE.with(|s| s.borrow().as_ref().map(|g| g.map_height).unwrap_or(0))
}

#[wasm_bindgen]
pub fn player_x_roguelike() -> i32 {
    ROGUELIKE_STATE.with(|s| s.borrow().as_ref().map(|g| g.player_x).unwrap_or(0))
}

#[wasm_bindgen]
pub fn player_y_roguelike() -> i32 {
    ROGUELIKE_STATE.with(|s| s.borrow().as_ref().map(|g| g.player_y).unwrap_or(0))
}

#[wasm_bindgen]
pub fn player_direction_roguelike() -> i32 {
    ROGUELIKE_STATE.with(|s| s.borrow().as_ref().map(|g| g.player_direction).unwrap_or(0))
}

#[wasm_bindgen]
pub fn map_data_roguelike() -> Vec<u8> {
    ROGUELIKE_STATE.with(|s| {
        if let Some(g) = s.borrow().as_ref() {
            // Convert map to bytes (0=Floor, 1=Wall, 2=Room, 3=StairDown, 4=StairUp)
            g.map.iter().flat_map(|row| {
                row.iter().map(|tile| match tile {
                    crate::state::TileType::Floor => 0u8,
                    crate::state::TileType::Wall => 1u8,
                    crate::state::TileType::Room => 2u8,
                    crate::state::TileType::StairDown => 3u8,
                    crate::state::TileType::StairUp => 4u8,
                })
            }).collect()
        } else {
            Vec::new()
        }
    })
}

#[wasm_bindgen]
pub fn render_roguelike(canvas_id: &str, width: i32, height: i32) {
    ROGUELIKE_STATE.with(|s| {
        if let Some(g) = s.borrow().as_ref() {
            crate::state::render_canvas(g, canvas_id, width, height);
        }
    });
}

#[wasm_bindgen]
pub fn visited_data_roguelike() -> Vec<u8> {
    ROGUELIKE_STATE.with(|s| {
        if let Some(g) = s.borrow().as_ref() {
            g.visited.iter().flat_map(|row| {
                row.iter().map(|&v| if v { 1u8 } else { 0u8 })
            }).collect()
        } else {
            Vec::new()
        }
    })
}

#[wasm_bindgen]
pub fn enemy_count_roguelike() -> usize {
    ROGUELIKE_STATE.with(|s| {
        s.borrow().as_ref().map(|g| g.enemies.len()).unwrap_or(0)
    })
}

#[wasm_bindgen]
pub fn enemy_data_roguelike(index: usize) -> Vec<i32> {
    ROGUELIKE_STATE.with(|s| {
        if let Some(g) = s.borrow().as_ref() {
            if index < g.enemies.len() {
                let e = &g.enemies[index];
                vec![e.x, e.y, e.hp as i32]
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    })
}

#[wasm_bindgen]
pub fn player_atk_roguelike() -> u32 {
    ROGUELIKE_STATE.with(|s| {
        s.borrow().as_ref().map(|g| g.equipment.get_atk_bonus()).unwrap_or(0)
    })
}

#[wasm_bindgen]
pub fn player_def_roguelike() -> u32 {
    ROGUELIKE_STATE.with(|s| {
        s.borrow().as_ref().map(|g| g.equipment.get_def_bonus()).unwrap_or(0)
    })
}

#[wasm_bindgen]
pub fn player_equipped_weapon_roguelike() -> i32 {
    ROGUELIKE_STATE.with(|s| {
        if let Some(g) = s.borrow().as_ref() {
            match g.equipment.weapon {
                Some(w) => w as i32,
                None => -1,
            }
        } else {
            -1
        }
    })
}

#[wasm_bindgen]
pub fn player_equipped_armor_roguelike() -> i32 {
    ROGUELIKE_STATE.with(|s| {
        if let Some(g) = s.borrow().as_ref() {
            match g.equipment.armor {
                Some(a) => a as i32,
                None => -1,
            }
        } else {
            -1
        }
    })
}

#[wasm_bindgen]
pub fn player_equipped_accessory_roguelike() -> i32 {
    ROGUELIKE_STATE.with(|s| {
        if let Some(g) = s.borrow().as_ref() {
            match g.equipment.accessory {
                Some(a) => a as i32,
                None => -1,
            }
        } else {
            -1
        }
    })
}

#[wasm_bindgen]
pub fn messages_roguelike() -> Vec<String> {
    ROGUELIKE_STATE.with(|s| {
        s.borrow().as_ref().map(|g| g.messages.clone()).unwrap_or_default()
    })
}

#[wasm_bindgen]
pub fn clear_messages_roguelike() {
    ROGUELIKE_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.messages.clear();
        }
    });
}
