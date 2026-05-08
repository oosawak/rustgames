# RustGames — Rust + wgpu WASM ゲームエンジン

Rust / wgpu / WebGL をベースとした **WASM ゲームエンジン**と、そのゲームを動かすための **JavaScript ゲームエンジンライブラリ (`engine.js`)** の実装。

🎮 **プレイ**: https://oosawak.github.io/rustgames/play/

---

## 📋 プロジェクト構成

```
/rustgames
├── wasm_app/                🦀 Rust WASM ゲームエンジン
│   ├── src/
│   │   ├── lib.rs           - wasm-bindgen エクスポート（全 WASM API）
│   │   ├── game.rs          - ゲームループ・状態管理
│   │   ├── gpu.rs           - wgpu 初期化・レンダリングパイプライン
│   │   ├── geometry.rs      - 頂点・Uniform 構造体（bytemuck）
│   │   ├── shader.rs        - WGSL シェーダー（フォグ・点光源）
│   │   ├── maze.rs          - 迷路生成（再帰バックトラッキング）
│   │   ├── enemy.rs         - 敵 AI（BFS 経路探索）
│   │   ├── theme.rs         - 壁テーマ（6 種類のカラーパレット）
│   │   ├── scene.rs         - シーン遷移（Title / Playing / LevelClear / GameOver）
│   │   ├── storage.rs       - localStorage 保存（スコア・音量・プレイ回数）
│   │   ├── audio_tool.rs    - SoundDef ビルダー（Web Audio API 用 JSON 生成）
│   │   ├── font.rs          - フォント埋め込み（embed-font feature）
│   │   ├── camera.rs        - FPS カメラ（スムーズ補間）
│   │   ├── particle.rs      - パーティクルシステム
│   │   ├── input.rs         - 入力処理
│   │   └── math.rs          - 数学ユーティリティ
│   └── Cargo.toml
│
├── docs/                    📦 GitHub Pages 配信ディレクトリ
│   ├── engine.js            - JS ゲームエンジンライブラリ ⭐
│   ├── engine.css           - エンジン共通スタイル
│   ├── fonts/               - Gen Interface JP フォント (OFL)
│   │   ├── GenInterfaceJP-Regular.ttf
│   │   ├── GenInterfaceJP-Light.ttf
│   │   └── GenInterfaceJP-Bold.ttf
│   └── play/
│       ├── maze3d/          - Neon Maze 3D ゲーム
│       │   ├── index.html
│       │   ├── style.css
│       │   └── wasm/        - wasm-pack ビルド出力
│       ├── blaster3d/       - Neon Blast 3D ゲーム
│       │   ├── index.html
│       │   ├── style.css
│       │   └── wasm/        - wasm-pack ビルド出力
│       ├── earthdef/        - Earth Defense（開発中）
│       │   ├── index.html
│       │   ├── style.css
│       │   └── wasm/        - wasm-pack ビルド出力
│       └── index.html       - ゲーム一覧
│
├── ffi_bridge/              🔗 Unity FFI Bridge
│   └── lib.rs               - C 互換ライブラリ (DLL/so/dylib)
│
└── Cargo.toml               📦 ワークスペース設定
```

---

## 🎮 収録ゲーム

| ゲーム | URL | 説明 |
|--------|-----|------|
| 🌐 Neon Maze 3D | [プレイ](https://oosawak.github.io/rustgames/play/maze3d/) | ネオン風 3D 一人称迷路 |
| 🗺️ Neon Maze 2D | [プレイ](https://oosawak.github.io/rustgames/play/maze.html) | Canvas 2D ネオン迷路 |
| ⚡ Neon Blast 3D | [プレイ](https://oosawak.github.io/rustgames/play/blaster3d/) | 3D 弾幕シューター・戦車操作 |
| 🌍 Earth Defense | [プレイ](https://oosawak.github.io/rustgames/play/earthdef/) | 3D スペースシューター（開発中）|

### Neon Maze 3D
wgpu WebGL で描画するネオン風 3D 一人称迷路ゲーム。

| 機能 | 内容 |
|------|------|
| レンダリング | wgpu / WebGL、WGSL シェーダー、指数二乗フォグ、点光源 |
| 迷路生成 | 再帰バックトラッキング（9×9、レベルごとに再生成）|
| 敵 AI | BFS 経路探索、ゲームオーバー判定 |
| 壁テーマ | 6 種類（Neon Cyan / Void Purple / Ice Cave / Lava / Forest / Gold）|
| シーン遷移 | Title → Playing → LevelClear → GameOver |
| スコア保存 | localStorage（ベスト歩数・最高レベル・プレイ回数）|
| サウンド | Web Audio API、SoundDef JSON ドリブン（7 種の効果音 + アンビエント）|
| ミニマップ | Canvas 2D、視野コーン・方位表示 |
| 入力 | キーボード・D-pad・スワイプ対応 |

### ⚡ Neon Blast 3D
wgpu WebGL で 256 発同時描画する 3D 弾幕シューター。戦車形状のキャラクターで戦う。

| 機能 | 内容 |
|------|------|
| 操作 | WASD/矢印=絶対方向移動、Q/E=砲塔回転、F=自動射撃切替、Tab=カメラ切替 |
| 砲塔 | 車体と砲塔が独立回転。自動エイム or 手動操作を切替可 |
| 弾 | ホーミング弾（最近の敵を追尾）、256 発同時 |
| 敵 | Basic / Shooter の 2 種、5 ウェーブ + 3 フェーズボス |
| カメラ | TPS / TOP / FPS の 3 モード（Tab 切替）|
| 衝突 | 敵同士・敵とプレイヤーの押し出し処理 |

### 🌍 Earth Defense（開発中）
地球（3D ボックス）を無数の敵キューブから守る宇宙シューター。

| 機能 | 内容 |
|------|------|
| カメラ | 左ジョイスティックで地球周回軌道カメラ |
| レーザー | BEAM / SPREAD / REFLECT の 3 種類 |
| フラッシュ爆弾 | 全敵一掃、3 チャージ制 |
| 敵種類 | Basic / Speed / Armored / Splitter（4 種）|

---

## ⚙️ engine.js — JS ゲームエンジンライブラリ

`docs/engine.js` は WASM ゲームに共通する機能をまとめたライブラリです。  
新しいゲームを作るときは `engine.js` をインポートするだけで使えます。

```js
import { FontLoader, AudioEngine, InputManager, CanvasManager } from '../../engine.js';
```

### FontLoader

GenInterfaceJP フォントを自動ロードして `document.fonts` に登録します。

```js
// wasm モジュールを渡すだけ — 取得元を自動判定
await FontLoader.load('GenInterfaceJP', wasm);
```

フォント取得の優先順位：

| 優先度 | 条件 | 取得元 |
|--------|------|--------|
| 1 | `embed-font` feature でビルド済み | WASM バイナリ内の埋め込みバイト列 → Blob URL |
| 2 | `fontUrl` オプション指定あり | 指定 URL |
| 3 | （デフォルト）| `engine.js` と同階層の `fonts/` ディレクトリ（`import.meta.url` で自動解決）|

```js
// カスタム URL を指定する場合（別ドメインへの移植時など）
await FontLoader.load('GenInterfaceJP', wasm, { fontUrl: 'https://cdn.example.com/font.ttf' });
```

### AudioEngine

Web Audio API のラッパー。Rust の `audio_tool.rs` が生成した SoundDef JSON を再生します。

```js
const audio = new AudioEngine();  // autoAmbient: true（デフォルト）

audio.ensure();                   // AudioContext を遅延初期化（ユーザー操作後に呼ぶ）
audio.play(soundDef, volume);     // SoundDef JSON を再生
audio.startAmbient();             // アンビエントドローンを開始
audio.setSeVolume(0.8);           // SE 音量を変更
audio.setAmbVolume(0.4);          // アンビエント音量を変更
```

### InputManager

キーボード・D-pad・タッチスワイプを統一管理します。

```js
const input = new InputManager({ swipeMin: 35 });

input.onAction(action => wasm.move_game(action)); // 0=前 1=左 2=右 3=後

input.bindKeyboard({ ArrowUp:0, KeyW:0, ArrowLeft:1, KeyA:1,
                     ArrowRight:2, KeyD:2, ArrowDown:3, KeyS:3 });

input.bindDpad([['db-up',0],['db-left',1],['db-right',2],['db-down',3]],
               () => audio.ensure());

input.bindSwipe(['#hud', '#dpad'], { onHint: flashHint, onStart: () => audio.ensure() });
```

### CanvasManager

Canvas をウィンドウサイズに合わせてリサイズします。

```js
const canvas = new CanvasManager('game-canvas', { hudHeight: 44, dpadHeight: 160 });
canvas.resize();       // 即時リサイズ
canvas.bindResize();   // window.resize に自動バインド
```

---

## 🔤 フォント — Gen Interface JP

エンジン標準フォントとして [Gen Interface JP](https://github.com/yamatoiizuka/gen-interface-jp)（OFL ライセンス）を採用しています。

```css
font-family: 'GenInterfaceJP', monospace;
```

### フォントの埋め込み（完全自己完結ビルド）

`embed-font` feature を有効にすると、フォント TTF が WASM バイナリに同梱されます。  
日本語フォントが OS に入っていない環境（一部 Linux・組み込み等）での表示を保証します。

```bash
# フォント埋め込みビルド（WASM バイナリ +約 6MB）
wasm-pack build wasm_app --target web -- --features embed-font

# 通常ビルド（外部ファイルから読み込み）
wasm-pack build wasm_app --target web
```

WASM エクスポート（JS から呼び出し可能）：

```js
wasm.engine_font_embedded()  // → boolean: 埋め込み済みかどうか
wasm.engine_font_regular()   // → Uint8Array: Regular フォントバイト列
wasm.engine_font_bold()      // → Uint8Array: Bold フォントバイト列
```

> `FontLoader.load()` がこれらを自動的に呼び出すため、通常は直接使う必要はありません。

---

## 🔊 SoundBuilder — audio_tool.rs

Rust 側でサウンドを定義し、JSON として WASM エクスポートします。  
JS 側は `AudioEngine.play()` に渡すだけで再生できます。

```js
// WASM からサウンド定義を取得（初期化時にキャッシュ）
const soundDef = JSON.parse(wasm.sound_def_maze3d(4)); // 4 = レベルクリア
audio.play(soundDef, volume);

// 全定義を一括取得（デバッグ・ツール用）
const allDefs = JSON.parse(wasm.all_sound_defs_maze3d());
```

| イベント ID | 内容 |
|-------------|------|
| 1 | 足音（左） |
| 2 | 足音（右） |
| 3 | 壁衝突 |
| 4 | レベルクリア |
| 5 | ゴール接近 |
| 6 | 敵接近 |
| 7 | ゲームオーバー |

---

## 🛠️ ビルド方法

### WASM ビルド（通常）

```bash
cd /path/to/rustgames

wasm-pack build wasm_app --target web --out-dir docs/play/maze3d/wasm
rm -f docs/play/maze3d/wasm/.gitignore
```

### WASM ビルド（フォント埋め込み）

```bash
wasm-pack build wasm_app --target web --out-dir docs/play/maze3d/wasm -- --features embed-font
rm -f docs/play/maze3d/wasm/.gitignore
```

### ローカル確認

```bash
# docs/ を静的サーバーで配信（Python）
python3 -m http.server 8080 --directory docs

# または Node.js
npx serve docs
```

---

## 🔗 Unity FFI Bridge

Rust ゲームエンジンのロジックを Unity から呼び出す C 互換インターフェース。

```bash
# Windows DLL
cargo build --release -p ffi_bridge --target x86_64-pc-windows-msvc

# macOS dylib
cargo build --release -p ffi_bridge --target x86_64-apple-darwin

# Linux so
cargo build --release -p ffi_bridge
```

```csharp
// Unity C# から呼び出す例
RustGameBridge.game_initialize(1280, 720);
RustGameBridge.game_update(Time.deltaTime);
var state = RustGameBridge.game_get_state();
```

---

## 🧰 技術スタック

| カテゴリ | 採用技術 |
|----------|----------|
| グラフィックス | **wgpu** (WebGPU/WebGL) |
| 言語 | **Rust 1.70+** |
| WASM バインディング | **wasm-bindgen** |
| 数学 | **glam** |
| シェーダー | **WGSL** |
| JS エンジン | **engine.js** (ES Modules) |
| フォント | **Gen Interface JP** (OFL) |
| Unity 統合 | **FFI** (C 互換 ABI) |

---

## 📝 ライセンス

MIT License

フォント（Gen Interface JP）: [OFL (SIL Open Font License)](https://openfontlicense.org/)

---

**バージョン**: 0.2.0  
**開発環境**: Rust 1.70+, wasm-pack, Node.js (optional)
