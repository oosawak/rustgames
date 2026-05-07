---
layout: default
title: API リファレンス
---

# API リファレンス

## engine.js — JS ゲームエンジンライブラリ

`docs/engine.js` は ES Modules 形式のライブラリです。

```js
import { FontLoader, AudioEngine, InputManager, CanvasManager } from '../../engine.js';
```

---

### FontLoader

Gen Interface JP フォントを自動ロードして `document.fonts` に登録します。

```js
// 基本的な使い方（wasm モジュールを渡すだけ）
await FontLoader.load('GenInterfaceJP', wasm);

// カスタム URL を指定
await FontLoader.load('GenInterfaceJP', wasm, { fontUrl: 'https://cdn.example.com/font.ttf' });
```

**フォント取得の優先順位:**

| 優先度 | 条件 | 取得元 |
|--------|------|--------|
| 1 | `embed-font` feature ビルド | WASM バイナリ内バイト列 → Blob URL |
| 2 | `fontUrl` オプション指定あり | 指定 URL |
| 3 | デフォルト | `engine.js` 隣の `fonts/` (import.meta.url で自動解決) |

---

### AudioEngine

```js
const audio = new AudioEngine();        // autoAmbient: true (デフォルト)
audio.ensure();                          // AudioContext を遅延初期化
audio.play(soundDefJson, volume);        // SoundDef JSON を再生
audio.startAmbient({ freqs: [55, 55.5], filterFreq: 180, fadeIn: 3.0 });
audio.setSeVolume(0.8);
audio.setAmbVolume(0.4);
audio.seVolume                           // 現在の SE 音量
audio.ambVolume                          // 現在のアンビエント音量
```

---

### InputManager

```js
const input = new InputManager({ swipeMin: 35 });

// アクションコールバック登録 (0=前 1=左折 2=右折 3=後)
input.onAction(action => wasm.move_game(action));

// キーボード
input.bindKeyboard({ ArrowUp:0, KeyW:0, ArrowLeft:1, KeyA:1,
                     ArrowRight:2, KeyD:2, ArrowDown:3, KeyS:3 });

// D-pad ボタン
input.bindDpad([['db-up',0],['db-left',1],['db-right',2],['db-down',3]],
               () => audio.ensure());

// スワイプ
input.bindSwipe(['#hud','#dpad'], { onHint: flashHint, onStart: () => audio.ensure() });
```

---

### CanvasManager

```js
const canvas = new CanvasManager('game-canvas', {
  hudHeight: 44,     // HUD の高さ px
  dpadHeight: 160,   // D-pad の高さ px
  mobileBreak: 600,  // モバイル判定幅 px
});
canvas.resize();       // 即時リサイズ
canvas.bindResize();   // window.resize に自動バインド
```

---

## WASM エクスポート API (Neon Maze 3D)

### 初期化・ゲームループ

| 関数 | 説明 |
|------|------|
| `init_maze3d(canvasId)` | ゲーム初期化 |
| `tick_maze3d(ts)` | 毎フレーム更新 (requestAnimationFrame のタイムスタンプを渡す) |
| `start_game_maze3d()` | ゲーム開始 / リセット |
| `next_level_maze3d()` | 次のレベルへ |
| `reset_maze3d()` | リセット |

### 入力

| 関数 | 説明 |
|------|------|
| `move_maze3d(action)` | 移動 (0=前 1=左折 2=右折 3=後) |

### 状態取得

| 関数 | 戻り値 | 説明 |
|------|--------|------|
| `scene_maze3d()` | u8 | 0=Title 1=Playing 2=LevelClear 3=GameOver |
| `steps_maze3d()` | u32 | 現レベルの歩数 |
| `total_steps_maze3d()` | u32 | 合計歩数 |
| `level_maze3d()` | u32 | 現在のレベル |
| `theme_name_maze3d()` | String | 壁テーマ名 |
| `warp_maze3d()` | f32 | ワープエフェクト量 (0.0〜1.0) |
| `warp_done_maze3d()` | bool | ワープ完了フラグ |
| `maze_data_maze3d()` | Uint8Array | 迷路データ (9×9) |
| `player_x_maze3d()` | u32 | プレイヤー X 座標 |
| `player_z_maze3d()` | u32 | プレイヤー Z 座標 |
| `player_facing_maze3d()` | u8 | 向き (1=N 2=E 4=S 8=W) |
| `enemy_x_maze3d()` | i32 | 敵 X 座標 (-1=非存在) |
| `enemy_z_maze3d()` | i32 | 敵 Z 座標 |

### スコア保存 (localStorage)

| 関数 | 説明 |
|------|------|
| `best_steps_maze3d()` | ベスト歩数を取得 |
| `best_level_maze3d()` | 最高レベルを取得 |
| `play_count_maze3d()` | プレイ回数を取得 |
| `load_se_vol_maze3d()` | SE 音量を取得 |
| `load_amb_vol_maze3d()` | アンビエント音量を取得 |
| `save_audio_vol_maze3d(se, amb)` | 音量を保存 |

### サウンド

| 関数 | 説明 |
|------|------|
| `audio_event_maze3d()` | 音声イベントを取得 (0=なし) |
| `audio_step_parity_maze3d()` | 足音左右フラグ |
| `sound_def_maze3d(event)` | SoundDef JSON 文字列 |
| `all_sound_defs_maze3d()` | 全 SoundDef JSON 配列 |

### フォント (embed-font feature)

| 関数 | 説明 |
|------|------|
| `engine_font_embedded()` | フォントが埋め込まれているか |
| `engine_font_regular()` | Regular フォントバイト列 |
| `engine_font_bold()` | Bold フォントバイト列 |
