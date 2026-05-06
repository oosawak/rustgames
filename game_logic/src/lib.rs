pub mod puzzle;
pub mod particle;
pub mod physics;

pub use puzzle::{PuzzleState, PuzzleLogic};
pub use particle::{ParticleSystem, Particle};
pub use physics::PhysicsWorld;

use cgmath::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

pub struct GameState {
    pub score: u32,
    pub moves: u32,
    pub time_elapsed: f32,
    pub puzzle: PuzzleLogic,
    pub particles: ParticleSystem,
    pub physics: PhysicsWorld,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            score: 0,
            moves: 0,
            time_elapsed: 0.0,
            puzzle: PuzzleLogic::new(),
            particles: ParticleSystem::new(),
            physics: PhysicsWorld::new(),
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        self.time_elapsed += delta_time;
        self.puzzle.update(delta_time);
        self.particles.update(delta_time);
        self.physics.update(delta_time);
    }
    
    pub fn move_cube(&mut self, cube_id: u32, delta_position: (i32, i32, i32)) -> bool {
        if self.puzzle.move_cube(cube_id, delta_position) {
            self.moves = self.puzzle.move_count;
            return true;
        }
        false
    }
    
    pub fn reset(&mut self) {
        self.score = 0;
        self.moves = 0;
        self.time_elapsed = 0.0;
        self.puzzle.reset();
        self.particles.clear();
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
