// audio_tool.rs — サウンドビルダー: 音のパラメータをコードで定義する

/// オシレーターの波形種別
#[derive(Clone, Copy)]
pub enum WaveType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

impl WaveType {
    pub fn as_str(&self) -> &'static str {
        match self {
            WaveType::Sine     => "sine",
            WaveType::Square   => "square",
            WaveType::Sawtooth => "sawtooth",
            WaveType::Triangle => "triangle",
        }
    }
}

/// 単一オシレーターの音定義
#[derive(Clone)]
pub struct OscDef {
    pub wave:       WaveType,
    pub freq_start: f32,  // Hz
    pub freq_end:   f32,  // Hz (同じなら固定音程)
    pub freq_time:  f32,  // 周波数変化にかける時間(秒)
    pub gain_start: f32,  // 0.0-1.0
    pub gain_end:   f32,  // 0.0 (通常フェードアウト)
    pub duration:   f32,  // 秒
    pub detune:     f32,  // セント (0=なし)
    pub osc_delay:  f32,  // このオシレーター固有の遅延(秒) アルペジオ等に使用
}

impl OscDef {
    pub fn new(wave: WaveType, freq: f32, dur: f32, gain: f32) -> Self {
        Self {
            wave, freq_start: freq, freq_end: freq, freq_time: dur,
            gain_start: gain, gain_end: 0.0, duration: dur,
            detune: 0.0, osc_delay: 0.0,
        }
    }
    pub fn with_sweep(mut self, freq_end: f32, freq_time: f32) -> Self {
        self.freq_end = freq_end; self.freq_time = freq_time; self
    }
    pub fn with_gain_end(mut self, gain_end: f32) -> Self {
        self.gain_end = gain_end; self
    }
    pub fn with_detune(mut self, detune: f32) -> Self {
        self.detune = detune; self
    }
    pub fn with_osc_delay(mut self, osc_delay: f32) -> Self {
        self.osc_delay = osc_delay; self
    }
}

/// 複数オシレーターの合成サウンド定義
#[derive(Clone)]
pub struct SoundDef {
    pub name:  &'static str,
    pub oscs:  Vec<OscDef>,
    pub delay: f32,  // 発音遅延(秒)
}

impl SoundDef {
    pub fn new(name: &'static str) -> Self {
        Self { name, oscs: Vec::new(), delay: 0.0 }
    }
    pub fn add(mut self, osc: OscDef) -> Self {
        self.oscs.push(osc); self
    }
    pub fn with_delay(mut self, delay: f32) -> Self {
        self.delay = delay; self
    }
    /// JSON文字列に変換（JS側に渡す）
    pub fn to_json(&self) -> String {
        let oscs_json: Vec<String> = self.oscs.iter().map(|o| {
            format!(
                r#"{{"wave":"{}","freqStart":{},"freqEnd":{},"freqTime":{},"gainStart":{},"gainEnd":{},"duration":{},"detune":{},"oscDelay":{}}}"#,
                o.wave.as_str(), o.freq_start, o.freq_end, o.freq_time,
                o.gain_start, o.gain_end, o.duration, o.detune, o.osc_delay
            )
        }).collect();
        format!(r#"{{"name":"{}","delay":{},"oscs":[{}]}}"#,
            self.name, self.delay, oscs_json.join(","))
    }
}

// ── ゲーム用サウンド定義一覧 ────────────────────────────────────────────────

/// 左足音: triangle 120Hz→80Hz sweep, 0.08秒
pub fn sound_step_left() -> SoundDef {
    SoundDef::new("step_left")
        .add(OscDef::new(WaveType::Triangle, 120.0, 0.08, 0.15)
            .with_sweep(80.0, 0.08))
}

/// 右足音: triangle 100Hz→65Hz sweep, 0.08秒
pub fn sound_step_right() -> SoundDef {
    SoundDef::new("step_right")
        .add(OscDef::new(WaveType::Triangle, 100.0, 0.08, 0.12)
            .with_sweep(65.0, 0.08))
}

/// 壁衝突: sawtooth 180Hz→60Hz + square 90Hz(短め)
pub fn sound_wall_hit() -> SoundDef {
    SoundDef::new("wall_hit")
        .add(OscDef::new(WaveType::Sawtooth, 180.0, 0.12, 0.2)
            .with_sweep(60.0, 0.12))
        .add(OscDef::new(WaveType::Square, 90.0, 0.06, 0.1))
}

/// レベルクリア: sine 3和音アルペジオ C5-E5-G5 (osc_delayで時差発音)
pub fn sound_level_clear() -> SoundDef {
    SoundDef::new("level_clear")
        .add(OscDef::new(WaveType::Sine, 523.25, 0.3, 0.3)
            .with_osc_delay(0.0))
        .add(OscDef::new(WaveType::Sine, 659.25, 0.3, 0.3)
            .with_osc_delay(0.1))
        .add(OscDef::new(WaveType::Sine, 783.99, 0.3, 0.3)
            .with_osc_delay(0.2))
}

/// ゴール接近: sine 880Hz→1760Hz sweep 0.2秒
pub fn sound_goal_near() -> SoundDef {
    SoundDef::new("goal_near")
        .add(OscDef::new(WaveType::Sine, 880.0, 0.2, 0.2)
            .with_sweep(1760.0, 0.2))
}

/// 敵接近: square 55Hz→110Hz sweep 0.15秒
pub fn sound_enemy_near() -> SoundDef {
    SoundDef::new("enemy_near")
        .add(OscDef::new(WaveType::Square, 55.0, 0.15, 0.3)
            .with_sweep(110.0, 0.15))
}

/// ゲームオーバー: sawtooth 2重下降 (0.05秒差)
pub fn sound_game_over() -> SoundDef {
    SoundDef::new("game_over")
        .add(OscDef::new(WaveType::Sawtooth, 220.0, 0.8, 0.4)
            .with_sweep(55.0, 0.8)
            .with_osc_delay(0.0))
        .add(OscDef::new(WaveType::Sawtooth, 185.0, 0.8, 0.4)
            .with_sweep(46.0, 0.8)
            .with_osc_delay(0.05))
}
