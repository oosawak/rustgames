// audio.rs — オーディオシステム（音声イベント管理）
//
// 【初心者向け説明】
// ブラウザの Web Audio API を使って効果音を生成します。
// RustのWASM側は「何の音を鳴らすか」をフラグで通知するだけで、
// 実際の音の生成はJavaScript側が担当します。
//
// 音の種類：
//   足音     : セル移動のたびに短いノイズバースト
//   壁衝突音 : 壁にぶつかった時の低いドン音
//   レベルクリア: 上昇アルペジオ（ドミソド）
//   アンビエント: ダンジョンの低いドローン音（ループ）

/// オーディオイベントの種類
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum AudioEvent {
    None       = 0,
    Step       = 1, // 足音
    WallHit    = 2, // 壁衝突
    LevelClear = 3, // レベルクリア
    GoalNear   = 4, // ゴール付近（1セル以内）
    EnemyNear  = 5, // 敵が1マス以内
    GameOver   = 6, // 捕まった
}

/// ゲームのオーディオ状態（毎フレームリセット）
pub struct AudioState {
    pub event:       AudioEvent,
    pub step_parity: bool,  // 足音の左右交互制御（true=左足, false=右足）
}

impl AudioState {
    pub fn new() -> Self {
        Self { event: AudioEvent::None, step_parity: false }
    }

    pub fn trigger(&mut self, ev: AudioEvent) {
        self.event = ev;
        if ev == AudioEvent::Step {
            self.step_parity = !self.step_parity;
        }
    }

    pub fn consume(&mut self) -> AudioEvent {
        let ev = self.event;
        self.event = AudioEvent::None;
        ev
    }
}
