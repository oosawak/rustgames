use std::cell::RefCell;
use wasm_bindgen::prelude::*;

thread_local! {
    static PUZZLE: RefCell<Option<AnimalPuzzleState>> = RefCell::new(None);
}

#[derive(Clone)]
struct AnimalPuzzleState {
    size: usize,
    zones: Vec<u8>,
    animals: Vec<u8>,
    solution: Vec<u8>,
}

impl AnimalPuzzleState {
    fn new(size: usize) -> Self {
        let len = size * size;
        Self {
            size,
            zones: vec![0; len],
            animals: vec![0; len],
            solution: vec![0; len],
        }
    }

    fn idx(&self, row: usize, col: usize) -> Option<usize> {
        if row < self.size && col < self.size {
            Some(row * self.size + col)
        } else {
            None
        }
    }

    fn is_valid_placement(&self, row: usize, col: usize) -> bool {
        let idx = match self.idx(row, col) {
            Some(idx) => idx,
            None => return false,
        };
        if self.animals[idx] != 0 {
            return false;
        }

        let target_zone = self.zones[idx];
        for r in 0..self.size {
            for c in 0..self.size {
                let i = r * self.size + c;
                if self.animals[i] == 0 {
                    continue;
                }
                if r == row || c == col {
                    return false;
                }
                if self.zones[i] == target_zone {
                    return false;
                }
                if (r as i32 - row as i32).abs() <= 1 && (c as i32 - col as i32).abs() <= 1 {
                    return false;
                }
            }
        }

        true
    }

    fn toggle(&mut self, row: usize, col: usize) -> bool {
        let idx = match self.idx(row, col) {
            Some(idx) => idx,
            None => return false,
        };

        if self.animals[idx] != 0 {
            self.animals[idx] = 0;
            return true;
        }

        if !self.is_valid_placement(row, col) {
            return false;
        }

        self.animals[idx] = 1;
        true
    }

    fn count_placed(&self) -> u32 {
        self.animals.iter().map(|v| u32::from(*v)).sum()
    }

    fn is_solved(&self) -> bool {
        self.animals == self.solution
    }
}

fn with_state<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut AnimalPuzzleState) -> R,
{
    PUZZLE.with(|state| {
        let mut slot = state.borrow_mut();
        slot.as_mut().map(f)
    })
}

#[wasm_bindgen]
pub fn animal_puzzle_init(size: usize) {
    PUZZLE.with(|state| {
        *state.borrow_mut() = Some(AnimalPuzzleState::new(size));
    });
}

#[wasm_bindgen]
pub fn animal_puzzle_size() -> usize {
    PUZZLE.with(|state| state.borrow().as_ref().map(|p| p.size).unwrap_or(0))
}

#[wasm_bindgen]
pub fn animal_puzzle_set_zones(zones: Vec<u8>) -> bool {
    with_state(|p| {
        if zones.len() != p.size * p.size {
            return false;
        }
        p.zones = zones;
        true
    })
    .unwrap_or(false)
}

#[wasm_bindgen]
pub fn animal_puzzle_set_solution(solution: Vec<u8>) -> bool {
    with_state(|p| {
        if solution.len() != p.size * p.size {
            return false;
        }
        p.solution = solution;
        true
    })
    .unwrap_or(false)
}

#[wasm_bindgen]
pub fn animal_puzzle_toggle(row: usize, col: usize) -> bool {
    with_state(|p| p.toggle(row, col)).unwrap_or(false)
}

#[wasm_bindgen]
pub fn animal_puzzle_is_valid(row: usize, col: usize) -> bool {
    PUZZLE.with(|state| {
        state
            .borrow()
            .as_ref()
            .map(|p| p.is_valid_placement(row, col))
            .unwrap_or(false)
    })
}

#[wasm_bindgen]
pub fn animal_puzzle_is_occupied(row: usize, col: usize) -> bool {
    PUZZLE.with(|state| {
        let slot = state.borrow();
        let puzzle = match slot.as_ref() {
            Some(p) => p,
            None => return false,
        };
        puzzle.idx(row, col).map(|idx| puzzle.animals[idx] != 0).unwrap_or(false)
    })
}

#[wasm_bindgen]
pub fn animal_puzzle_animals() -> Vec<u8> {
    PUZZLE.with(|state| {
        state
            .borrow()
            .as_ref()
            .map(|p| p.animals.clone())
            .unwrap_or_default()
    })
}

#[wasm_bindgen]
pub fn animal_puzzle_zones() -> Vec<u8> {
    PUZZLE.with(|state| {
        state
            .borrow()
            .as_ref()
            .map(|p| p.zones.clone())
            .unwrap_or_default()
    })
}

#[wasm_bindgen]
pub fn animal_puzzle_count() -> u32 {
    PUZZLE.with(|state| {
        state
            .borrow()
            .as_ref()
            .map(|p| p.count_placed())
            .unwrap_or(0)
    })
}

#[wasm_bindgen]
pub fn animal_puzzle_is_solved() -> bool {
    PUZZLE.with(|state| state.borrow().as_ref().map(|p| p.is_solved()).unwrap_or(false))
}

#[wasm_bindgen]
pub fn animal_puzzle_reset() {
    with_state(|p| {
        p.animals.fill(0);
    });
}
