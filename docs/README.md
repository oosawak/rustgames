# RustGames — ドキュメント

このディレクトリは GitHub Pages として配信されます。

🎮 **プレイ**: https://oosawak.github.io/rustgames/play/

---

## 収録ゲーム

### Neon Maze 3D
**URL**: https://oosawak.github.io/rustgames/play/maze3d/

wgpu / WebGL で描画するネオン風 3D 一人称迷路ゲーム。

| 機能 | 内容 |
|------|------|
| レンダリング | wgpu / WebGL、WGSL シェーダー、指数二乗フォグ、点光源 |
| 迷路生成 | 再帰バックトラッキング（9×9、レベルごとに再生成）|
| 敵 AI | BFS 経路探索、ゲームオーバー判定 |
| 壁テーマ | 6 種類（Neon Cyan / Void Purple / Ice Cave / Lava / Forest / Gold）|
| シーン遷移 | Title → Playing → LevelClear → GameOver |
| スコア保存 | localStorage（ベスト歩数・最高レベル・プレイ回数）|
| サウンド | Web Audio API、SoundDef JSON ドリブン（7 種 + アンビエント）|
| ミニマップ | Canvas 2D、視野コーン・方位表示 |
| 入力 | キーボード・D-pad・スワイプ対応 |

---

## engine.js — JS ゲームエンジンライブラリ

`docs/engine.js` は WASM ゲームに共通する機能をまとめた ES Modules ライブラリです。

```js
import { FontLoader, AudioEngine, InputManager, CanvasManager } from '../../engine.js';
```

| クラス | 機能 |
|--------|------|
| `FontLoader` | Gen Interface JP フォントの自動ロード（WASM 埋め込み / URL / 自動解決）|
| `AudioEngine` | Web Audio API ラッパー（SoundDef 再生 + アンビエント）|
| `InputManager` | キーボード / D-pad / スワイプ統一入力 |
| `CanvasManager` | Canvas レスポンシブリサイズ |

---

## フォント — Gen Interface JP

エンジン標準フォント。`docs/fonts/` に配置（OFL ライセンス）。

- `embed-font` feature でビルドすると WASM バイナリに同梱（日本語表示を完全自己完結で保証）
- 通常ビルドは `engine.js` が `docs/fonts/` から自動ロード

---

## ディレクトリ構成

```
docs/
├── engine.js            - JS ゲームエンジンライブラリ
├── engine.css           - エンジン共通スタイル
├── fonts/               - Gen Interface JP フォント (OFL)
├── play/
│   ├── index.html       - ゲーム一覧ページ
│   ├── maze3d/          - Neon Maze 3D
│   │   ├── index.html
│   │   ├── style.css
│   │   └── wasm/        - wasm-pack ビルド出力
│   └── game.html        - (旧 3D パズル、参考用)
├── index.md             - トップページ
├── setup/               - セットアップガイド
├── api/                 - API リファレンス
├── guides/              - 統合ガイド
└── examples/            - 実行例
```
