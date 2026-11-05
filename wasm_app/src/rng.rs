// rng.rs — JavaScript 実装と同一の乱数列を再現するためのユーティリティ
//
// html-player 版 CubeBoy / LINE BOY は
//   - FNV-1a ハッシュ（文字列シード）
//   - mulberry32 系の RNG: a += 0x6D2B79F5; t = imul(a ^ a>>>15, 1|a); ...
// を使って部屋・地形を決定的に生成している。Rust 側でも同じレベルを
// 再現するため、JS の 32bit 演算（Math.imul / >>>）をそのまま移植する。

/// JS の文字列シードハッシュ（FNV-1a, 32bit）
/// charCodeAt は UTF-16 コード単位なので encode_utf16 で辿る。
pub fn hash_seed(s: &str) -> u32 {
    let mut h: u32 = 2166136261;
    for c in s.encode_utf16() {
        h ^= c as u32;
        h = h.wrapping_mul(16777619);
    }
    h
}

fn imul(a: u32, b: u32) -> u32 {
    a.wrapping_mul(b)
}

/// mulberry32（JS 実装と同一手順）
pub struct JsRng {
    a: u32,
}

impl JsRng {
    pub fn new(seed: u32) -> Self {
        JsRng { a: seed }
    }

    /// 0.0 <= x < 1.0
    pub fn next_f64(&mut self) -> f64 {
        self.a = self.a.wrapping_add(0x6D2B79F5);
        let a = self.a;
        let mut t = imul(a ^ (a >> 15), 1 | a);
        t = (t.wrapping_add(imul(t ^ (t >> 7), 61 | t))) ^ t;
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }

    /// a..=b の整数（JS の ri と同じ）
    pub fn ri(&mut self, a: i32, b: i32) -> i32 {
        a + (self.next_f64() * ((b - a + 1) as f64)).floor() as i32
    }

    /// a..b の実数（JS の rr / RU と同じ）
    pub fn rr(&mut self, a: f64, b: f64) -> f64 {
        a + self.next_f64() * (b - a)
    }
}

/// Math.random() の代用（決定的でなくてよい場面用：パーティクル等）
pub struct GameRng {
    s: u64,
}

impl GameRng {
    pub fn new() -> Self {
        let seed = (js_sys::Date::now() as u64) ^ 0x9E3779B97F4A7C15;
        GameRng { s: seed | 1 }
    }

    /// 0.0 <= x < 1.0
    pub fn next_f64(&mut self) -> f64 {
        // xorshift64*
        let mut x = self.s;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.s = x;
        let v = x.wrapping_mul(0x2545F4914F6CDD1D);
        ((v >> 11) as f64) / ((1u64 << 53) as f64)
    }

    pub fn range(&mut self, a: f64, b: f64) -> f64 {
        a + self.next_f64() * (b - a)
    }

    pub fn range_f32(&mut self, a: f32, b: f32) -> f32 {
        self.range(a as f64, b as f64) as f32
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::new()
    }
}
