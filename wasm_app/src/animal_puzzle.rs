use serde::Serialize;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;

const ANIMAL_KIND_COUNT: u8 = 9;

thread_local! {
    static PUZZLE: RefCell<Option<AnimalPuzzleState>> = RefCell::new(None);
}

#[derive(Clone, Serialize)]
pub struct AnimalPuzzleSpec {
    size: usize,
    difficulty: u8,
    animal_kind: u8,
    zones: Vec<u8>,
    solution: Vec<u8>,
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

    fn from_spec(spec: &AnimalPuzzleSpec) -> Self {
        let len = spec.size * spec.size;
        Self {
            size: spec.size,
            zones: spec.zones.clone(),
            animals: vec![0; len],
            solution: spec.solution.clone(),
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

#[derive(Clone, Copy)]
struct Rng(u64);

impl Rng {
    fn new() -> Self {
        let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
        Self((seed | 1) ^ 0x9e3779b97f4a7c15)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn gen_range(&mut self, upper: usize) -> usize {
        if upper == 0 {
            0
        } else {
            (self.next_u64() % upper as u64) as usize
        }
    }

    fn gen_signed(&mut self, spread: i32) -> i32 {
        if spread <= 0 {
            0
        } else {
            let width = (spread as u64) * 2 + 1;
            (self.next_u64() % width) as i32 - spread
        }
    }

    fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.gen_range(i + 1);
            slice.swap(i, j);
        }
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

fn set_state_from_spec(spec: &AnimalPuzzleSpec) {
    PUZZLE.with(|state| {
        *state.borrow_mut() = Some(AnimalPuzzleState::from_spec(spec));
    });
}

fn is_adjacent(a_row: usize, a_col: usize, b_row: usize, b_col: usize) -> bool {
    (a_row as i32 - b_row as i32).abs() <= 1 && (a_col as i32 - b_col as i32).abs() <= 1
}

fn solution_mask_from_cols(cols: &[usize], size: usize) -> Vec<u8> {
    let mut mask = vec![0; size * size];
    for (row, &col) in cols.iter().enumerate() {
        mask[row * size + col] = 1;
    }
    mask
}

fn generate_solution_cols(size: usize, rng: &mut Rng) -> Option<Vec<usize>> {
    let mut cols = vec![usize::MAX; size];
    let mut used_cols = vec![false; size];

    fn backtrack(
        row: usize,
        size: usize,
        rng: &mut Rng,
        cols: &mut [usize],
        used_cols: &mut [bool],
    ) -> bool {
        if row == size {
            return true;
        }

        let mut candidates: Vec<usize> = (0..size).collect();
        rng.shuffle(&mut candidates);

        for col in candidates {
            if used_cols[col] {
                continue;
            }

            let mut ok = true;
            for prev_row in 0..row {
                if is_adjacent(row, col, prev_row, cols[prev_row]) {
                    ok = false;
                    break;
                }
            }
            if !ok {
                continue;
            }

            used_cols[col] = true;
            cols[row] = col;
            if backtrack(row + 1, size, rng, cols, used_cols) {
                return true;
            }
            used_cols[col] = false;
        }

        false
    }

    if backtrack(0, size, rng, &mut cols, &mut used_cols) {
        Some(cols)
    } else {
        None
    }
}

fn target_zone_sizes(size: usize, difficulty: u8, rng: &mut Rng) -> Vec<usize> {
    let total = size * size;
    let base = total / size;
    let min_size = if size >= 5 { 3 } else { 2 };
    let max_size = total - min_size * (size - 1);
    let spread = match difficulty {
        0 | 1 => 0,
        2 => 2,
        _ => 4,
    };

    let mut targets = vec![base; size];
    for target in &mut targets {
        let raw = (*target as i32 + rng.gen_signed(spread))
            .clamp(min_size as i32, max_size as i32);
        *target = raw as usize;
    }

    let singleton_count = match difficulty {
        0 | 1 => 2,
        2 => 1,
        _ => 1,
    }
    .min(size);
    for target in targets.iter_mut().take(singleton_count) {
        *target = 1;
    }

    let mut sum = targets.iter().sum::<usize>();
    while sum < total {
        let idx = rng.gen_range(size);
        if targets[idx] < max_size {
            targets[idx] += 1;
            sum += 1;
        }
    }
    while sum > total {
        let idx = rng.gen_range(size);
        if targets[idx] > min_size {
            targets[idx] -= 1;
            sum -= 1;
        }
    }

    targets
}

fn difficulty_profile(difficulty: u8) -> (i32, i32, usize) {
    match difficulty {
        0 | 1 => (3, 0, 120),
        2 => (2, 2, 220),
        _ => (0, 4, 360),
    }
}

fn candidate_zones_for_cell(
    idx: usize,
    size: usize,
    zones: &[u8],
    assigned: &[bool],
) -> Vec<usize> {
    let row = idx / size;
    let col = idx % size;
    let mut candidates = Vec::new();
    let mut seen = [false; 16];

    let mut push = |nidx: Option<usize>| {
        if let Some(nidx) = nidx {
            if assigned[nidx] {
                let zone = zones[nidx] as usize;
                if zone < seen.len() && !seen[zone] {
                    seen[zone] = true;
                    candidates.push(zone);
                }
            }
        }
    };

    push((row > 0).then_some((row - 1) * size + col));
    push((row + 1 < size).then_some((row + 1) * size + col));
    push((col > 0).then_some(row * size + col - 1));
    push((col + 1 < size).then_some(row * size + col + 1));

    candidates
}

fn generate_zones(size: usize, solution_cols: &[usize], difficulty: u8, rng: &mut Rng) -> Vec<u8> {
    let total = size * size;
    let mut zones = vec![u8::MAX; total];
    let mut assigned = vec![false; total];
    let mut counts = vec![0usize; size];
    let targets = target_zone_sizes(size, difficulty, rng);
    let mut frontier = Vec::new();
    let mut frontier_mark = vec![false; total];
    let (contiguity_weight, noise_spread, _) = difficulty_profile(difficulty);

    let push_frontier = |idx: usize, frontier: &mut Vec<usize>, frontier_mark: &mut [bool], assigned: &[bool]| {
        if idx < total && !assigned[idx] && !frontier_mark[idx] {
            frontier_mark[idx] = true;
            frontier.push(idx);
        }
    };

    for (zone, &col) in solution_cols.iter().enumerate() {
        let idx = zone * size + col;
        zones[idx] = zone as u8;
        assigned[idx] = true;
        counts[zone] = 1;

        let row = zone;
        if row > 0 {
            push_frontier((row - 1) * size + col, &mut frontier, &mut frontier_mark, &assigned);
        }
        if row + 1 < size {
            push_frontier((row + 1) * size + col, &mut frontier, &mut frontier_mark, &assigned);
        }
        if col > 0 {
            push_frontier(row * size + col - 1, &mut frontier, &mut frontier_mark, &assigned);
        }
        if col + 1 < size {
            push_frontier(row * size + col + 1, &mut frontier, &mut frontier_mark, &assigned);
        }
    }

    let mut assigned_count = size;
    while assigned_count < total {
        if frontier.is_empty() {
            for idx in 0..total {
                if !assigned[idx] && candidate_zones_for_cell(idx, size, &zones, &assigned).is_empty() == false {
                    push_frontier(idx, &mut frontier, &mut frontier_mark, &assigned);
                }
            }
            if frontier.is_empty() {
                break;
            }
        }

        let frontier_idx = rng.gen_range(frontier.len());
        let idx = frontier.swap_remove(frontier_idx);
        frontier_mark[idx] = false;
        if assigned[idx] {
            continue;
        }

        let candidates = candidate_zones_for_cell(idx, size, &zones, &assigned);
        if candidates.is_empty() {
            continue;
        }

        let mut best_zone = candidates[0];
        let mut best_score = i32::MIN;

        for zone in candidates {
            let deficit = targets[zone] as i32 - counts[zone] as i32;
            let noise = rng.gen_signed(noise_spread);
            let edge_bias = if idx % size == 0 || idx % size + 1 == size || idx / size == 0 || idx / size + 1 == size {
                1
            } else {
                0
            };
            let mut same_zone_neighbors = 0;
            let row = idx / size;
            let col = idx % size;
            if row > 0 && assigned[(row - 1) * size + col] && zones[(row - 1) * size + col] as usize == zone {
                same_zone_neighbors += 1;
            }
            if row + 1 < size && assigned[(row + 1) * size + col] && zones[(row + 1) * size + col] as usize == zone {
                same_zone_neighbors += 1;
            }
            if col > 0 && assigned[row * size + col - 1] && zones[row * size + col - 1] as usize == zone {
                same_zone_neighbors += 1;
            }
            if col + 1 < size && assigned[row * size + col + 1] && zones[row * size + col + 1] as usize == zone {
                same_zone_neighbors += 1;
            }
            let score = deficit * 10 + same_zone_neighbors * contiguity_weight + noise + edge_bias;
            if score > best_score {
                best_score = score;
                best_zone = zone;
            }
        }

        zones[idx] = best_zone as u8;
        counts[best_zone] += 1;
        assigned[idx] = true;
        assigned_count += 1;

        let row = idx / size;
        let col = idx % size;
        if row > 0 {
            push_frontier((row - 1) * size + col, &mut frontier, &mut frontier_mark, &assigned);
        }
        if row + 1 < size {
            push_frontier((row + 1) * size + col, &mut frontier, &mut frontier_mark, &assigned);
        }
        if col > 0 {
            push_frontier(row * size + col - 1, &mut frontier, &mut frontier_mark, &assigned);
        }
        if col + 1 < size {
            push_frontier(row * size + col + 1, &mut frontier, &mut frontier_mark, &assigned);
        }
    }

    zones
}

fn count_solutions(zones: &[u8], size: usize, limit: usize) -> usize {
    let mut used_cols = vec![false; size];
    let mut used_zones = vec![false; size];
    let mut placement = vec![usize::MAX; size];

    fn backtrack(
        row: usize,
        size: usize,
        zones: &[u8],
        used_cols: &mut [bool],
        used_zones: &mut [bool],
        placement: &mut [usize],
        limit: usize,
        count: &mut usize,
    ) {
        if *count >= limit || row == size {
            if row == size {
                *count += 1;
            }
            return;
        }

        for col in 0..size {
            if used_cols[col] {
                continue;
            }

            let zone = zones[row * size + col] as usize;
            if zone >= used_zones.len() || used_zones[zone] {
                continue;
            }

            let mut ok = true;
            for prev_row in 0..row {
                let prev_col = placement[prev_row];
                if is_adjacent(row, col, prev_row, prev_col) {
                    ok = false;
                    break;
                }
            }
            if !ok {
                continue;
            }

            used_cols[col] = true;
            used_zones[zone] = true;
            placement[row] = col;

            backtrack(
                row + 1,
                size,
                zones,
                used_cols,
                used_zones,
                placement,
                limit,
                count,
            );

            placement[row] = usize::MAX;
            used_zones[zone] = false;
            used_cols[col] = false;

            if *count >= limit {
                return;
            }
        }
    }

    let mut count = 0;
    backtrack(
        0,
        size,
        zones,
        &mut used_cols,
        &mut used_zones,
        &mut placement,
        limit,
        &mut count,
    );
    count
}

fn generate_puzzle_spec(size: usize, difficulty: u8) -> AnimalPuzzleSpec {
    let mut rng = Rng::new();
    let (_, _, base_attempts) = difficulty_profile(difficulty);
    let attempts = base_attempts.saturating_add(size.saturating_mul(size));

    for _ in 0..attempts {
        let Some(solution_cols) = generate_solution_cols(size, &mut rng) else {
            continue;
        };
        let zones = generate_zones(size, &solution_cols, difficulty, &mut rng);
        if count_solutions(&zones, size, 2) == 1 {
            return AnimalPuzzleSpec {
                size,
                difficulty,
                animal_kind: rng.gen_range(ANIMAL_KIND_COUNT as usize) as u8,
                zones,
                solution: solution_mask_from_cols(&solution_cols, size),
            };
        }
    }

    // Fallback: return the last generated shape even if the uniqueness check
    // could not find a perfect board in time.
    let solution_cols = generate_solution_cols(size, &mut rng).unwrap_or_else(|| {
        (0..size).map(|row| row % size).collect()
    });
    let zones = generate_zones(size, &solution_cols, difficulty, &mut rng);
    AnimalPuzzleSpec {
        size,
        difficulty,
        animal_kind: rng.gen_range(ANIMAL_KIND_COUNT as usize) as u8,
        zones,
        solution: solution_mask_from_cols(&solution_cols, size),
    }
}

#[wasm_bindgen]
pub fn animal_puzzle_generate(size: usize, difficulty: u8) -> String {
    if size == 0 {
        return "{}".to_string();
    }

    let spec = generate_puzzle_spec(size, difficulty.max(1));
    set_state_from_spec(&spec);
    serde_json::to_string(&spec).unwrap_or_else(|_| "{}".to_string())
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
        puzzle
            .idx(row, col)
            .map(|idx| puzzle.animals[idx] != 0)
            .unwrap_or(false)
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
