// 迷路モジュール: 迷路データ構造と再帰的バックトラック生成アルゴリズムを定義する

use crate::constants::*;
use crate::math::lcg;

pub struct Maze {
    pub cells: [u8; MAZE_W * MAZE_H],
}

impl Maze {
    pub fn new(seed: u64) -> Self {
        let mut rng = seed | 1;
        let mut cells = [N|E|S|W_DIR; MAZE_W * MAZE_H];
        let mut vis   = [false; MAZE_W * MAZE_H];
        let mut stk: Vec<(usize,usize)> = vec![(0,0)];
        vis[0] = true;
        while let Some(&(cx,cz)) = stk.last() {
            let mut nb: Vec<(usize,usize,u8,u8)> = Vec::new();
            if cz>0        &&!vis[(cz-1)*MAZE_W+cx]{nb.push((cx,cz-1,N,S));}
            if cz+1<MAZE_H &&!vis[(cz+1)*MAZE_W+cx]{nb.push((cx,cz+1,S,N));}
            if cx>0        &&!vis[cz*MAZE_W+cx-1]  {nb.push((cx-1,cz,W_DIR,E));}
            if cx+1<MAZE_W &&!vis[cz*MAZE_W+cx+1]  {nb.push((cx+1,cz,E,W_DIR));}
            if nb.is_empty(){stk.pop();}
            else{
                let(nx,nz,d,db)=nb[lcg(&mut rng)%nb.len()];
                cells[cz*MAZE_W+cx]&=!d; cells[nz*MAZE_W+nx]&=!db;
                vis[nz*MAZE_W+nx]=true; stk.push((nx,nz));
            }
        }
        // ── Braid: open ~60% of dead-ends → side paths ──
        for cz in 0..MAZE_H { for cx in 0..MAZE_W {
            if 4-cells[cz*MAZE_W+cx].count_ones() as usize != 1 { continue; }
            if lcg(&mut rng) % 10 >= 6 { continue; }
            let w = cells[cz*MAZE_W+cx];
            let mut cands: Vec<(u8,u8,usize,usize)> = Vec::new();
            if cz>0        && w&N!=0 {cands.push((N,S,cx,cz-1));}
            if cz+1<MAZE_H && w&S!=0 {cands.push((S,N,cx,cz+1));}
            if cx+1<MAZE_W && w&E!=0 {cands.push((E,W_DIR,cx+1,cz));}
            if cx>0        && w&W_DIR!=0{cands.push((W_DIR,E,cx-1,cz));}
            if let Some(&(d,db,nx,nz)) = cands.get(lcg(&mut rng)%cands.len().max(1)){
                cells[cz*MAZE_W+cx]&=!d; cells[nz*MAZE_W+nx]&=!db;
            }
        }}
        Maze { cells }
    }

    pub fn wall(&self,cx:usize,cz:usize,dir:u8)->bool{ self.cells[cz*MAZE_W+cx]&dir!=0 }

    pub fn can_move(&self,cx:usize,cz:usize,dir:u8)->Option<(usize,usize)>{
        if self.wall(cx,cz,dir){return None;}
        match dir{
            d if d==N=>if cz>0       {Some((cx,cz-1))}else{None},
            d if d==S=>if cz+1<MAZE_H{Some((cx,cz+1))}else{None},
            d if d==E=>if cx+1<MAZE_W{Some((cx+1,cz))}else{None},
            _        =>if cx>0       {Some((cx-1,cz))}else{None},
        }
    }
}
