// camera.rs — カメラの位置・向き・射影行列の計算
//
// 【初心者向け説明】
// 3Dゲームの「カメラ」は3つの要素で決まります：
//
//   eye  = カメラの位置（どこに立っているか）
//   ctr  = 注視点（どこを見ているか）
//   fov  = 画角（視野の広さ。90°=広角、60°=標準）
//
// 現在の設定：プレイヤーの0.45セル後ろ・少し上から前方を見下ろす
// 「オーバーショルダー（肩越し）視点」です。
//
// 将来の拡張例：
//   - スムーズ追従（lerp でカメラをゆっくり追いかけさせる）
//   - 壁めり込み防止（eye が壁内に入ったら引き戻す）
//   - カメラシェイク（爆発・衝突時のぶれ）

use crate::math::{M4, look_at, perspective};
use crate::constants::{N, S, E, EYE_H};

/// カメラの設定パラメータ
pub struct CameraConfig {
    /// プレイヤー後方へのオフセット（セル単位）
    pub back_offset: f32,
    /// プレイヤー目線より上へのオフセット
    pub up_offset:   f32,
    /// 注視点の前方オフセット（プレイヤー前方何セル先を見るか）
    pub look_ahead:  f32,
    /// 縦方向の注視点調整（マイナスで少し下を見る）
    pub look_down:   f32,
    /// 画角（ラジアン）
    pub fov_rad:     f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            back_offset: 0.45,
            up_offset:   0.18,
            look_ahead:  0.6,
            look_down:  -0.04,
            fov_rad:     90.0f32.to_radians(),
        }
    }
}

/// カメラ行列（ビュー × 射影）を計算する
///
/// # 引数
/// - `px`, `pz`   : プレイヤーのセル座標
/// - `facing`     : 向き（N/E/S/W_DIR）
/// - `aspect`     : 画面のアスペクト比（幅 / 高さ）
/// - `cfg`        : カメラ設定
///
/// # 戻り値
/// `(view行列, proj行列)`  → 外側で mat_mul(proj, view) して使う
pub fn calc_camera(
    px: usize, pz: usize,
    facing: u8,
    aspect: f32,
    cfg: &CameraConfig,
) -> (M4, M4) {
    // プレイヤーのワールド座標（セルの中心）
    let center = [px as f32 + 0.5, EYE_H, pz as f32 + 0.5];

    // 向いている方向のベクトル
    let fwd: [f32; 3] = match facing {
        d if d == N => [ 0.0, 0.0, -1.0],
        d if d == S => [ 0.0, 0.0,  1.0],
        d if d == E => [ 1.0, 0.0,  0.0],
        _           => [-1.0, 0.0,  0.0],
    };

    // カメラ位置 = プレイヤー後方 + 少し上
    let eye = [
        center[0] - fwd[0] * cfg.back_offset,
        center[1] + cfg.up_offset,
        center[2] - fwd[2] * cfg.back_offset,
    ];

    // 注視点 = プレイヤー前方
    let ctr = [
        center[0] + fwd[0] * cfg.look_ahead,
        center[1] + cfg.look_down,
        center[2] + fwd[2] * cfg.look_ahead,
    ];

    let view = look_at(eye, ctr, [0.0, 1.0, 0.0]);
    let proj = perspective(cfg.fov_rad, aspect, 0.04, 50.0);

    (view, proj)
}
