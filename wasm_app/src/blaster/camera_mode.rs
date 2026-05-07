#[derive(Clone, Copy, PartialEq)]
pub enum CameraMode { Tps = 0, Top = 1, Fps = 2 }

impl CameraMode {
    pub fn next(self) -> Self {
        match self {
            Self::Tps => Self::Top,
            Self::Top => Self::Fps,
            Self::Fps => Self::Tps,
        }
    }
    pub fn as_u8(self) -> u8 { self as u8 }
    pub fn name(self) -> &'static str {
        match self { Self::Tps => "TPS", Self::Top => "TOP", Self::Fps => "FPS" }
    }
}

/// プレイヤー位置と向きからカメラの eye/target を計算する
pub fn camera_view(mode: CameraMode, pos: [f32; 3], angle: f32) -> [[f32; 3]; 2] {
    let (sin, cos) = (angle.sin(), angle.cos());
    let dir = [sin, 0.0, cos];
    match mode {
        CameraMode::Tps => {
            let eye = [pos[0] - dir[0] * 4.0, pos[1] + 2.5, pos[2] - dir[2] * 4.0];
            let ctr = [pos[0], pos[1] + 0.3, pos[2]];
            [eye, ctr]
        }
        CameraMode::Top => {
            let eye = [pos[0], pos[1] + 18.0, pos[2] + 0.01];
            let ctr = [pos[0], pos[1], pos[2]];
            [eye, ctr]
        }
        CameraMode::Fps => {
            let eye = [pos[0], pos[1] + 0.4, pos[2]];
            let ctr = [pos[0] + dir[0], pos[1] + 0.4, pos[2] + dir[2]];
            [eye, ctr]
        }
    }
}
