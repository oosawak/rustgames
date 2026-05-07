use crate::geometry::{Vertex, Light, Uni};
use crate::math::{mat_mul, perspective, look_at};
use super::state::BlasterGame;
use super::camera_mode::{CameraMode, camera_view};

fn push_box(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>,
            cx: f32, cy: f32, cz: f32,
            sx: f32, sy: f32, sz: f32,
            col: [f32; 4]) {
    let base = verts.len() as u32;
    let (x0, x1) = (cx - sx, cx + sx);
    let (y0, y1) = (cy - sy, cy + sy);
    let (z0, z1) = (cz - sz, cz + sz);
    for &(x, y, z) in &[
        (x0,y0,z0),(x1,y0,z0),(x1,y1,z0),(x0,y1,z0),
        (x0,y0,z1),(x1,y0,z1),(x1,y1,z1),(x0,y1,z1),
    ] {
        verts.push(Vertex { pos: [x, y, z], _p: 0.0, col });
    }
    for &(a, b, c, d) in &[
        (0u32,1,2,3),(4,7,6,5),(0,4,5,1),(2,6,7,3),(0,3,7,4),(1,5,6,2)
    ] {
        idxs.extend_from_slice(&[base+a,base+b,base+c, base+a,base+c,base+d]);
    }
}

pub fn build_blaster_scene(g: &BlasterGame) -> (Vec<Vertex>, Vec<u32>) {
    let mut verts: Vec<Vertex> = Vec::with_capacity(4096);
    let mut idxs:  Vec<u32>   = Vec::with_capacity(8192);

    // ── アリーナ床 ──────────────────────────────────────────────────────────
    {
        let base = verts.len() as u32;
        let s = 10.0f32;
        let fc = [0.02f32, 0.02, 0.08, 1.0];
        for &(x, z) in &[(-s, s), (s, s), (s, -s), (-s, -s)] {
            verts.push(Vertex { pos: [x, -0.05, z], _p: 0.0, col: fc });
        }
        idxs.extend_from_slice(&[base, base+1, base+2, base, base+2, base+3]);
    }

    // ── グリッド線（ネオン cyan・間引いて頂点数を抑える） ───────────────────
    let grid_col = [0.0f32, 0.3, 0.5, 3.0];
    for i in (-4i32..=4).step_by(2) {
        let fi = i as f32 * 2.5;
        push_box(&mut verts, &mut idxs, fi,  0.0, 0.0, 0.03, 0.02, 10.0, grid_col);
        push_box(&mut verts, &mut idxs, 0.0, 0.0, fi,  10.0, 0.02, 0.03, grid_col);
    }

    // ── アリーナ外壁 ────────────────────────────────────────────────────────
    let wall_col = [0.0f32, 0.5, 0.8, 3.0];
    push_box(&mut verts, &mut idxs,  0.0, 0.5, -10.1, 10.1, 0.6, 0.08, wall_col);
    push_box(&mut verts, &mut idxs,  0.0, 0.5,  10.1, 10.1, 0.6, 0.08, wall_col);
    push_box(&mut verts, &mut idxs, -10.1, 0.5,  0.0, 0.08, 0.6, 10.1, wall_col);
    push_box(&mut verts, &mut idxs,  10.1, 0.5,  0.0, 0.08, 0.6, 10.1, wall_col);

    // ── 自機 ────────────────────────────────────────────────────────────────
    if g.player_hp > 0 {
        let (px, py, pz) = (g.player_x, 0.5f32, g.player_z);
        let ang = g.player_angle;
        let (s, c) = (ang.sin(), ang.cos());
        let ship_col = [0.2f32, 0.8, 1.0, 3.0];
        let tip  = [px + s*0.5,        py,       pz + c*0.5];
        let lr   = [px - c*0.25,       py,       pz + s*0.25];
        let rr   = [px + c*0.25,       py,       pz - s*0.25];
        let rear = [px - s*0.4,        py,       pz - c*0.4];
        let wl   = [px - s*0.15 - c*0.5, py,     pz - c*0.15 + s*0.5];
        let wr   = [px - s*0.15 + c*0.5, py,     pz - c*0.15 - s*0.5];
        let top  = [px, py + 0.3, pz];
        let base = verts.len() as u32;
        for &pos in &[tip, lr, rr, rear, wl, wr, top] {
            let col = if pos == top { [0.5f32, 0.9, 1.0, 3.0] } else { ship_col };
            verts.push(Vertex { pos, _p: 0.0, col });
        }
        // base=tip, +1=lr, +2=rr, +3=rear, +4=wl, +5=wr, +6=top
        idxs.extend_from_slice(&[
            base,   base+1, base+6,
            base,   base+6, base+2,
            base,   base+1, base+3,
            base,   base+2, base+3,
            base+1, base+4, base+3,
            base+2, base+3, base+5,
        ]);
    }

    // ── 敵 ──────────────────────────────────────────────────────────────────
    for enemy in &g.enemies {
        if !enemy.active { continue; }
        let col = match enemy.kind {
            super::enemy::EnemyKind::Basic   => [1.0f32, 0.3, 0.1, 3.0],
            super::enemy::EnemyKind::Shooter => [1.0f32, 0.1, 0.8, 3.0],
        };
        let sz = match enemy.kind {
            super::enemy::EnemyKind::Basic   => 0.35,
            super::enemy::EnemyKind::Shooter => 0.45,
        };
        push_box(&mut verts, &mut idxs, enemy.x, enemy.y, enemy.z, sz, sz, sz, col);
        let hp_ratio = enemy.hp as f32 / enemy.max_hp as f32;
        let bar_col = [1.0 - hp_ratio, hp_ratio, 0.0, 3.0];
        push_box(&mut verts, &mut idxs, enemy.x, enemy.y + sz + 0.15, enemy.z,
                 hp_ratio * sz, 0.06, 0.06, bar_col);
    }

    // ── ボス ────────────────────────────────────────────────────────────────
    if g.boss.active {
        let b = &g.boss;
        let phase_col: [f32; 4] = match b.phase {
            super::boss::BossPhase::Phase1 => [1.0, 0.5, 0.0, 3.0],
            super::boss::BossPhase::Phase2 => [1.0, 0.1, 0.5, 3.0],
            super::boss::BossPhase::Phase3 => [1.0, 0.0, 0.0, 3.0],
            super::boss::BossPhase::Dead   => [0.3, 0.3, 0.3, 3.0],
        };
        push_box(&mut verts, &mut idxs, b.x, b.y,       b.z, 1.2, 0.5, 1.2, phase_col);
        push_box(&mut verts, &mut idxs, b.x, b.y + 0.5, b.z, 0.4, 0.4, 0.4, [1.0, 1.0, 0.5, 3.0]);
        let hp_ratio = (b.hp as f32 / b.max_hp as f32).max(0.0);
        push_box(&mut verts, &mut idxs, b.x, b.y + 1.2, b.z,
                 hp_ratio * 2.0, 0.1, 0.1, [1.0 - hp_ratio, hp_ratio, 0.0, 3.0]);
    }

    // ── 弾 ──────────────────────────────────────────────────────────────────
    for bullet in &g.bullets.pool {
        if !bullet.active { continue; }
        let r = if bullet.is_player { 0.12 } else { 0.10 };
        let col = [bullet.col[0], bullet.col[1], bullet.col[2], 3.0f32];
        push_box(&mut verts, &mut idxs, bullet.x, bullet.y, bullet.z, r, r, r, col);
    }

    // ── パーティクル ────────────────────────────────────────────────────────
    for p in &g.particles {
        if p.life <= 0.0 { continue; }
        let alpha = (p.life / p.max_life).min(1.0);
        let col = [p.col[0] * alpha, p.col[1] * alpha, p.col[2] * alpha, 3.0f32];
        push_box(&mut verts, &mut idxs, p.x, p.y, p.z, 0.08, 0.08, 0.08, col);
    }

    (verts, idxs)
}

pub fn build_blaster_uni(g: &BlasterGame, aspect: f32) -> Uni {
    let [eye, ctr] = camera_view(g.camera, [g.player_x, 0.5, g.player_z], g.player_angle);
    let up = match g.camera {
        CameraMode::Top => [0.0f32, 1.0, 0.01],
        _ => [0.0, 1.0, 0.0],
    };
    let view = look_at(eye, ctr, up);
    let proj = perspective(std::f32::consts::FRAC_PI_2 * 0.8, aspect, 0.05, 80.0);
    let vp   = mat_mul(proj, view);

    let lights = [
        Light { pos: [g.player_x, 1.5, g.player_z, 0.0],  col: [0.3, 0.8, 1.0, 3.0] },
        Light { pos: [g.boss.x,   2.0, g.boss.z,   1.0],  col: [1.0, 0.3, 0.2, 4.0] },
        Light { pos: [0.0, 3.0, 0.0, 2.0],                col: [0.2, 0.2, 1.0, 1.5] },
        Light { pos: [0.0, 3.0, 0.0, 3.0],                col: [0.1, 0.1, 0.3, 1.0] },
    ];

    Uni {
        vp,
        time: g.time as f32,
        warp: 0.0,
        pad: [0.0; 2],
        lights,
        fog_col: [0.0, 0.0, 0.02, 1.0],
    }
}
