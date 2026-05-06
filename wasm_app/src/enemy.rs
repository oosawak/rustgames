// enemy.rs — 追跡敵AIモジュール
//
// 敵はBFS（幅優先探索）で迷路内の最短経路を求め、
// 一定時間ごとにプレイヤーに向かって1マス進みます。

use crate::maze::Maze;
use crate::constants::*;

pub struct Enemy {
    pub ex: usize,
    pub ez: usize,
    pub vis_x: f32,
    pub vis_z: f32,
    pub move_timer: f32,
    pub move_interval: f32,
    pub active: bool,
    pub caught: bool,
}

impl Enemy {
    pub fn new(level: u32) -> Self {
        let interval = (2.2 - (level as f32 - 1.0) * 0.12).max(0.6);
        Self {
            ex: MAZE_W - 1,
            ez: 0,
            vis_x: (MAZE_W - 1) as f32 + 0.5,
            vis_z: 0.5,
            move_timer: 3.0,
            move_interval: interval,
            active: true,
            caught: false,
        }
    }

    pub fn update(&mut self, dt: f32, maze: &Maze, px: usize, pz: usize) {
        if !self.active || self.caught { return; }

        const ENEMY_SPEED: f32 = 7.0;
        let tx = self.ex as f32 + 0.5;
        let tz = self.ez as f32 + 0.5;
        self.vis_x += (tx - self.vis_x) * (ENEMY_SPEED * dt).min(1.0);
        self.vis_z += (tz - self.vis_z) * (ENEMY_SPEED * dt).min(1.0);

        self.move_timer -= dt;
        if self.move_timer > 0.0 { return; }
        self.move_timer = self.move_interval;

        if let Some((nx, nz)) = bfs_next(maze, self.ex, self.ez, px, pz) {
            self.ex = nx;
            self.ez = nz;
        }

        if self.ex == px && self.ez == pz {
            self.caught = true;
        }
    }

    pub fn distance_to(&self, px: usize, pz: usize) -> usize {
        let dx = (self.ex as i32 - px as i32).unsigned_abs() as usize;
        let dz = (self.ez as i32 - pz as i32).unsigned_abs() as usize;
        dx + dz
    }
}

/// BFS最短経路探索：(sx,sz)から(tx,tz)への最初の1手を返す
fn bfs_next(maze: &Maze, sx: usize, sz: usize, tx: usize, tz: usize) -> Option<(usize, usize)> {
    if sx == tx && sz == tz { return None; }

    let size = MAZE_W * MAZE_H;
    let mut visited = vec![false; size];
    let mut parent: Vec<usize> = (0..size).collect();
    let mut queue = std::collections::VecDeque::new();

    let start = sz * MAZE_W + sx;
    let goal  = tz * MAZE_W + tx;
    visited[start] = true;
    queue.push_back(start);

    let dirs: &[(u8, i32, i32)] = &[(N, 0, -1), (S, 0, 1), (E, 1, 0), (W_DIR, -1, 0)];

    'bfs: while let Some(cur) = queue.pop_front() {
        let cx = cur % MAZE_W;
        let cz = cur / MAZE_W;
        for &(dir, ddx, ddz) in dirs {
            let nx = cx as i32 + ddx;
            let nz = cz as i32 + ddz;
            if nx < 0 || nz < 0 || nx >= MAZE_W as i32 || nz >= MAZE_H as i32 { continue; }
            let (nx, nz) = (nx as usize, nz as usize);
            if maze.wall(cx, cz, dir) { continue; }
            let ni = nz * MAZE_W + nx;
            if visited[ni] { continue; }
            visited[ni] = true;
            parent[ni] = cur;
            if ni == goal { break 'bfs; }
            queue.push_back(ni);
        }
    }

    if !visited[goal] { return None; }

    // ゴールから逆算して最初の1手（startの直接の子）を求める
    let mut cur = goal;
    while parent[cur] != start {
        cur = parent[cur];
    }
    Some((cur % MAZE_W, cur / MAZE_W))
}
