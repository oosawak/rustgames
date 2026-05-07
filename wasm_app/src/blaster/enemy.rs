#[derive(Clone, Copy, PartialEq)]
pub enum EnemyKind { Basic, Shooter }

#[derive(Clone)]
pub struct BlasterEnemy {
    pub active: bool,
    pub kind: EnemyKind,
    pub x: f32, pub y: f32, pub z: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub shoot_timer: f32,
    pub angle: f32,
    pub vx: f32, pub vz: f32,
}

impl BlasterEnemy {
    pub fn new_basic(x: f32, z: f32) -> Self {
        Self { active: true, kind: EnemyKind::Basic, x, y: 0.5, z,
               hp: 2, max_hp: 2, shoot_timer: 0.0, angle: 0.0, vx: 0.0, vz: 0.0 }
    }
    pub fn new_shooter(x: f32, z: f32) -> Self {
        Self { active: true, kind: EnemyKind::Shooter, x, y: 0.5, z,
               hp: 4, max_hp: 4, shoot_timer: 1.5, angle: 0.0, vx: 0.0, vz: 0.0 }
    }
}
