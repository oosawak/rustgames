// DUNGEON//LOAD - 独立した WASM エントリーポイント

#[path = "../../wasm_roguelike/src/state.rs"]
mod state;
pub use state::RoguelikeGame;

use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static DUNGEONLOAD_STATE: RefCell<Option<RoguelikeGame>> = RefCell::new(None);
}

impl RoguelikeGame {
    fn equip_weapon(&mut self, idx: usize) -> bool {
        if idx < self.eq_inventory.weapons.len() {
            self.equipment.weapon = Some(self.eq_inventory.weapons[idx]);
            return true;
        }
        false
    }

    fn equip_armor(&mut self, idx: usize) -> bool {
        if idx < self.eq_inventory.armors.len() {
            self.equipment.armor = Some(self.eq_inventory.armors[idx]);
            return true;
        }
        false
    }

    fn equip_accessory(&mut self, idx: usize) -> bool {
        if idx < self.eq_inventory.accessories.len() {
            self.equipment.accessory = Some(self.eq_inventory.accessories[idx]);
            return true;
        }
        false
    }

    fn unequip_weapon(&mut self) {
        self.equipment.weapon = None;
    }

    fn unequip_armor(&mut self) {
        self.equipment.armor = None;
    }

    fn unequip_accessory(&mut self) {
        self.equipment.accessory = None;
    }
}

#[wasm_bindgen]
pub fn init_roguelike() {
    console_error_panic_hook::set_once();
    let game = RoguelikeGame::new();
    DUNGEONLOAD_STATE.with(|s| *s.borrow_mut() = Some(game));
}

#[wasm_bindgen]
pub fn start_game_roguelike() {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.start_game();
        }
    });
}

#[wasm_bindgen]
pub fn move_roguelike(action: i32) {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.move_player(action);
        }
    });
}

#[wasm_bindgen]
pub fn tick_roguelike(ts: f64) {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.tick(ts);
        }
    });
}

#[wasm_bindgen]
pub fn scene_roguelike() -> u8 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.scene.as_u8()).unwrap_or(0))
}

#[wasm_bindgen]
pub fn hp_roguelike() -> u32 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.hp).unwrap_or(0))
}

#[wasm_bindgen]
pub fn max_hp_roguelike() -> u32 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.max_hp).unwrap_or(0))
}

#[wasm_bindgen]
pub fn mp_roguelike() -> u32 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.mp).unwrap_or(0))
}

#[wasm_bindgen]
pub fn max_mp_roguelike() -> u32 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.max_mp).unwrap_or(0))
}

#[wasm_bindgen]
pub fn level_roguelike() -> u32 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.level).unwrap_or(0))
}

#[wasm_bindgen]
pub fn depth_roguelike() -> u32 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.depth).unwrap_or(0))
}

#[wasm_bindgen]
pub fn map_width_roguelike() -> i32 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.map_width).unwrap_or(0))
}

#[wasm_bindgen]
pub fn map_height_roguelike() -> i32 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.map_height).unwrap_or(0))
}

#[wasm_bindgen]
pub fn player_x_roguelike() -> i32 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.player_x).unwrap_or(0))
}

#[wasm_bindgen]
pub fn player_y_roguelike() -> i32 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.player_y).unwrap_or(0))
}

#[wasm_bindgen]
pub fn player_direction_roguelike() -> i32 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.player_direction).unwrap_or(0))
}

#[wasm_bindgen]
pub fn map_data_roguelike() -> Vec<u8> {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow().as_ref() {
            g.map
                .iter()
                .flat_map(|row| {
                    row.iter().map(|tile| match tile {
                        crate::state::TileType::Floor => 0u8,
                        crate::state::TileType::Wall => 1u8,
                        crate::state::TileType::Room => 2u8,
                        crate::state::TileType::StairDown => 3u8,
                        crate::state::TileType::StairUp => 4u8,
                    })
                })
                .collect()
        } else {
            Vec::new()
        }
    })
}

#[wasm_bindgen]
pub fn render_roguelike(canvas_id: &str, width: i32, height: i32) {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow().as_ref() {
            crate::state::render_canvas(g, canvas_id, width, height);
        }
    });
}

#[wasm_bindgen]
pub fn visited_data_roguelike() -> Vec<u8> {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow().as_ref() {
            g.visited
                .iter()
                .flat_map(|row| row.iter().map(|&v| if v { 1u8 } else { 0u8 }))
                .collect()
        } else {
            Vec::new()
        }
    })
}

#[wasm_bindgen]
pub fn enemy_count_roguelike() -> usize {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.enemies.len()).unwrap_or(0))
}

#[wasm_bindgen]
pub fn enemy_data_roguelike(index: usize) -> Vec<i32> {
    DUNGEONLOAD_STATE.with(|s| {
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
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.equipment.get_atk_bonus()).unwrap_or(0))
}

#[wasm_bindgen]
pub fn player_def_roguelike() -> u32 {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.equipment.get_def_bonus()).unwrap_or(0))
}

#[wasm_bindgen]
pub fn player_equipped_weapon_roguelike() -> i32 {
    DUNGEONLOAD_STATE.with(|s| {
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
    DUNGEONLOAD_STATE.with(|s| {
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
    DUNGEONLOAD_STATE.with(|s| {
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
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.messages.clone()).unwrap_or_default())
}

#[wasm_bindgen]
pub fn clear_messages_roguelike() {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.messages.clear();
        }
    });
}

#[wasm_bindgen]
pub fn inventory_roguelike() -> Vec<u32> {
    DUNGEONLOAD_STATE.with(|s| s.borrow().as_ref().map(|g| g.inventory.to_vec()).unwrap_or_default())
}

#[wasm_bindgen]
pub fn weapon_inventory_roguelike() -> Vec<i32> {
    DUNGEONLOAD_STATE.with(|s| {
        s.borrow()
            .as_ref()
            .map(|g| g.eq_inventory.weapons.iter().map(|w| *w as i32).collect())
            .unwrap_or_default()
    })
}

#[wasm_bindgen]
pub fn armor_inventory_roguelike() -> Vec<i32> {
    DUNGEONLOAD_STATE.with(|s| {
        s.borrow()
            .as_ref()
            .map(|g| g.eq_inventory.armors.iter().map(|w| *w as i32).collect())
            .unwrap_or_default()
    })
}

#[wasm_bindgen]
pub fn accessory_inventory_roguelike() -> Vec<i32> {
    DUNGEONLOAD_STATE.with(|s| {
        s.borrow()
            .as_ref()
            .map(|g| g.eq_inventory.accessories.iter().map(|w| *w as i32).collect())
            .unwrap_or_default()
    })
}

#[wasm_bindgen]
pub fn equip_weapon_roguelike(index: usize) -> bool {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.equip_weapon(index)
        } else {
            false
        }
    })
}

#[wasm_bindgen]
pub fn equip_armor_roguelike(index: usize) -> bool {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.equip_armor(index)
        } else {
            false
        }
    })
}

#[wasm_bindgen]
pub fn equip_accessory_roguelike(index: usize) -> bool {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.equip_accessory(index)
        } else {
            false
        }
    })
}

#[wasm_bindgen]
pub fn unequip_weapon_roguelike() {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.unequip_weapon();
        }
    });
}

#[wasm_bindgen]
pub fn unequip_armor_roguelike() {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.unequip_armor();
        }
    });
}

#[wasm_bindgen]
pub fn unequip_accessory_roguelike() {
    DUNGEONLOAD_STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() {
            g.unequip_accessory();
        }
    });
}
