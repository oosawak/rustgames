---
layout: default
title: RustGames
description: Rust + wgpu WASM ゲームエンジン
---

# 🎮 RustGames

Rust / wgpu / WebGL をベースとした WASM ゲームエンジンと、JS ゲームエンジンライブラリ (`engine.js`) の実装。

## 🕹️ ゲームをプレイ

| ゲーム | URL | 説明 |
|--------|-----|------|
| 🌐 Neon Maze 3D | [プレイする](https://oosawak.github.io/rustgames/play/maze3d/) | ネオン風 3D 一人称迷路 |
| 🗺️ Neon Maze 2D | [プレイする](https://oosawak.github.io/rustgames/play/maze.html) | Canvas 2D ネオン迷路 |
| ⚡ Neon Blast 3D | [プレイする](https://oosawak.github.io/rustgames/play/blaster3d/) | 3D 弾幕シューター・戦車操作 |
| 🌍 Earth Defense | [プレイする](https://oosawak.github.io/rustgames/play/earthdef/) | 3D スペースシューター（開発中）|
| ゲーム一覧 | [一覧を見る](https://oosawak.github.io/rustgames/play/) | |

## ✨ 主な特徴

- 🦀 **Rust + wgpu** — WGSL シェーダー、点光源、指数フォグ
- 🌐 **WASM 対応** — ブラウザで直接実行（wasm-pack）
- 🔤 **Gen Interface JP フォント** — エンジン標準フォント、WASM 埋め込みも可能
- 🎵 **SoundBuilder** — Rust で音を定義、JSON で Web Audio API に渡す
- 🎮 **engine.js** — フォント・音声・入力・Canvas を統合した JS ライブラリ
- 🔗 **Unity FFI** — C 互換 ABI で Unity から呼び出し可能

## 📚 ドキュメント

- [セットアップ](setup/) — 開発環境構築
- [API リファレンス](api/) — engine.js / WASM エクスポート API
- [統合ガイド](guides/) — Unity FFI Bridge
- [実行例](examples/) — ビルド・デプロイ手順

## 🛠️ 技術スタック

| カテゴリ | 技術 |
|----------|------|
| グラフィックス | **wgpu** (WebGL バックエンド) |
| 言語 | **Rust 1.70+** |
| WASM バインディング | **wasm-bindgen** |
| シェーダー | **WGSL** |
| JS エンジン | **engine.js** (ES Modules) |
| フォント | **Gen Interface JP** (OFL) |
| Unity 統合 | **FFI** (C 互換 ABI) |
