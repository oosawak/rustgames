use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CubeColor {
    Red,
    Green,
    Blue,
    Yellow,
    Purple,
}

#[derive(Clone, Copy, Debug)]
pub struct Cube {
    pub position: (i32, i32, i32),
    pub color: CubeColor,
    pub id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PuzzleState {
    Playing,
    Won,
    Lost,
    Paused,
}

pub struct PuzzleLogic {
    pub state: PuzzleState,
    pub cubes: HashMap<u32, Cube>,
    pub goal_positions: HashMap<u32, (i32, i32, i32)>,
    pub next_id: u32,
    pub move_count: u32,
}

impl PuzzleLogic {
    pub fn new() -> Self {
        let mut puzzle = PuzzleLogic {
            state: PuzzleState::Playing,
            cubes: HashMap::new(),
            goal_positions: HashMap::new(),
            next_id: 1,
            move_count: 0,
        };
        
        puzzle.initialize_level();
        puzzle
    }
    
    fn initialize_level(&mut self) {
        // Add 4 cubes in a 2x2 grid
        let colors = vec![CubeColor::Red, CubeColor::Green, CubeColor::Blue, CubeColor::Yellow];
        let positions = vec![
            (0, 0, 0),
            (1, 0, 0),
            (0, 1, 0),
            (1, 1, 0),
        ];
        let goal_positions = vec![
            (0, 0, 2),
            (1, 0, 2),
            (0, 1, 2),
            (1, 1, 2),
        ];
        
        for (color, (pos, goal)) in colors.iter().zip(positions.iter().zip(goal_positions.iter())) {
            let cube = Cube {
                position: *pos,
                color: *color,
                id: self.next_id,
            };
            self.cubes.insert(self.next_id, cube);
            self.goal_positions.insert(self.next_id, *goal);
            self.next_id += 1;
        }
    }
    
    pub fn move_cube(&mut self, cube_id: u32, delta_position: (i32, i32, i32)) -> bool {
        if let Some(cube) = self.cubes.get(&cube_id) {
            // Calculate new position by adding delta
            let new_x = cube.position.0 + delta_position.0;
            let new_y = cube.position.1 + delta_position.1;
            let new_z = cube.position.2 + delta_position.2;
            let new_pos = (new_x, new_y, new_z);
            
            // Check if position is valid (within grid bounds)
            if new_x >= -5 && new_x <= 5 && new_y >= -5 && new_y <= 5 && new_z >= 0 && new_z <= 5 {
                // Check for collision with other cubes
                let occupied = self.cubes.iter().any(|(other_id, other_cube)| {
                    *other_id != cube_id && other_cube.position == new_pos
                });
                
                if !occupied {
                    // Now we can safely do the mutable borrow
                    if let Some(cube_mut) = self.cubes.get_mut(&cube_id) {
                        cube_mut.position = new_pos;
                        self.move_count += 1;
                        self.check_win();
                        return true;
                    }
                }
            }
        }
        false
    }
    
    pub fn check_win(&mut self) {
        let all_correct = self.cubes.iter().all(|(id, cube)| {
            self.goal_positions.get(id).map_or(false, |goal| cube.position == *goal)
        });
        
        if all_correct {
            self.state = PuzzleState::Won;
        }
    }
    
    pub fn get_cube_at(&self, pos: (i32, i32, i32)) -> Option<u32> {
        self.cubes
            .iter()
            .find(|(_, cube)| cube.position == pos)
            .map(|(id, _)| *id)
    }
    
    pub fn is_won(&self) -> bool {
        self.state == PuzzleState::Won
    }
    
    pub fn reset(&mut self) {
        self.state = PuzzleState::Playing;
        self.cubes.clear();
        self.goal_positions.clear();
        self.next_id = 1;
        self.move_count = 0;
        self.initialize_level();
    }
    
    pub fn pause(&mut self) {
        if self.state == PuzzleState::Playing {
            self.state = PuzzleState::Paused;
        }
    }
    
    pub fn resume(&mut self) {
        if self.state == PuzzleState::Paused {
            self.state = PuzzleState::Playing;
        }
    }
    
    pub fn update(&mut self, _delta_time: f32) {
        // Puzzle update logic (animations, state transitions, etc.)
    }
}

impl Default for PuzzleLogic {
    fn default() -> Self {
        Self::new()
    }
}
