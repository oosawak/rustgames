// 定数モジュール: 迷路サイズ、壁の高さ、方向フラグ、バッファ上限などを定義する

pub const MAZE_W: usize = 9;
pub const MAZE_H: usize = 9;
pub const WALL_H: f32 = 1.5;   // 壁の高さ（高いほど開放感が増す）
pub const EYE_H:  f32 = 0.45;  // 視点の高さ（低いほど見上げる感じになる）
pub const MAX_VERTS: usize = 12288;
pub const MAX_IDX:   usize = 20480;

pub const N:     u8 = 1;
pub const E:     u8 = 2;
pub const S:     u8 = 4;
pub const W_DIR: u8 = 8;
