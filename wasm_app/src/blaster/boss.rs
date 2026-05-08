#[derive(Clone, Copy, PartialEq)]
pub enum BossPhase { Phase1, Phase2, Phase3, Dead }

pub struct Boss {
    pub active: bool,
    pub x: f32, pub y: f32, pub z: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub phase: BossPhase,
    pub body_angle:   f32,   // 車体（円運動の接線方向）
    pub turret_angle: f32,   // 砲塔（プレイヤー方向・独立回転）
    pub shoot_timer: f32,
    pub pattern_angle: f32,
    pub move_angle: f32,
}

impl Boss {
    pub fn new() -> Self {
        Self {
            active: false, x: 0.0, y: 0.5, z: -8.0,
            hp: 60, max_hp: 60, phase: BossPhase::Phase1,
            body_angle: 0.0, turret_angle: 0.0,
            shoot_timer: 0.0, pattern_angle: 0.0, move_angle: 0.0,
        }
    }

    pub fn spawn(&mut self) {
        *self = Boss::new();
        self.active = true;
    }

    pub fn update_phase(&mut self) {
        self.phase = if self.hp > 40      { BossPhase::Phase1 }
                     else if self.hp > 20 { BossPhase::Phase2 }
                     else if self.hp > 0  { BossPhase::Phase3 }
                     else                 { BossPhase::Dead };
    }
}
