use crate::audio_tool::{OscDef, SoundDef, WaveType};

pub fn sound_shoot() -> SoundDef {
    SoundDef::new("shoot").add(
        OscDef::new(WaveType::Square, 880.0, 0.08, 0.3).with_sweep(440.0, 0.08)
    )
}

pub fn sound_enemy_hit() -> SoundDef {
    SoundDef::new("enemy_hit").add(
        OscDef::new(WaveType::Sawtooth, 220.0, 0.12, 0.4)
            .with_sweep(110.0, 0.12).with_detune(10.0)
    )
}

pub fn sound_player_hit() -> SoundDef {
    SoundDef::new("player_hit")
        .add(OscDef::new(WaveType::Sawtooth, 150.0, 0.3, 0.5).with_sweep(60.0, 0.3).with_detune(20.0))
        .add(OscDef::new(WaveType::Square, 300.0, 0.25, 0.3).with_sweep(100.0, 0.25))
}

pub fn sound_explosion() -> SoundDef {
    SoundDef::new("explosion")
        .add(OscDef::new(WaveType::Sawtooth, 200.0, 0.4, 0.6).with_sweep(30.0, 0.4).with_detune(30.0))
        .add(OscDef::new(WaveType::Square, 400.0, 0.35, 0.4).with_sweep(50.0, 0.35))
}

pub fn sound_stage_clear() -> SoundDef {
    SoundDef::new("stage_clear")
        .add(OscDef::new(WaveType::Sine, 523.0, 0.15, 0.5).with_osc_delay(0.0))
        .add(OscDef::new(WaveType::Sine, 659.0, 0.15, 0.5).with_osc_delay(0.18))
        .add(OscDef::new(WaveType::Sine, 784.0, 0.2, 0.5).with_osc_delay(0.36))
}

pub fn sound_game_over() -> SoundDef {
    SoundDef::new("game_over")
        .add(OscDef::new(WaveType::Sawtooth, 440.0, 0.8, 0.5).with_sweep(110.0, 0.8).with_detune(15.0))
        .add(OscDef::new(WaveType::Square, 220.0, 0.7, 0.4).with_sweep(55.0, 0.7).with_osc_delay(0.1))
}

pub fn sound_boss_appear() -> SoundDef {
    SoundDef::new("boss_appear")
        .add(OscDef::new(WaveType::Sawtooth, 55.0, 1.0, 0.6).with_sweep(220.0, 1.0).with_detune(20.0))
        .add(OscDef::new(WaveType::Square, 110.0, 1.0, 0.4).with_sweep(440.0, 1.0).with_osc_delay(0.2))
}
