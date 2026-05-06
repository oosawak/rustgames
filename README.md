# Rust Wgpu Game Engine + Unity Integration

Rustで構築した**wgpu ベースのゲームエンジン**と、**ネイティブプラグイン（FFI）経由での Unity 統合**の完全実装。

## 📋 プロジェクト構成

```
/rustgames
├── engine/              ⚙️  コアゲームエンジン
│   ├── renderer.rs      - wgpu レンダリングパイプライン
│   ├── scene.rs         - シーン・オブジェクト管理
│   ├── input.rs         - 入力処理
│   ├── math.rs          - 数学ユーティリティ
│   └── graphics/        - 3D メッシュ、シェーダー
│
├── game_logic/          🎮 ゲームロジック
│   ├── puzzle.rs        - パズルルール（立方体配置）
│   ├── particle.rs      - パーティクルシステム（光る線）
│   └── physics.rs       - 簡易物理演算
│
├── native_app/          🖥️  ネイティブアプリ
│   └── main.rs          - winit + wgpu スタンドアロン版
│
├── wasm_app/            🌐 WASM版
│   └── lib.rs           - wasm-bindgen + HTML Canvas
│
├── ffi_bridge/          🔗 Unity FFI Bridge ⭐ NEW
│   └── lib.rs           - C互換ライブラリ (DLL/so/dylib)
│
└── Cargo.toml           📦 ワークスペース設定
```

## 🎯 主な機能

### Engine API
- **Renderer**: wgpu ベースの 3D レンダリング
- **Scene Manager**: GameObject 階層管理
- **Input System**: キーボード・マウス入力
- **Math Library**: glam ベースの線形代数

### Game Features
- **3D パズル**: 立方体の配置・移動ロジック
- **Particle System**: 光る線エフェクト（色付きパーティクル）
- **Physics**: 簡易重力・衝突判定
- **UI**: スコア・移動数・ゲーム状態表示

## 🛠️ ビルド方法

### Rust ネイティブ版（スタンドアロン）
```bash
cd rustgames

# ビルド
cargo build --release -p native_app

# 実行
./target/release/native_app
```

### WASM 版（ブラウザ）
```bash
# wasm-pack インストール
cargo install wasm-pack

# ビルド
wasm-pack build wasm_app --target web --release

# ローカルサーバーで実行
python -m http.server 8000
```

### FFI Bridge（Unity 統合用）
```bash
# Windows DLL
cargo build --release -p ffi_bridge --target x86_64-pc-windows-msvc
# → target/release/ffi_bridge.dll

# macOS dylib
cargo build --release -p ffi_bridge --target x86_64-apple-darwin
# → target/release/libffi_bridge.dylib

# Linux so
cargo build --release -p ffi_bridge
# → target/release/libffi_bridge.so
```

## 🎮 Unity 統合ガイド

### 1. FFI Bridge ライブラリをコピー
```
Unity Project/Assets/Plugins/
├── ffi_bridge.dll        (Windows)
├── libffi_bridge.dylib   (macOS)
└── libffi_bridge.so      (Linux)
```

### 2. C# ラッパー をコピー
```
Unity Project/Assets/Scripts/
└── RustGameBridge.cs     (FFI インターフェース)
```

### 3. GameManager を追加
```
Unity Project/Assets/Scripts/
└── GameManager.cs        (Rust エンジン統合)
```

### 4. シーンセットアップ
```
Canvas
├── ScoreText
├── MovesText
├── TimeText
├── StatusText
└── ResetButton
```

### 5. コード例
```csharp
// ゲーム初期化
RustGameBridge.game_initialize(1280, 720);

// 毎フレーム更新
RustGameBridge.game_update(Time.deltaTime);

// 状態取得
var state = RustGameBridge.game_get_state();
Debug.Log($"Score: {state.score}, Moves: {state.moves}");

// 立方体操作
RustGameBridge.game_move_cube(cubeId, 0, 0, 1);

// クリーンアップ
RustGameBridge.game_cleanup();
```

## 📊 アーキテクチャ

### FFI ブリッジ設計
```
┌────────────────────────────┐
│    Unity (C#)              │
│  P/Invoke DLL Import       │
└────────────┬───────────────┘
             │
        ┌────▼────┐
        │  FFI    │
        │ Bridge  │
        └────┬────┘
             │
┌────────────▼───────────────┐
│  Rust Game Engine           │
│  • Engine                   │
│  • GameLogic                │
│  • ParticleSystem           │
│  • Physics                  │
└─────────────────────────────┘
```

### C 互換インターフェース
```rust
#[repr(C)]
pub struct GameStateFFI {
    pub score: u32,
    pub moves: u32,
    pub time_elapsed: f32,
    pub is_won: bool,
    pub puzzle_move_count: u32,
}

#[no_mangle]
pub extern "C" fn game_initialize(width: u32, height: u32) -> i32;
pub extern "C" fn game_update(delta_time: f32) -> i32;
pub extern "C" fn game_get_state() -> GameStateFFI;
pub extern "C" fn game_move_cube(cube_id: u32, x: i32, y: i32, z: i32) -> i32;
```

## 🚀 動作確認

### Rust テスト
```bash
cargo test --all
```

### Unity 統合テスト
```csharp
// GameManager でログを確認
✓ Game initialized successfully
✓ Update: Score=100, Moves=5
✓ Game cleaned up
```

## 📝 ライセンス

MIT License - 自由に使用、修正、配布可能

## 🤝 貢献

プルリクエスト、Issue 報告を歓迎します！

---

**作成日**: 2026年5月6日  
**バージョン**: 0.1.0  
**開発環境**: Rust 1.70+, Unity 2021+
