/**
 * RustGames Engine JS
 * WASM ゲーム共通ライブラリ (FontLoader / AudioEngine / InputManager / CanvasManager)
 *
 * 使い方:
 *   import { FontLoader, AudioEngine, InputManager, CanvasManager } from '../../engine.js';
 */

// ── FontLoader ────────────────────────────────────────────────────────────────
/**
 * ゲームエンジン標準フォント (GenInterfaceJP) ロードユーティリティ。
 *
 * フォント取得の優先順位:
 *   1. WASM バイナリ埋め込み (embed-font feature でビルドした場合)
 *   2. 呼び出し元が指定したカスタム URL
 *   3. engine.js と同階層の fonts/ ディレクトリ (import.meta.url で自動解決)
 */
export class FontLoader {
  /**
   * フォントを非同期ロードして document.fonts に登録する。
   * @param {string}      fontName   - CSS font-family 名
   * @param {object|null} wasmModule - wasm-bindgen モジュール (engine_font_embedded 等を含む)
   * @param {object}      opts
   *   @param {string|null} opts.fontUrl - カスタム URL (省略時は自動解決)
   */
  static async load(fontName, wasmModule = null, opts = {}) {
    const { fontUrl = null } = opts;
    let src;

    if (wasmModule?.engine_font_embedded?.()) {
      // 1. WASM 埋め込みバイト列 → Blob URL に変換
      const bytes = wasmModule.engine_font_regular();
      const blob  = new Blob([bytes], { type: 'font/ttf' });
      src = `url(${URL.createObjectURL(blob)})`;
    } else if (fontUrl) {
      // 2. 開発者が指定した URL
      src = `url(${fontUrl})`;
    } else {
      // 3. engine.js と同じディレクトリの fonts/ を自動解決
      //    例: https://example.com/docs/engine.js → https://example.com/docs/fonts/GenInterfaceJP-Regular.ttf
      const resolved = new URL(
        `fonts/${fontName.replace(/\s+/g, '')}-Regular.ttf`,
        import.meta.url
      );
      src = `url(${resolved})`;
    }

    try {
      const face = new FontFace(fontName, src);
      await face.load();
      document.fonts.add(face);
    } catch (e) {
      console.warn(`[engine] FontLoader: ${fontName} の読み込みに失敗しました`, e);
    }
  }
}

// ── AudioEngine ───────────────────────────────────────────────────────────────
/**
 * Web Audio API ラッパー。
 * Rust の audio_tool.rs が生成した SoundDef JSON を受け取って再生する。
 */
export class AudioEngine {
  /**
   * @param {object} opts
   *   @param {boolean} opts.autoAmbient - ensure() 初回呼び出し時にアンビエントを自動開始 (デフォルト true)
   */
  constructor(opts = {}) {
    this._ctx          = null;
    this._ambientNode  = null;
    this._ambientGain  = null;
    this._autoAmbient  = opts.autoAmbient ?? true;
    this._ambientOpts  = null; // startAmbient() に渡すオプションを保持
    this.seVolume      = 1.0;
    this.ambVolume     = 0.4;
  }

  /**
   * AudioContext を遅延初期化し、suspended なら再開する。
   * 初回呼び出し時に autoAmbient が true なら startAmbient() も呼ぶ。
   */
  ensure() {
    const isNew = !this._ctx;
    if (!this._ctx) {
      this._ctx = new (window.AudioContext || window.webkitAudioContext)();
    }
    if (this._ctx.state === 'suspended') this._ctx.resume();
    if (isNew && this._autoAmbient) this.startAmbient(this._ambientOpts ?? {});
    return this._ctx;
  }

  /** AudioContext インスタンスを返す（未初期化時は null） */
  get ctx() { return this._ctx; }

  /**
   * SoundDef JSON を再生する（audio_tool.rs が生成したもの）。
   * @param {object} def - JSON.parse(wasm.sound_def_xxx(event)) の結果
   * @param {number} vol - ボリュームスケール
   */
  play(def, vol = 1.0) {
    if (!this._ctx || !def?.oscs) return;
    const baseT = this._ctx.currentTime + (def.delay ?? 0);
    for (const osc of def.oscs) {
      const t    = baseT + (osc.oscDelay ?? 0);
      const node = this._ctx.createOscillator();
      const gain = this._ctx.createGain();
      node.connect(gain);
      gain.connect(this._ctx.destination);
      node.type = osc.wave;
      node.frequency.setValueAtTime(osc.freqStart, t);
      if (osc.freqEnd !== osc.freqStart)
        node.frequency.linearRampToValueAtTime(osc.freqEnd, t + osc.freqTime);
      if (osc.detune) node.detune.setValueAtTime(osc.detune, t);
      gain.gain.setValueAtTime(osc.gainStart * vol, t);
      gain.gain.linearRampToValueAtTime(osc.gainEnd * vol, t + osc.duration);
      node.start(t);
      node.stop(t + osc.duration + 0.01);
    }
  }

  /**
   * アンビエントドローンを開始する（重複呼び出しは無視）。
   * @param {object} opts
   *   @param {number[]} opts.freqs      - ドローン周波数 Hz 配列 (デフォルト [55, 55.5])
   *   @param {number}   opts.filterFreq - LPF カットオフ Hz (デフォルト 180)
   *   @param {number}   opts.fadeIn     - フェードイン秒数 (デフォルト 3.0)
   */
  startAmbient(opts = {}) {
    if (!this._ctx || this._ambientNode) return;
    const { freqs = [55, 55.5], filterFreq = 180, fadeIn = 3.0 } = opts;

    this._ambientGain = this._ctx.createGain();
    this._ambientGain.gain.setValueAtTime(0.0, this._ctx.currentTime);
    this._ambientGain.gain.linearRampToValueAtTime(
      0.06 * this.ambVolume, this._ctx.currentTime + fadeIn
    );
    this._ambientGain.connect(this._ctx.destination);

    for (const freq of freqs) {
      const osc = this._ctx.createOscillator();
      const lpf = this._ctx.createBiquadFilter();
      lpf.type = 'lowpass';
      lpf.frequency.value = filterFreq;
      lpf.Q.value = 2;
      osc.type = 'sawtooth';
      osc.frequency.value = freq;
      osc.connect(lpf);
      lpf.connect(this._ambientGain);
      osc.start();
    }
    this._ambientNode = this._ambientGain;
  }

  /** アンビエント音量を変更する */
  setAmbVolume(v) {
    this.ambVolume = v;
    if (this._ambientGain && this._ctx)
      this._ambientGain.gain.setValueAtTime(0.06 * v, this._ctx.currentTime);
  }

  /** SE 音量を変更する */
  setSeVolume(v) { this.seVolume = v; }
}

// ── InputManager ──────────────────────────────────────────────────────────────
/**
 * キーボード / D-pad / タッチスワイプ 入力を統一管理する。
 * onAction(callback) でアクションインデックスを受け取る。
 */
export class InputManager {
  /**
   * @param {object} opts
   *   @param {number} opts.swipeMin - スワイプ最小移動距離 px (デフォルト 35)
   */
  constructor(opts = {}) {
    this._swipeMin  = opts.swipeMin ?? 35;
    this._callbacks = [];
    this._repeatId  = null;
  }

  /** アクション発火時のコールバックを登録する */
  onAction(fn) { this._callbacks.push(fn); }

  _emit(action) { this._callbacks.forEach(fn => fn(action)); }

  _startRepeat(action) {
    this._stopRepeat();
    this._emit(action);
    this._repeatId = setInterval(() => this._emit(action), 200);
  }
  _stopRepeat() {
    if (this._repeatId !== null) { clearInterval(this._repeatId); this._repeatId = null; }
  }

  /**
   * キーボード入力をバインドする。
   * @param {object} keymap - { 'ArrowUp': 0, 'KeyW': 0, ... }
   */
  bindKeyboard(keymap) {
    document.addEventListener('keydown', e => {
      if (e.code in keymap) { e.preventDefault(); this._emit(keymap[e.code]); }
    });
  }

  /**
   * D-pad ボタンをバインドする（ホールド長押しリピート付き）。
   * @param {Array<[string, number]>} pairs   - [['db-up', 0], ['db-left', 1], ...]
   * @param {function|null}           onStart - 押下時コールバック (例: audio.ensure.bind(audio))
   */
  bindDpad(pairs, onStart = null) {
    for (const [id, action] of pairs) {
      const btn = document.getElementById(id);
      if (!btn) continue;
      btn.addEventListener('mousedown',  () => { onStart?.(); this._startRepeat(action); });
      btn.addEventListener('mouseup',    () => this._stopRepeat());
      btn.addEventListener('mouseleave', () => this._stopRepeat());
      btn.addEventListener('touchstart', e => {
        e.preventDefault(); this._startRepeat(action);
      }, { passive: false });
      btn.addEventListener('touchend', e => {
        e.preventDefault(); this._stopRepeat();
      }, { passive: false });
    }
  }

  /**
   * 全画面スワイプ入力をバインドする。
   * @param {string[]} excludeSelectors - スワイプを無視するセレクタ (例: ['#hud', '#dpad'])
   * @param {object}   opts
   *   @param {function|null} opts.onHint  - スワイプ時ビジュアルヒント関数 (action) => void
   *   @param {function|null} opts.onStart - touchstart コールバック
   */
  bindSwipe(excludeSelectors = [], opts = {}) {
    const { onHint = null, onStart = null } = opts;
    const exclude = excludeSelectors.join(', ');
    let x0 = 0, y0 = 0, moved = false;

    document.addEventListener('touchstart', e => {
      if (exclude && e.target.closest(exclude)) return;
      onStart?.();
      x0 = e.touches[0].clientX;
      y0 = e.touches[0].clientY;
      moved = false;
    }, { passive: true });

    document.addEventListener('touchmove', e => {
      if (exclude && e.target.closest(exclude)) return;
      const dx = e.touches[0].clientX - x0;
      const dy = e.touches[0].clientY - y0;
      if (!moved && (Math.abs(dx) > this._swipeMin || Math.abs(dy) > this._swipeMin)) {
        moved = true;
        const action = Math.abs(dx) > Math.abs(dy)
          ? (dx < 0 ? 1 : 2)
          : (dy < 0 ? 0 : 3);
        onHint?.(action);
        this._emit(action);
      }
    }, { passive: true });

    document.addEventListener('touchend', () => { moved = false; }, { passive: true });
  }
}

// ── CanvasManager ─────────────────────────────────────────────────────────────
/**
 * Canvas のサイズをウィンドウに合わせて管理する。
 */
export class CanvasManager {
  /**
   * @param {string} canvasId
   * @param {object} opts
   *   @param {number} opts.hudHeight   - HUD の高さ px (デフォルト 44)
   *   @param {number} opts.dpadHeight  - D-pad の高さ px (デフォルト 160)
   *   @param {number} opts.mobileBreak - モバイル判定幅 px (デフォルト 600)
   */
  constructor(canvasId, opts = {}) {
    this._id     = canvasId;
    this._hudH   = opts.hudHeight   ?? 44;
    this._dpadH  = opts.dpadHeight  ?? 160;
    this._breakW = opts.mobileBreak ?? 600;
  }

  /** Canvas をウィンドウサイズに合わせてリサイズする */
  resize() {
    const c = document.getElementById(this._id);
    if (!c) return;
    const dpadH = window.innerWidth < this._breakW ? 0 : this._dpadH;
    c.width  = window.innerWidth;
    c.height = window.innerHeight - this._hudH - dpadH;
  }

  /** window resize イベントに自動バインドする */
  bindResize() {
    window.addEventListener('resize', () => this.resize());
  }
}
