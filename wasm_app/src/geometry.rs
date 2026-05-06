// ジオメトリモジュール: 頂点・ライト・ユニフォーム構造体、色定数、シーン構築関数を定義する

use bytemuck::{Pod, Zeroable};
use crate::constants::*;
use crate::math::lcg;
use crate::maze::Maze;
use crate::particle::Particle;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub _p:  f32,        // パディング → col はオフセット16に配置
    pub col: [f32; 4],   // col.a = マテリアル (1=通常, 2=壁アニメ, 3=パーティクル)
}

pub const STRIDE: u64 = 32;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Light {
    pub pos: [f32; 4],  // xyz=ワールド座標, w=フリッカー位相
    pub col: [f32; 4],  // rgb=色, a=強度
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uni {
    pub vp:     [[f32; 4]; 4], // 64バイト (オフセット0)
    pub time:   f32,            // オフセット64
    pub warp:   f32,            // オフセット68
    pub pad:    [f32; 2],       // オフセット72 → 合計80バイト
    pub lights: [Light; 4],     // オフセット80, 128バイト → 合計208バイト
}

pub const WALL_COL:  [f32;4] = [0.05, 0.80, 1.0, 2.0]; // a=2 → 壁マテリアル
pub const FLOOR_COL: [f32;4] = [0.0,  0.04, 0.12, 1.0];
pub const CEIL_COL:  [f32;4] = [0.0,  0.02, 0.06, 1.0];

// 4つのライトカラー: rgb + 強度
pub const LIGHT_COLS: [[f32;4];4] = [
    [1.00, 0.45, 0.05, 3.2],  // 暖かいオレンジ
    [0.55, 0.10, 1.00, 3.2],  // パープル
    [0.05, 0.80, 1.00, 3.2],  // シアンブルー
    [1.00, 0.25, 0.60, 3.2],  // ピンク
];

/// 4つのライト位置を迷路の交差点セルから探す（間隔を空けて配置）
pub fn find_lights(maze: &Maze, rng: &mut u64) -> [[f32;4];4] {
    let mut scored: Vec<(usize,usize,usize)> = (0..MAZE_H).flat_map(|cz|
        (0..MAZE_W).map(move |cx| {
            let open = 4 - maze.cells[cz*MAZE_W+cx].count_ones() as usize;
            (open, cx, cz)
        })
    ).collect();
    scored.sort_by(|a,b| b.0.cmp(&a.0));

    let mut picks: Vec<(usize,usize)> = Vec::new();
    for &(_score, cx, cz) in &scored {
        let too_close = picks.iter().any(|&(px,pz)| {
            let dx = cx as i32 - px as i32;
            let dz = cz as i32 - pz as i32;
            dx*dx + dz*dz < 9
        });
        if !too_close { picks.push((cx,cz)); }
        if picks.len() == 4 { break; }
    }
    while picks.len() < 4 {
        let &(_,cx,cz) = &scored[lcg(rng) % scored.len().min(16)];
        picks.push((cx,cz));
    }

    let mut result = [[0f32;4];4];
    for (i, &(cx,cz)) in picks.iter().enumerate() {
        result[i] = [cx as f32+0.5, WALL_H*0.88, cz as f32+0.5, i as f32 * 1.57];
    }
    result
}

pub fn quad(vs:&mut Vec<Vertex>,ix:&mut Vec<u32>,
        v0:[f32;3],v1:[f32;3],v2:[f32;3],v3:[f32;3],col:[f32;4]){
    let b=vs.len() as u32;
    for p in[v0,v1,v2,v3]{vs.push(Vertex{pos:p,_p:0.0,col});}
    ix.extend_from_slice(&[b,b+1,b+2,b,b+2,b+3]);
}

pub fn pillar(vs:&mut Vec<Vertex>,ix:&mut Vec<u32>,
          cx:f32,cz:f32,r:f32,h:f32,col:[f32;4]){
    for &(ax,az,bx,bz) in &[
        (cx-r,cz-r,cx+r,cz-r),(cx+r,cz+r,cx-r,cz+r),
        (cx+r,cz-r,cx+r,cz+r),(cx-r,cz+r,cx-r,cz-r),
    ]{ quad(vs,ix,[ax,0.0,az],[bx,0.0,bz],[bx,h,bz],[ax,h,az],col); }
}

/// パーティクルを十字形（2枚の垂直クワッド）でレンダリングする
pub fn particle_cross(vs:&mut Vec<Vertex>,ix:&mut Vec<u32>,
                  p:&Particle){
    let size = 0.10 * p.life;
    let h = size * 0.5;
    let [x,y,z] = p.pos;
    let brightness = p.life * 3.0;
    let col = [0.2 * brightness, brightness, brightness, 3.0]; // a=3 → パーティクル

    // X方向のスパーク
    quad(vs,ix,[x-h,y-h,z],[x+h,y-h,z],[x+h,y+h,z],[x-h,y+h,z],col);
    // Z方向のスパーク
    quad(vs,ix,[x,y-h,z-h],[x,y-h,z+h],[x,y+h,z+h],[x,y+h,z-h],col);
}

/// 迷路シーン全体のジオメトリを構築して頂点・インデックスバッファを返す
pub fn build_scene(maze:&Maze, time:f32, particles:&[Particle],
               light_pos:&[[f32;4];4]) -> (Vec<Vertex>,Vec<u32>){
    let mut vs:Vec<Vertex>=Vec::with_capacity(2048);
    let mut ix:Vec<u32>   =Vec::with_capacity(4096);
    let mw=MAZE_W as f32; let mh=MAZE_H as f32;

    // 床・天井
    quad(&mut vs,&mut ix,[0.0,0.0,0.0],[mw,0.0,0.0],[mw,0.0,mh],[0.0,0.0,mh],FLOOR_COL);
    quad(&mut vs,&mut ix,[0.0,WALL_H,mh],[mw,WALL_H,mh],[mw,WALL_H,0.0],[0.0,WALL_H,0.0],CEIL_COL);

    // 外壁
    quad(&mut vs,&mut ix,[mw,0.0,0.0],[mw,WALL_H,0.0],[0.0,WALL_H,0.0],[0.0,0.0,0.0],WALL_COL);
    quad(&mut vs,&mut ix,[0.0,0.0,mh],[0.0,WALL_H,mh],[mw,WALL_H,mh],[mw,0.0,mh],WALL_COL);
    quad(&mut vs,&mut ix,[0.0,0.0,0.0],[0.0,WALL_H,0.0],[0.0,WALL_H,mh],[0.0,0.0,mh],WALL_COL);
    quad(&mut vs,&mut ix,[mw,0.0,mh],[mw,WALL_H,mh],[mw,WALL_H,0.0],[mw,0.0,0.0],WALL_COL);

    // 内壁（両面）
    for cz in 0..MAZE_H { for cx in 0..MAZE_W {
        let(x,z)=(cx as f32,cz as f32);
        if cz+1<MAZE_H && maze.wall(cx,cz,S){
            quad(&mut vs,&mut ix,[x,0.0,z+1.0],[x+1.0,0.0,z+1.0],[x+1.0,WALL_H,z+1.0],[x,WALL_H,z+1.0],WALL_COL);
            quad(&mut vs,&mut ix,[x,0.0,z+1.0],[x,WALL_H,z+1.0],[x+1.0,WALL_H,z+1.0],[x+1.0,0.0,z+1.0],WALL_COL);
        }
        if cx+1<MAZE_W && maze.wall(cx,cz,E){
            quad(&mut vs,&mut ix,[x+1.0,0.0,z],[x+1.0,0.0,z+1.0],[x+1.0,WALL_H,z+1.0],[x+1.0,WALL_H,z],WALL_COL);
            quad(&mut vs,&mut ix,[x+1.0,0.0,z],[x+1.0,WALL_H,z],[x+1.0,WALL_H,z+1.0],[x+1.0,0.0,z+1.0],WALL_COL);
        }
    }}

    // ── 各ライト位置に吊り下げランタンを配置 ──
    for (i, lp) in light_pos.iter().enumerate() {
        let (lx, lz) = (lp[0], lp[2]);
        let lc = LIGHT_COLS[i];
        let ec = [lc[0]*3.0, lc[1]*3.0, lc[2]*3.0, 1.0];
        let pulse = (time * 2.2 + lp[3]).sin() * 0.1 + 0.9;
        let pc = [lc[0]*2.5*pulse, lc[1]*2.5*pulse, lc[2]*2.5*pulse, 1.0];
        let r = 0.10f32;
        quad(&mut vs,&mut ix,
            [lx-r, WALL_H-0.02, lz-r],[lx+r, WALL_H-0.02, lz-r],
            [lx+r, WALL_H-0.02, lz+r],[lx-r, WALL_H-0.02, lz+r], ec);
        pillar(&mut vs,&mut ix, lx, lz, 0.05, WALL_H*0.72, pc);
        let gr = 0.28f32;
        quad(&mut vs,&mut ix,
            [lx-gr, 0.01, lz-gr],[lx+gr, 0.01, lz-gr],
            [lx+gr, 0.01, lz+gr],[lx-gr, 0.01, lz+gr],
            [lc[0]*pulse*0.6, lc[1]*pulse*0.6, lc[2]*pulse*0.6, 1.0]);
    }

    // スタート床グロー（緑の十字パターン）
    let st = (time * 1.8).sin() * 0.3 + 0.7;
    let gr = 0.42f32;
    quad(&mut vs,&mut ix,
        [0.5-gr, 0.012, 0.5-gr],[0.5+gr, 0.012, 0.5-gr],
        [0.5+gr, 0.012, 0.5+gr],[0.5-gr, 0.012, 0.5+gr],
        [0.0, st*0.9, st*0.35, 1.0]);
    let cr = 0.16f32;
    quad(&mut vs,&mut ix,
        [0.5-cr, 0.015, 0.5-cr],[0.5+cr, 0.015, 0.5-cr],
        [0.5+cr, 0.015, 0.5+cr],[0.5-cr, 0.015, 0.5+cr],
        [0.1, st*2.2, st*0.8, 1.0]);

    // ゴールピラー（マゼンタ発光）
    let p = (time*2.5).sin()*0.5+0.5;
    let(gx,gz)=((MAZE_W-1) as f32+0.5,(MAZE_H-1) as f32+0.5);
    pillar(&mut vs,&mut ix, gx,gz, 0.15,WALL_H*1.1, [3.0,p*0.4,3.0,1.0]);
    let r2=0.45f32;
    quad(&mut vs,&mut ix,
        [gx-r2,0.02,gz-r2],[gx+r2,0.02,gz-r2],
        [gx+r2,0.02,gz+r2],[gx-r2,0.02,gz+r2],[2.0,p*0.3,2.0,1.0]);

    // パーティクル描画
    for p in particles.iter().filter(|p| p.life > 0.0) {
        particle_cross(&mut vs, &mut ix, p);
    }

    (vs,ix)
}
