// シェーダーモジュール: WGSLシェーダーコード文字列を保持する
//
// col.a encodes material:
//   1.0  = normal (floor/ceiling/pillar/start/goal)
//   2.0  = wall  → receives point-light shading + subtle grain
//   3.0  = particle → always bright, bypass lighting
//
// Uniforms: vp(64) + time(4) + warp(4) + pad(8) + 4×Light(32each) + fog_col(16) = 224 bytes
// Light layout: pos.xyz=worldpos  pos.w=flicker_phase  col.rgb=color  col.a=intensity

pub const SHADER: &str = r#"
struct Light {
    pos : vec4<f32>,
    col : vec4<f32>,
}
struct Uni {
    vp     : mat4x4<f32>,
    time   : f32,
    warp   : f32,
    pad0   : f32,
    pad1   : f32,
    lights : array<Light, 4>,
    fog_col: vec4<f32>,
}
@group(0) @binding(0) var<uniform> u: Uni;

struct VIn {
    @location(0) pos : vec3<f32>,
    @location(1) col : vec4<f32>,
}
struct VOut {
    @builtin(position) clip     : vec4<f32>,
    @location(0)       col      : vec4<f32>,
    @location(1)       depth    : f32,
    @location(2)       world_y  : f32,
    @location(3)       world_xz : vec2<f32>,
}

@vertex
fn vs_main(v: VIn) -> VOut {
    var o: VOut;
    let c     = u.vp * vec4<f32>(v.pos, 1.0);
    o.clip     = c;
    o.col      = v.col;
    o.depth    = c.w;
    o.world_y  = v.pos.y;
    o.world_xz = vec2<f32>(v.pos.x, v.pos.z);
    return o;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
    var rgb = v.col.rgb;

    // ── Particles: emissive, just fog-dim ──────────────────────────────────────
    if v.col.a > 2.5 {
        let fog = max(exp(-0.18 * 0.18 * v.depth * v.depth), 0.35);
        return vec4<f32>(min(rgb * fog, vec3<f32>(1.0)), 1.0);
    }

    // ── Point-light accumulation ───────────────────────────────────────────────
    let wpos = vec3<f32>(v.world_xz.x, v.world_y, v.world_xz.y);
    var light_acc = vec3<f32>(0.0);
    for (var i = 0; i < 4; i++) {
        let lpos    = u.lights[i].pos.xyz;
        let lcol    = u.lights[i].col.rgb;
        let lint    = u.lights[i].col.a;
        let phase   = u.lights[i].pos.w;
        // gentle flicker
        let flicker = sin(u.time * 2.2 + phase * 6.283) * 0.10 + 0.90;
        let dist    = length(lpos - wpos);
        let att     = lint * flicker / (1.0 + dist * dist * 0.55);
        light_acc  += lcol * att;
    }

    // ambient: dim so lights stand out
    let ambient = 0.09;

    // ── Walls: subtle hash grain + lighting ───────────────────────────────────
    if v.col.a > 1.5 {
        // noise grain: tiny variation that breaks up flat look
        let gu = floor(v.world_xz.x * 5.0) + floor(v.world_y * 6.0) * 11.0;
        let gv = floor(v.world_xz.y * 5.0) + floor(v.world_y * 6.0) * 11.0;
        let grain = fract(sin(gu * 127.1 + gv * 311.7) * 43758.5) * 0.08 + 0.92;
        rgb = rgb * grain;
    }

    // lighting × fog  (fog_col.a encodes density scale: 1.0=maze, 0.0=space)
    let fog_density = 0.18 * u.fog_col.a;
    let fog = exp(-fog_density * fog_density * v.depth * v.depth);
    var fog_final: f32;
    if fog_density < 0.001 {
        fog_final = 1.0;  // space: no fog attenuation
    } else {
        let fog_floor = clamp(v.world_y * 2.5, 0.0, 1.0);
        fog_final = fog * (0.75 + fog_floor * 0.25);
    }
    let lit = rgb * (ambient + light_acc);
    rgb = mix(u.fog_col.rgb, lit, fog_final);

    // ── Warp: chromatic distortion + flash ────────────────────────────────────
    if u.warp > 0.01 {
        let flicker = sin(u.time * 35.0) * 0.5 + 0.5;
        let flash   = u.warp * flicker * 0.45;
        let shift   = u.warp * sin(v.depth * 0.5 + u.time * 12.0) * 0.12;
        rgb = clamp(rgb + vec3<f32>(flash + shift, flash * 0.3, flash - shift),
                    vec3<f32>(0.0), vec3<f32>(1.0));
    }

    return vec4<f32>(rgb, 1.0);
}
"#;
