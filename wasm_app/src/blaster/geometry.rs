use crate::geometry::{Vertex, Light, Uni};
use crate::math::{mat_mul, perspective, look_at};
use super::state::BlasterGame;
use super::camera_mode::{CameraMode, camera_view};

// ── Y 軸回転ヘルパー ─────────────────────────────────────────────────────────
#[inline]
fn rot_y(x: f32, z: f32, a: f32) -> (f32, f32) {
    let (sa, ca) = (a.sin(), a.cos());
    (x * ca + z * sa, -x * sa + z * ca)
}

/// タンク中心からのローカルオフセット(ox,oy,oz)と半サイズ(sx,sy,sz)を
/// Y 軸角 angle で回転したボックスを追加する
fn push_part(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>,
             cx: f32, cy: f32, cz: f32,
             ox: f32, oy: f32, oz: f32,
             sx: f32, sy: f32, sz: f32,
             angle: f32, col: [f32; 4]) {
    let (rx, rz) = rot_y(ox, oz, angle);
    let (wcx, wcy, wcz) = (cx + rx, cy + oy, cz + rz);
    let base = verts.len() as u32;
    for &(lx, ly, lz) in &[
        (-sx,-sy,-sz),(sx,-sy,-sz),(sx,sy,-sz),(-sx,sy,-sz),
        (-sx,-sy, sz),(sx,-sy, sz),(sx,sy, sz),(-sx,sy, sz),
    ] {
        let (vx, vz) = rot_y(lx, lz, angle);
        verts.push(Vertex { pos: [wcx + vx, wcy + ly, wcz + vz], _p: 0.0, col });
    }
    for &(a, b, c, d) in &[
        (0u32,1,2,3),(4,7,6,5),(0,4,5,1),(2,6,7,3),(0,3,7,4),(1,5,6,2)
    ] {
        idxs.extend_from_slice(&[base+a,base+b,base+c, base+a,base+c,base+d]);
    }
}

/// 軸平行ボックス（アリーナ・パーティクル用）
fn push_box(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>,
            cx: f32, cy: f32, cz: f32,
            sx: f32, sy: f32, sz: f32, col: [f32; 4]) {
    push_part(verts, idxs, cx, cy, cz, 0.0, 0.0, 0.0, sx, sy, sz, 0.0, col);
}

/// ダイヤモンド型の弾（インデックス数を節約）
fn push_diamond(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>,
                x: f32, y: f32, z: f32, r: f32, col: [f32; 4]) {
    let base = verts.len() as u32;
    for &pos in &[(x,y+r,z),(x,y-r,z),(x+r,y,z),(x-r,y,z),(x,y,z+r),(x,y,z-r)] {
        verts.push(Vertex { pos: [pos.0, pos.1, pos.2], _p: 0.0, col });
    }
    idxs.extend_from_slice(&[
        base,base+2,base+4, base,base+4,base+3, base,base+3,base+5, base,base+5,base+2,
        base+1,base+4,base+2, base+1,base+3,base+4, base+1,base+5,base+3, base+1,base+2,base+5,
    ]);
}

/// 戦車を描画する
/// - body_angle  : 車体（車台・履帯）の向き ← 移動方向
/// - turret_angle: 砲塔・砲身の向き         ← 自動エイム方向（独立回転）
fn draw_tank(verts: &mut Vec<Vertex>, idxs: &mut Vec<u32>,
             cx: f32, cz: f32,
             body_angle: f32, turret_angle: f32,
             track_col: [f32; 4], hull_col: [f32; 4],
             turret_col: [f32; 4], barrel_col: [f32; 4]) {
    // 左右履帯（車体と同方向）
    push_part(verts,idxs, cx,0.0,cz, -0.60,0.12,0.0, 0.13,0.12,0.86, body_angle, track_col);
    push_part(verts,idxs, cx,0.0,cz,  0.60,0.12,0.0, 0.13,0.12,0.86, body_angle, track_col);
    // 車体ハル
    push_part(verts,idxs, cx,0.0,cz,  0.0,0.42,0.0,  0.50,0.18,0.72, body_angle, hull_col);
    // 砲塔（turret_angle で独立回転）
    push_part(verts,idxs, cx,0.0,cz,  0.0,0.76,-0.04, 0.27,0.15,0.24, turret_angle, turret_col);
    // 砲身（砲塔と同じ angle）
    push_part(verts,idxs, cx,0.0,cz,  0.0,0.68,0.60,  0.07,0.07,0.30, turret_angle, barrel_col);
}

pub fn build_blaster_scene(g: &BlasterGame) -> (Vec<Vertex>, Vec<u32>) {
    let mut verts: Vec<Vertex> = Vec::with_capacity(4096);
    let mut idxs:  Vec<u32>   = Vec::with_capacity(8192);

    // ── アリーナ床 ────────────────────────────────────────────────────────────
    push_box(&mut verts,&mut idxs, 0.0,-0.05,0.0, 18.0,0.05,18.0, [0.02,0.02,0.08,1.0]);

    // ── グリッド線（間引き）────────────────────────────────────────────────────
    let grid_col = [0.0f32,0.25,0.45,3.0];
    for i in -9i32..=9 {
        let fi = i as f32 * 2.0;
        push_box(&mut verts,&mut idxs, fi,  0.0,0.0,  0.025,0.015,18.0, grid_col);
        push_box(&mut verts,&mut idxs, 0.0,0.0,fi,   18.0,0.015,0.025, grid_col);
    }

    // アリーナ外壁
    let wall_col = [0.0f32,0.5,0.8,3.0];
    push_box(&mut verts,&mut idxs,  0.0,0.5,-18.1, 18.1,0.5,0.07, wall_col);
    push_box(&mut verts,&mut idxs,  0.0,0.5, 18.1, 18.1,0.5,0.07, wall_col);
    push_box(&mut verts,&mut idxs,-18.1,0.5,  0.0, 0.07,0.5,18.1, wall_col);
    push_box(&mut verts,&mut idxs, 18.1,0.5,  0.0, 0.07,0.5,18.1, wall_col);

    // ── 自機（プレイヤー戦車） ────────────────────────────────────────────────
    if g.player_hp > 0 {
        let flash = (g.invincible > 0.0) && ((g.time * 8.0) as i32 % 2 == 0);
        if !flash {
            draw_tank(&mut verts, &mut idxs,
                g.player_x, g.player_z,
                g.player_body_angle, g.player_turret_angle,
                [0.02,0.20,0.55,3.0],  // 履帯
                [0.08,0.45,0.85,3.0],  // 車体
                [0.15,0.65,1.00,3.0],  // 砲塔
                [0.40,0.90,1.00,3.0],  // 砲身
            );
        }
    }

    // ── 敵戦車 ────────────────────────────────────────────────────────────────
    for enemy in &g.enemies {
        if !enemy.active { continue; }
        let (track_col, hull_col, turret_col, barrel_col) = match enemy.kind {
            super::enemy::EnemyKind::Basic => (
                [0.30f32,0.05,0.02,3.0],
                [0.65,0.15,0.04,3.0],
                [0.85,0.30,0.08,3.0],
                [1.00,0.50,0.15,3.0],
            ),
            super::enemy::EnemyKind::Shooter => (
                [0.25f32,0.02,0.35,3.0],
                [0.55,0.04,0.75,3.0],
                [0.80,0.10,1.00,3.0],
                [1.00,0.35,1.00,3.0],
            ),
        };
        draw_tank(&mut verts, &mut idxs,
            enemy.x, enemy.z,
            enemy.body_angle, enemy.turret_angle,
            track_col, hull_col, turret_col, barrel_col,
        );
        // HP バー（砲塔上空）
        let hp_ratio = (enemy.hp as f32 / enemy.max_hp as f32).clamp(0.0,1.0);
        push_box(&mut verts,&mut idxs,
            enemy.x, 1.5, enemy.z,
            hp_ratio * 0.4, 0.05, 0.05,
            [1.0-hp_ratio, hp_ratio, 0.0, 3.0]);
    }

    // ── ボス戦車（大型・2砲塔） ───────────────────────────────────────────────
    if g.boss.active {
        let b = &g.boss;
        let phase_col: ([f32;4],[f32;4],[f32;4],[f32;4]) = match b.phase {
            super::boss::BossPhase::Phase1 =>
                ([0.35,0.18,0.02,3.0],[0.70,0.35,0.05,3.0],[0.90,0.55,0.10,3.0],[1.0,0.75,0.20,3.0]),
            super::boss::BossPhase::Phase2 =>
                ([0.40,0.02,0.20,3.0],[0.80,0.05,0.45,3.0],[1.00,0.10,0.65,3.0],[1.0,0.30,0.80,3.0]),
            super::boss::BossPhase::Phase3 =>
                ([0.35,0.00,0.00,3.0],[0.75,0.00,0.00,3.0],[1.00,0.05,0.05,3.0],[1.0,0.20,0.20,3.0]),
            super::boss::BossPhase::Dead =>
                ([0.15,0.15,0.15,3.0],[0.25,0.25,0.25,3.0],[0.30,0.30,0.30,3.0],[0.35,0.35,0.35,3.0]),
        };
        // 大型車体（1.6倍スケール）
        push_part(&mut verts,&mut idxs, b.x,0.0,b.z, -0.92,0.19,0.0, 0.21,0.19,1.35, b.body_angle, phase_col.0);
        push_part(&mut verts,&mut idxs, b.x,0.0,b.z,  0.92,0.19,0.0, 0.21,0.19,1.35, b.body_angle, phase_col.0);
        push_part(&mut verts,&mut idxs, b.x,0.0,b.z,  0.0,0.67,0.0,  0.78,0.27,1.15, b.body_angle, phase_col.1);
        // 砲塔（大型・独立回転）
        push_part(&mut verts,&mut idxs, b.x,0.0,b.z,  0.0,1.22,-0.06, 0.42,0.24,0.38, b.turret_angle, phase_col.2);
        // 左右2本の砲身
        push_part(&mut verts,&mut idxs, b.x,0.0,b.z, -0.18,1.08,0.95, 0.09,0.09,0.48, b.turret_angle, phase_col.3);
        push_part(&mut verts,&mut idxs, b.x,0.0,b.z,  0.18,1.08,0.95, 0.09,0.09,0.48, b.turret_angle, phase_col.3);
        // HP バー
        let hp_ratio = (b.hp as f32 / b.max_hp as f32).max(0.0);
        push_box(&mut verts,&mut idxs, b.x,2.2,b.z, hp_ratio*1.8,0.1,0.1,
                 [1.0-hp_ratio,hp_ratio,0.0,3.0]);
    }

    // ── 弾（ダイヤモンド型） ──────────────────────────────────────────────────
    for bullet in &g.bullets.pool {
        if !bullet.active { continue; }
        let r = if bullet.is_player { 0.15 } else { 0.12 };
        let col = [bullet.col[0], bullet.col[1], bullet.col[2], 3.0f32];
        push_diamond(&mut verts, &mut idxs, bullet.x, bullet.y, bullet.z, r, col);
    }

    // ── パーティクル ────────────────────────────────────────────────────────
    for p in &g.particles {
        if p.life <= 0.0 { continue; }
        let a = (p.life / p.max_life).min(1.0);
        let col = [p.col[0]*a, p.col[1]*a, p.col[2]*a, 3.0f32];
        push_diamond(&mut verts, &mut idxs, p.x, p.y, p.z, 0.1, col);
    }

    (verts, idxs)
}

pub fn build_blaster_uni(g: &BlasterGame, aspect: f32) -> Uni {
    let [eye, ctr] = camera_view(g.camera, [g.player_x, 0.5, g.player_z], g.player_body_angle);
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

