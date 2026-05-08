# RustGames — ドキュメント

このディレクトリは GitHub Pages として配信されます。

🎮 **プレイ**: https://oosawak.github.io/rustgames/play/

---

## 収録ゲーム

### 🌐 Neon Maze 3D
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

### ⚡ Neon Blast 3D
**URL**: https://oosawak.github.io/rustgames/play/blaster3d/

wgpu WebGL で 256 発同時描画する 3D 弾幕シューター。戦車形状のキャラクターで戦う。

| 機能 | 内容 |
|------|------|
| レンダリング | wgpu / WebGL、3D 戦車モデル（5 パーツ）、パーティクル |
| 操作 | WASD/矢印=絶対方向移動、Q/E=砲塔回転、F=自動射撃切替 |
| 砲塔 | 車体と砲塔が独立回転。自動エイム or 手動操作 |
| 弾 | ホーミング弾（最近の敵を追尾）、256 発同時 |
| 敵 | Basic / Shooter の 2 種、5 ウェーブ + 3 フェーズボス |
| カメラ | TPS（後方視点）/ TOP（俯瞰）/ FPS の 3 モード（Tab 切替）|
| 衝突 | 敵同士・敵とプレイヤーの押し出し処理 |

---

### 🌍 Earth Defense（開発中）
**URL**: https://oosawak.github.io/rustgames/play/earthdef/

地球（3D ボックス）を無数の敵キューブから守る宇宙シューター。

| 機能 | 内容 |
|------|------|
| カメラ | 左ジョイスティックで地球を周回する軌道カメラ |
| レーザー | 右ジョイスティックで照準、離して発射 |
| レーザー種類 | BEAM（直線）/ SPREAD（3 方向）/ REFLECT（反射）|
| フラッシュ爆弾 | 全敵一掃、3 チャージ制（15 秒で 1 回復）|
| 敵種類 | Basic / Speed / Armored / Splitter（4 種）|
| タッチ操作 | デュアルバーチャルジョイスティック対応 |

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
│   ├── blaster3d/       - Neon Blast 3D
│   │   ├── index.html
│   │   ├── style.css
│   │   └── wasm/        - wasm-pack ビルド出力
│   ├── earthdef/        - Earth Defense（開発中）
│   │   ├── index.html
│   │   ├── style.css
│   │   └── wasm/        - wasm-pack ビルド出力
│   └── maze.html        - Neon Maze 2D（Canvas 2D）
├── index.md             - トップページ
├── setup/               - セットアップガイド
├── api/                 - API リファレンス
├── guides/              - Unity FFI 統合ガイド
└── examples/            - ビルド・デプロイ手順
```
