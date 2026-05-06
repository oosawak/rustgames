# Rust wgpu Game Engine

> 高性能な Rust + wgpu ベースのゲームエンジン。3D パズルゲームをネイティブアプリ、WASM、Unity プラグインで実行。

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.70+](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform: Windows | macOS | Linux | WASM](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20WASM-blue.svg)](#)

## 🚀 クイックスタート

### ネイティブ版
```bash
git clone https://github.com/oosawak/rustgames.git
cd rustgames
cargo run --release -p native_app
```

### WASM 版
```bash
wasm-pack build wasm_app --target web --release
python -m http.server 8000
# http://localhost:8000 で実行
```

### Unity プラグイン
```bash
cargo build --release -p ffi_bridge
# DLL/so/dylib を Unity/Assets/Plugins にコピー
```

## 📚 ドキュメント

- **[セットアップガイド](docs/setup/)** - 開発環境構築手順
- **[実行例](docs/examples/)** - プラットフォーム別実行方法
- **[API リファレンス](docs/api/)** - エンジン API 詳細
- **[統合ガイド](docs/guides/)** - 各プラットフォーム統合手順

## 🎮 プロジェクト構成

```
rustgames/
├── engine/          ⚙️ コアゲームエンジン
│   ├── renderer.rs  - wgpu レンダリング
│   ├── scene.rs     - シーン管理
│   ├── input.rs     - 入力処理
│   └── graphics/    - 3D メッシュ・シェーダー
│
├── game_logic/      🎮 ゲームロジック
│   ├── puzzle.rs    - パズルルール
│   ├── particle.rs  - パーティクル
│   └── physics.rs   - 物理演算
│
├── native_app/      🖥️ ネイティブアプリ
├── wasm_app/        🌐 WASM版
├── ffi_bridge/      🔗 Unity プラグイン
└── docs/            📚 ドキュメント
```

## ✨ 主な機能

- 🏗️ **API ベース設計** - 拡張可能なエンジン
- 🎯 **3D パズルゲーム** - 立方体配置・移動ロジック
- ✨ **パーティクルシステム** - 光る線エフェクト
- 🔗 **FFI 統合** - Unity ネイティブプラグイン対応
- 📱 **クロスプラットフォーム** - Win/Mac/Linux/Web
- 🔒 **メモリ安全** - Rust による自動安全性

## 🛠️ 技術スタック

| レイヤー | 技術 |
|---------|------|
| グラフィックス | **wgpu** (WebGPU) |
| 言語 | **Rust 1.70+** |
| 数学 | **glam**, cgmath |
| ネイティブ | **winit**, tokio |
| WASM | **wasm-bindgen** |
| 統合 | **FFI** (C 互換) |

## 📊 パフォーマンス

| プラットフォーム | FPS | メモリ |
|---|---|---|
| ネイティブ | 60+ fps | ~50MB |
| WASM | 45-50 fps | ~80MB |
| Unity Plugin | 60+ fps | ~60MB |

## 📖 例：Unity 統合

```csharp
// Assets/Scripts/GameManager.cs
using UnityEngine;

public class GameManager : MonoBehaviour {
    void Start() {
        RustGameBridge.game_initialize(1280, 720);
    }
    
    void Update() {
        RustGameBridge.game_update(Time.deltaTime);
        var state = RustGameBridge.game_get_state();
        Debug.Log($"Score: {state.score}");
    }
    
    void OnDestroy() {
        RustGameBridge.game_cleanup();
    }
}
```

## 🔧 ビルド

### 前提条件
- Rust 1.70+
- Python 3.8+ (WASM)
- Git

### インストール
```bash
git clone https://github.com/oosawak/rustgames.git
cd rustgames
cargo fetch
```

### ビルドコマンド

```bash
# ネイティブ
cargo build --release -p native_app

# WASM
wasm-pack build wasm_app --target web --release

# FFI Bridge (Windows)
cargo build --release -p ffi_bridge --target x86_64-pc-windows-msvc

# FFI Bridge (macOS)
cargo build --release -p ffi_bridge --target x86_64-apple-darwin

# FFI Bridge (Linux)
cargo build --release -p ffi_bridge
```

## 🧪 テスト

```bash
# 全テスト実行
cargo test --all

# 特定のクレートをテスト
cargo test -p engine
cargo test -p game_logic
cargo test -p ffi_bridge
```

## 📝 ドキュメント生成

```bash
# API ドキュメント生成
cargo doc --no-deps --open
```

## 🤝 貢献

プルリクエストを歓迎します！

1. Fork する
2. 機能ブランチを作成 (`git checkout -b feature/amazing-feature`)
3. コミット (`git commit -m 'Add amazing feature'`)
4. ブランチをプッシュ (`git push origin feature/amazing-feature`)
5. Pull Request を開く

## 📄 ライセンス

MIT License - 自由に使用、修正、配布が可能です。詳細は [LICENSE](LICENSE) を参照。

## 🎯 ロードマップ

- [x] コアエンジン実装
- [x] 3D グラフィックス機能
- [x] パズルゲーム実装
- [x] ネイティブアプリ対応
- [x] WASM ブラウザ対応
- [x] Unity FFI 統合
- [ ] マルチレベルシステム
- [ ] サウンドエフェクト
- [ ] テクスチャマッピング
- [ ] iOS/Android 対応

## 📞 サポート

- 📖 [GitHub Wiki](https://github.com/oosawak/rustgames/wiki)
- 💬 [Discussions](https://github.com/oosawak/rustgames/discussions)
- 🐛 [Issues](https://github.com/oosawak/rustgames/issues)

## 👨‍💻 作者

**oosawak**

## 🙏 謝辞

- [wgpu](https://github.com/gfx-rs/wgpu) - WebGPU 実装
- [winit](https://github.com/rust-windowing/winit) - ウィンドウマネージャー
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) - WASM バインディング

---

**バージョン**: 0.1.0  
**最終更新**: 2026年5月6日

**[📚 ドキュメントへ](docs/)** | **[⭐ Star on GitHub](https://github.com/oosawak/rustgames)**
