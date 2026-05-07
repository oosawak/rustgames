#[derive(Clone)]
pub struct Bullet {
    pub active: bool,
    pub x: f32, pub y: f32, pub z: f32,
    pub vx: f32, pub vy: f32, pub vz: f32,
    pub life: f32,
    pub is_player: bool,
    pub col: [f32; 3],
}

pub const MAX_BULLETS: usize = 256;

pub struct BulletPool {
    pub pool: Vec<Bullet>,
}

impl BulletPool {
    pub fn new() -> Self {
        Self {
            pool: (0..MAX_BULLETS).map(|_| Bullet {
                active: false, x: 0.0, y: 0.5, z: 0.0,
                vx: 0.0, vy: 0.0, vz: 0.0, life: 0.0,
                is_player: false, col: [1.0, 1.0, 1.0],
            }).collect(),
        }
    }

    pub fn spawn(&mut self, x: f32, y: f32, z: f32,
                 vx: f32, vy: f32, vz: f32,
                 life: f32, is_player: bool, col: [f32; 3]) -> bool {
        for b in &mut self.pool {
            if !b.active {
                *b = Bullet { active: true, x, y, z, vx, vy, vz, life, is_player, col };
                return true;
            }
        }
        false
    }

    pub fn tick(&mut self, dt: f32) {
        for b in &mut self.pool {
            if !b.active { continue; }
            b.x += b.vx * dt;
            b.y += b.vy * dt;
            b.z += b.vz * dt;
            b.life -= dt;
            if b.life <= 0.0 || b.x.abs() > 12.0 || b.z.abs() > 12.0 {
                b.active = false;
            }
        }
    }

    pub fn active_count(&self) -> usize {
        self.pool.iter().filter(|b| b.active).count()
    }
}
