// パーティクルモジュール: 壁衝突やゴール到達時に出るパーティクルの構造体を定義する

pub struct Particle {
    pub pos:  [f32; 3],
    pub vel:  [f32; 3],
    pub life: f32,   // 1.0 = 生成直後, 0.0 = 消滅
}
