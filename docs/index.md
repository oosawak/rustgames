---
layout: default
title: Rust wgpu Game Engine
description: Cross-platform 3D puzzle game with Unity integration
---

# 🎮 Rust wgpu Game Engine

高性能な **Rust + wgpu** ベースのゲームエンジンです。3D パズルゲームをネイティブアプリ、WASM、Unity プラグインで実行できます。

## ✨ 主な特徴

- 🚀 **ネイティブパフォーマンス** - Rust + wgpu による最適化
- 🖥️ **クロスプラットフォーム** - Windows, macOS, Linux 対応
- 🌐 **WASM 対応** - ブラウザで直接実行
- 🎮 **Unity 統合** - FFI ブリッジで完全な相互運用性
- 🔒 **メモリ安全** - Rust の型システムが自動保証
- 💡 **API ベース** - 拡張可能なエンジン設計

## 📦 プロジェクト構成

```
rustgames/
├── engine/          ⚙️ コアゲームエンジン
├── game_logic/      🎮 ゲームロジック
├── native_app/      🖥️ ネイティブアプリ
├── wasm_app/        🌐 WASM版
├── ffi_bridge/      🔗 Unity プラグイン
└── docs/            📚 ドキュメント
```

## 🚀 クイックスタート

### ネイティブ版を実行
```bash
cargo run --release -p native_app
```

### WASM 版をビルド
```bash
wasm-pack build wasm_app --target web --release
```

### Unity プラグインをビルド
```bash
cargo build --release -p ffi_bridge
```

## 📚 ドキュメント

- **[セットアップガイド](setup/)** - 開発環境構築
- **[実行例](examples/)** - プラットフォーム別実行方法
- **[API リファレンス](api/)** - エンジン API 詳細
- **[統合ガイド](guides/)** - 各プラットフォーム統合手順

## 🎯 ゲーム概要

**3D パズル：立方体配置ゲーム**
- 4 個の色付き立方体を 3D 空間に配置
- 初期位置から目標位置へ移動
- パーティクルエフェクト搭載
- スコア・移動数カウント

## 🛠️ 技術スタック

| 層 | 技術 |
|---|---|
| グラフィックス | wgpu, WGSL |
| エンジン | Rust, async/await |
| プラットフォーム | winit (ネイティブ), Canvas (WASM) |
| 統合 | FFI, P/Invoke (Unity C#) |

## 📝 ライセンス

MIT License

## 🤝 サポート

- 📖 [GitHub Wiki](https://github.com/oosawak/rustgames/wiki)
- 💬 [Discussions](https://github.com/oosawak/rustgames/discussions)
- 🐛 [Issues](https://github.com/oosawak/rustgames/issues)

---

**最新バージョン**: 0.1.0  
**最終更新**: 2026年5月6日
