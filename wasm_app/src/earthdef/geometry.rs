#![allow(dead_code)]

use std::f32::consts::PI;
use std::f32::consts::TAU;
use crate::geometry::{Vertex, Uni, Light};
use crate::math::{perspective, look_at, mat_mul};
use super::state::{EarthDefGame, EnemyKind, LaserType};

const CORNERS: [(f32, f32, f32); 8] = [
    (-1.0,-1.0,-1.0), (1.0,-1.0,-1.0), (1.0,1.0,-1.0), (-1.0,1.0,-1.0),
    (-1.0,-1.0, 1.0), (1.0,-1.0, 1.0), (1.0,1.0, 1.0), (-1.0,1.0, 1.0),
];

const FACES: [(u32, u32, u32, u32); 6] = [
    (0,1,2,3), (4,7,6,5), (0,4,5,1), (2,6,7,3), (0,3,7,4), (1,5,6,2),
];

fn rotate_y(x: f32, z: f32, a: f32) -> (f32, f32) {
    let (sa, ca) = (a.sin(), a.cos());
    (x*ca + z*sa, -x*sa + z*ca)
}

fn rotate_x(y: f32, z: f32, a: f32) -> (f32, f32) {
    let (sa, ca) = (a.sin(), a.cos());
    (y*ca - z*sa, y*sa + z*ca)
}

fn rotate_xyz(lx: f32, ly: f32, lz: f32, rx: f32, ry: f32, _rz: f32) -> (f32, f32, f32) {
    let (x1, z1) = rotate_y(lx, lz, ry);
    let (y2, z2) = rotate_x(ly, z1, rx);
    (x1, y2, z2)
}

fn push_rotated_box(
    verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>,
    cx: f32, cy: f32, cz: f32,
    sx: f32, sy: f32, sz: f32,
    rx: f32, ry: f32, rz: f32,
    col: [f32; 4],
) {
    let base = verts.len() as u32;
    for &(lx, ly, lz) in &CORNERS {
        let (vx, vy, vz) = rotate_xyz(lx*sx, ly*sy, lz*sz, rx, ry, rz);
        verts.push(Vertex { pos: [cx+vx, cy+vy, cz+vz], _p: 0.0, col });
    }
    for &(a, b, c, d) in &FACES {
        idxs.extend_from_slice(&[base+a, base+b, base+c, base+a, base+c, base+d]);
    }
}

fn push_box(
    verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>,
    cx: f32, cy: f32, cz: f32,
    sx: f32, sy: f32, sz: f32,
    col: [f32; 4],
) {
    push_rotated_box(verts, idxs, cx, cy, cz, sx, sy, sz, 0.0, 0.0, 0.0, col);
}

/// UV sphere for Earth with land/ocean coloring
fn push_earth_sphere(
    verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>,
    cx: f32, cy: f32, cz: f32,
    radius: f32,
    earth_rot: f32,
    flash: f32,
    damage_ratio: f32,
) {
    const NLAT: u32 = 12;
    const NLON: u32 = 18;
    let base = verts.len() as u32;

    for i in 0..=NLAT {
        let phi = PI * i as f32 / NLAT as f32 - PI * 0.5;
        let cp = phi.cos();
        let sp = phi.sin();
        for j in 0..=NLON {
            let theta = TAU * j as f32 / NLON as f32;
            let x = cx + radius * cp * theta.sin();
            let y = cy + radius * sp;
            let z = cz + radius * cp * theta.cos();

            let col = if flash > 0.0 {
                [1.0, flash * 0.3, flash * 0.3, 1.0]
            } else {
                let lng = theta + earth_rot;
                let continent = (lng * 1.7).sin() * (phi * 2.3).cos() * 0.7
                              + (lng * 0.9 + 1.1).cos() * 0.3;
                if continent > 0.1 {
                    let ice = (phi.abs() - 1.1).max(0.0) * 3.0;
                    let r = (0.2 + ice * 0.8) * damage_ratio;
                    let g2 = (0.6 + ice * 0.4) * damage_ratio;
                    let b = (0.2 + ice * 0.8) * damage_ratio;
                    [r, g2, b, 1.0]
                } else {
                    let depth = (-continent * 2.0).clamp(0.0, 1.0);
                    let r = (0.05 + depth * 0.05) * damage_ratio;
                    let g2 = (0.25 + depth * 0.1) * damage_ratio;
                    let b = (0.7 + depth * 0.3) * damage_ratio;
                    [r, g2, b, 1.0]
                }
            };
            verts.push(Vertex { pos: [x, y, z], _p: 0.0, col });
        }
    }

    let stride = NLON + 1;
    for i in 0..NLAT {
        for j in 0..NLON {
            let a = base + i * stride + j;
            let b = base + i * stride + j + 1;
            let c = base + (i+1) * stride + j + 1;
            let d = base + (i+1) * stride + j;
            idxs.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }
}

pub fn build_scene(g: &EarthDefGame) -> (Vec<Vertex>, Vec<u32>) {
    let mut verts: Vec<Vertex> = Vec::with_capacity(4096);
    let mut idxs: Vec<u32> = Vec::with_capacity(8192);

    // Background stars: 200 boxes at radius 30 – twinkle based on game time
    let mut star_rng: u64 = 0xdeadbeef;
    for _ in 0..200 {
        let f1    = crate::math::lcg_f(&mut star_rng);
        let f2    = crate::math::lcg_f(&mut star_rng);
        let phase = crate::math::lcg_f(&mut star_rng);
        let base_b = 0.5 + crate::math::lcg_f(&mut star_rng) * 0.5;
        let hue   = crate::math::lcg_f(&mut star_rng); // color tint
        let theta = f1 * TAU;
        let phi   = f2 * PI - PI * 0.5;
        let r = 30.0;
        let sx = r * phi.cos() * theta.sin();
        let sy = r * phi.sin();
        let sz = r * phi.cos() * theta.cos();
        // Twinkle: random speed and phase per star, including brief dim moments
        let twinkle = (0.3 + 0.7 * ((g.time * (1.5 + phase * 5.0) + phase * TAU).sin() * 0.5 + 0.5));
        let bright = base_b * twinkle;
        // Slight warm/cool tint
        let rc = (bright + hue * 0.25).min(1.0);
        let gc = bright;
        let bc = (bright + (1.0 - hue) * 0.25).min(1.0);
        let size = 0.04 + base_b * 0.06;
        push_box(&mut verts, &mut idxs, sx, sy, sz, size, size, size, [rc, gc, bc, 1.0]);
    }

    // Earth – smooth UV sphere with land/ocean coloring
    let earth_size = 1.0;
    let damage_ratio = (g.earth_hp as f32 / g.earth_max_hp as f32).max(0.2);
    let flash = g.earth_hit_flash;
    push_earth_sphere(&mut verts, &mut idxs, 0.0, 0.0, 0.0,
        earth_size, g.earth_rot, flash, damage_ratio);

    // Enemies
    for e in &g.enemies {
        if !e.active { continue; }
        let base_col: [f32; 4] = match e.kind {
            EnemyKind::Basic    => [0.2, 0.8, 0.3, 1.0],
            EnemyKind::Speed    => [0.9, 0.9, 0.1, 1.0],
            EnemyKind::Armored  => [0.8, 0.2, 0.2, 1.0],
            EnemyKind::Splitter => [0.1, 0.9, 0.9, 1.0],
        };
        // Glow pulse: each enemy pulses independently based on position phase
        let pulse = 0.6 + 0.4 * ((g.time * 5.0 + e.x * 1.3 + e.z * 0.9).sin() * 0.5 + 0.5);
        let col = if e.hit_flash > 0.0 {
            let f = e.hit_flash;
            [
                (base_col[0]*0.5 + f).min(1.0),
                (base_col[1]*0.5 + f*0.5).min(1.0),
                (base_col[2]*0.5 + f*0.5).min(1.0),
                1.0,
            ]
        } else {
            [base_col[0]*pulse, base_col[1]*pulse, base_col[2]*pulse, 1.0]
        };
        push_rotated_box(&mut verts, &mut idxs,
            e.x, e.y, e.z,
            e.scale, e.scale, e.scale,
            e.rot_x, e.rot_y, e.rot_z, col);
    }

    // Laser beams
    for b in &g.beams {
        let w = 0.04;
        let half_len = b.len * 0.5;
        let mid_x = b.ox + b.dx * half_len;
        let mid_y = b.oy + b.dy * half_len;
        let mid_z = b.oz + b.dz * half_len;

        let ry = b.dx.atan2(b.dz);
        let horiz = (b.dx*b.dx + b.dz*b.dz).sqrt();
        let rx = -(b.dy).atan2(horiz);

        let life_alpha = (b.life * 2.0).min(1.0);
        let col = [b.col[0]*life_alpha, b.col[1]*life_alpha, b.col[2]*life_alpha, 1.0];
        push_rotated_box(&mut verts, &mut idxs,
            mid_x, mid_y, mid_z,
            w, w, half_len,
            rx, ry, 0.0, col);
    }

    // Particles
    for p in &g.particles {
        if p.life <= 0.0 { continue; }
        let t = p.life / p.max_life;
        let s = 0.08 * t;
        let col = [
            (p.col[0]*t*2.0).min(1.0),
            (p.col[1]*t*2.0).min(1.0),
            (p.col[2]*t*2.0).min(1.0),
            1.0,
        ];
        push_box(&mut verts, &mut idxs, p.x, p.y, p.z, s, s, s, col);
    }

    // Flash effect
    if g.flash_active > 0.0 {
        let f = g.flash_active;
        let s = 35.0;
        push_box(&mut verts, &mut idxs, 0.0, 0.0, 0.0, s, s, s,
            [(f*2.0).min(1.0), (f*2.0).min(1.0), (f*2.0).min(1.0), 0.3]);
    }

    (verts, idxs)
}

pub fn build_uni(g: &EarthDefGame, asp: f32) -> Uni {
    let eye_x = g.cam_azimuth.sin() * g.cam_elevation.cos() * g.cam_distance;
    let eye_y = g.cam_elevation.sin() * g.cam_distance;
    let eye_z = g.cam_azimuth.cos() * g.cam_elevation.cos() * g.cam_distance;
    let view = look_at([eye_x, eye_y, eye_z], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let proj = perspective(std::f32::consts::FRAC_PI_3, asp, 0.5, 100.0);
    let vp = mat_mul(proj, view);

    let adx = g.aim_elevation.cos() * g.aim_azimuth.sin();
    let ady = g.aim_elevation.sin();
    let adz = g.aim_elevation.cos() * g.aim_azimuth.cos();

    let beam_intensity = if g.beams.is_empty() { 0.0 } else { 2.5 };
    let flash_intensity = if g.flash_active > 0.0 { g.flash_active * 5.0 } else { 0.0 };

    // Suppress unused import warning for LaserType — it's used via state fields
    let _ = LaserType::Beam;

    let lights = [
        Light { pos: [0.0, 8.0, 0.0, 0.0],               col: [1.0, 1.0, 1.0, 12.0] },
        Light { pos: [adx*8.0, ady*8.0, adz*8.0, 0.0],   col: [0.0, 1.0, 1.0, beam_intensity] },
        Light { pos: [0.0, 0.0, -8.0, 0.0],               col: [0.3, 0.5, 1.0, 8.0] },
        Light { pos: [0.0, 0.0, 0.0, 0.0],                col: [1.0, 1.0, 1.0, flash_intensity] },
    ];

    Uni {
        vp,
        time: g.time,
        warp: 0.0,
        pad: [0.0; 2],
        lights,
        fog_col: [0.0, 0.0, 0.02, 0.0],  // a=0.0: no fog (space)
    }
}
