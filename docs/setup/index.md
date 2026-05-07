---
layout: default
title: セットアップガイド
---

# セットアップガイド

## 前提条件

- Rust 1.70+
- wasm-pack
- Git

## インストール

```bash
# Rust インストール (未インストールの場合)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# wasm-pack インストール
cargo install wasm-pack

# リポジトリをクローン
git clone https://github.com/oosawak/rustgames.git
cd rustgames
```

## WASM ビルド

```bash
# 通常ビルド
wasm-pack build wasm_app --target web --out-dir docs/play/maze3d/wasm
rm -f docs/play/maze3d/wasm/.gitignore

# フォント埋め込みビルド (日本語表示を完全自己完結で保証、+約6MB)
wasm-pack build wasm_app --target web --out-dir docs/play/maze3d/wasm -- --features embed-font
rm -f docs/play/maze3d/wasm/.gitignore
```

## ローカル確認

```bash
# Python
python3 -m http.server 8080 --directory docs

# Node.js
npx serve docs
```

ブラウザで http://localhost:8080/play/maze3d/ を開く。

## FFI Bridge ビルド (Unity 向け)

```bash
# Windows
cargo build --release -p ffi_bridge --target x86_64-pc-windows-msvc

# macOS
cargo build --release -p ffi_bridge --target x86_64-apple-darwin

# Linux
cargo build --release -p ffi_bridge
```
