// engine.rs — RustGamesEngine: モード別初期化とゲームエンジンの入口
//
// 【初心者向け説明】
// このファイルはゲームエンジンの「受付」です。
// init() にモード文字列を渡すだけで、対応する初期化が行われます。
//
//   RustGamesEngine::init("3d", "canvas-id")  → wgpu 3D描画モード
//   RustGamesEngine::init("2d", "canvas-id")  → 2D描画モード（将来対応）
//   RustGamesEngine::init("console", "")      → コンソール出力モード（デバッグ用）
//
// 将来の拡張例：
//   - "vr"      : WebXR対応
//   - "headless": テスト・Unity FFI用（画面なし）

use wasm_bindgen::prelude::*;

/// ゲームエンジンの動作モード
#[derive(Debug, Clone, PartialEq)]
pub enum EngineMode {
    /// wgpu を使った3D描画モード
    Mode3D,
    /// 2D描画モード（将来実装）
    Mode2D,
    /// テキスト出力モード（デバッグ・FFI用）
    Console,
}

impl EngineMode {
    /// 文字列からモードに変換
    /// 不明な文字列は 3D にフォールバック
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "3d"      => Self::Mode3D,
            "2d"      => Self::Mode2D,
            "console" => Self::Console,
            other => {
                web_sys::console::warn_1(
                    &format!("RustGamesEngine: 不明なモード '{}' → 3D で起動", other).into()
                );
                Self::Mode3D
            }
        }
    }
}

/// ゲームエンジン本体
/// 初期化後の状態・設定を保持する
pub struct RustGamesEngine {
    pub mode:      EngineMode,
    pub canvas_id: String,
}

impl RustGamesEngine {
    /// エンジンを初期化する
    ///
    /// # 引数
    /// - `mode`      : "3d" / "2d" / "console"
    /// - `canvas_id` : HTML canvas 要素の id（3D/2D時に使用）
    ///
    /// # 使用例
    /// ```
    /// let engine = RustGamesEngine::init("3d", "maze3d-canvas");
    /// ```
    pub fn init(mode: &str, canvas_id: &str) -> Self {
        let engine_mode = EngineMode::from_str(mode);

        // モード別の起動ログ
        let msg = match &engine_mode {
            EngineMode::Mode3D  => format!("🎮 RustGames Engine: 3Dモード起動 (canvas={})", canvas_id),
            EngineMode::Mode2D  => format!("🎮 RustGames Engine: 2Dモード起動 (canvas={})", canvas_id),
            EngineMode::Console => "🎮 RustGames Engine: コンソールモード起動".to_string(),
        };
        web_sys::console::log_1(&msg.into());

        Self {
            mode: engine_mode,
            canvas_id: canvas_id.to_string(),
        }
    }

    /// 現在のモードが3Dかどうか
    pub fn is_3d(&self) -> bool { self.mode == EngineMode::Mode3D }

    /// 現在のモードが2Dかどうか
    pub fn is_2d(&self) -> bool { self.mode == EngineMode::Mode2D }
}
