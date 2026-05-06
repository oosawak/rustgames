---
layout: default
title: セットアップガイド
---

# 🛠️ セットアップガイド

## 前提条件

### システム要件
- **OS**: Windows 10+, macOS 10.13+, Ubuntu 18.04+
- **RAM**: 4GB 以上推奨
- **GPU**: WebGPU/Vulkan/Metal/DX12 対応

### 必須ツール
- **Rust 1.70+** ([rustup](https://rustup.rs/)でインストール)
- **Git**
- **Python 3.8+** (WASM ローカル実行時)

## インストール手順

### 1. リポジトリのクローン

```bash
git clone https://github.com/oosawak/rustgames.git
cd rustgames
```

### 2. Rust ツールチェーン確認

```bash
rustc --version
cargo --version
```

### 3. 依存関係のダウンロード

```bash
cargo fetch
```

## プラットフォーム別セットアップ

### 🖥️ ネイティブアプリ（Windows/Mac/Linux）

#### Windows
```bash
# Visual Studio C++ ビルドツールが必要
cargo build --release -p native_app
./target/release/native_app.exe
```

#### macOS
```bash
cargo build --release -p native_app
./target/release/native_app
```

#### Linux (Ubuntu/Debian)
```bash
# 必須パッケージ
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

cargo build --release -p native_app
./target/release/native_app
```

### 🌐 WASM 版（ブラウザ）

```bash
# wasm-pack インストール
cargo install wasm-pack

# ビルド
wasm-pack build wasm_app --target web --release

# ローカルサーバー起動
cd wasm_app/pkg
python -m http.server 8000

# http://localhost:8000 でアクセス
```

### 🎮 Unity プラグイン

#### 1. FFI Bridge ビルド

**Windows:**
```bash
cargo build --release -p ffi_bridge --target x86_64-pc-windows-msvc
```

**macOS:**
```bash
cargo build --release -p ffi_bridge --target x86_64-apple-darwin
```

**Linux:**
```bash
cargo build --release -p ffi_bridge
```

#### 2. Unity プロジェクト設定

```
Unity Project/Assets/Plugins/
├── ffi_bridge.dll        (Windows)
├── libffi_bridge.dylib   (macOS)
└── libffi_bridge.so      (Linux)
```

## トラブルシューティング

### ビルドエラー

**"Failed to find GPU adapter"**
- GPU ドライバを最新版に更新
- wgpu がサポートする GPU か確認

**"Cargo エラー: 依存関係の競合"**
```bash
cargo clean
cargo build --release
```

### ネイティブアプリが起動しない

**Linux での GLIBC エラー:**
```bash
ldd ./target/release/native_app
# 不足しているライブラリを apt-get install
```

### WASM ビルド失敗

**"wasm-pack not found"**
```bash
cargo install wasm-pack --locked
```

**WASM ファイルが大きい**
```bash
# リリースビルド時に最適化
wasm-pack build wasm_app --target web --release
```

## 開発環境構築

### エディタ設定

#### VS Code
```json
// .vscode/settings.json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.inlayHints.lifetimeElisionHints.enable": "all"
}
```

#### IntelliJ IDEA
- Rust plugin インストール
- Settings → Languages & Frameworks → Rust

### デバッグビルド

```bash
# デバッグシンボル付きビルド
cargo build -p native_app

# デバッガで実行（例：gdb）
gdb ./target/debug/native_app
```

### ログ出力

```bash
# ログレベル指定
RUST_LOG=debug cargo run -p native_app
RUST_LOG=engine=trace,game_logic=debug cargo run -p native_app
```

## 認証とセキュリティ

### Git 設定
```bash
git config --global user.name "Your Name"
git config --global user.email "your@email.com"

# SSH キーの設定推奨
ssh-keygen -t ed25519 -C "your@email.com"
```

## 次のステップ

- ✅ [実行例](../examples/) を参照
- ✅ [API リファレンス](../api/) で API を学習
- ✅ [統合ガイド](../guides/) で各プラットフォーム統合方法を確認

---

**最終更新**: 2026年5月6日
