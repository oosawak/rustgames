// input.rs — 入力管理（キーボード・タッチ・Dパッド・ゲームパッド）
//
// 【初心者向け説明】
// ゲームへの「入力」をここで一元管理します。
// プラットフォームが変わっても（スマホ↔デスクトップ）、
// ゲームロジック側は同じ「アクション番号」を受け取るだけでOKです。
//
// アクション番号の定義：
//   0 = 前進（Forward）
//   1 = 左折（Turn Left）
//   2 = 右折（Turn Right）
//   3 = 後退（Backward）
//
// 対応入力デバイス：
//   ┌─────────────────────────────────────────────────────┐
//   │ キーボード : ↑↓←→ / WASD                          │
//   │ タッチ     : スワイプ（上下左右）                   │
//   │ Dパッド    : 画面上の仮想ボタン                     │
//   │ ゲームパッド: Gamepad API（将来対応）               │
//   └─────────────────────────────────────────────────────┘
//
// ※ 現在の実装はJavaScript側 (index.html) で行っています。
//    将来的にここでRust/WASM側から直接入力イベントを管理できます。

/// ゲームアクション（入力の種類）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Forward  = 0,
    TurnLeft = 1,
    TurnRight= 2,
    Backward = 3,
}

impl Action {
    pub fn from_i32(n: i32) -> Option<Self> {
        match n {
            0 => Some(Self::Forward),
            1 => Some(Self::TurnLeft),
            2 => Some(Self::TurnRight),
            3 => Some(Self::Backward),
            _ => None,
        }
    }
}

// TODO: 将来実装
// pub struct InputManager { ... }
// impl InputManager {
//     pub fn register_touch(canvas: &HtmlCanvasElement) { ... }
//     pub fn register_keyboard() { ... }
//     pub fn register_gamepad() { ... }
//     pub fn poll() -> Vec<Action> { ... }
// }
