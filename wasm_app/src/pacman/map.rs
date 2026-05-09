use web_sys::CanvasRenderingContext2d;

// Tile IDs: 0=empty, 1=wall, 2=dot, 3=power pellet, 4=ghost house
pub const ROWS: usize = 31;
pub const COLS: usize = 28;

const MAP_DATA: [[u8; COLS]; ROWS] = [
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1], // 0
    [1,3,2,2,2,2,2,2,2,2,2,2,2,1,1,2,2,2,2,2,2,2,2,2,2,2,3,1], // 1
    [1,2,1,1,1,2,1,1,1,1,1,2,1,1,1,2,1,1,1,1,1,2,1,1,1,1,2,1], // 2
    [1,2,1,1,1,2,1,1,1,1,1,2,1,1,1,2,1,1,1,1,1,2,1,1,1,1,2,1], // 3
    [1,2,1,1,1,2,1,1,1,1,1,2,1,1,1,2,1,1,1,1,1,2,1,1,1,1,2,1], // 4
    [1,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,1], // 5
    [1,2,1,1,1,2,1,1,2,1,1,1,1,1,1,1,1,1,1,2,1,1,2,1,1,1,2,1], // 6
    [1,2,1,1,1,2,1,1,2,1,1,1,1,1,1,1,1,1,1,2,1,1,2,1,1,1,2,1], // 7
    [1,2,2,2,2,2,1,1,2,2,2,2,2,1,1,2,2,2,2,2,1,1,2,2,2,2,2,1], // 8
    [1,1,1,1,1,2,1,1,1,1,1,0,1,1,1,1,0,1,1,1,1,1,2,1,1,1,1,1], // 9
    [1,1,1,1,1,2,1,1,1,1,1,0,1,1,1,1,0,1,1,1,1,1,2,1,1,1,1,1], // 10
    [1,1,1,1,1,2,1,1,0,0,0,0,0,0,0,0,0,0,0,0,1,1,2,1,1,1,1,1], // 11
    [1,1,1,1,1,2,1,1,0,1,1,4,4,4,4,4,4,1,1,0,1,1,2,1,1,1,1,1], // 12
    [0,0,0,0,0,2,0,0,0,1,4,4,4,4,4,4,4,1,0,0,0,0,2,0,0,0,0,0], // 13 (tunnel)
    [0,0,0,0,0,2,0,0,0,1,4,4,4,4,4,4,4,1,0,0,0,0,2,0,0,0,0,0], // 14 (tunnel)
    [1,1,1,1,1,2,1,1,0,1,1,1,1,1,1,1,1,1,0,1,1,2,1,1,1,1,1,1], // 15
    [1,1,1,1,1,2,1,1,0,0,0,0,0,0,0,0,0,0,0,0,1,1,2,1,1,1,1,1], // 16
    [1,1,1,1,1,2,1,1,0,1,1,1,1,1,1,1,1,1,1,0,1,1,2,1,1,1,1,1], // 17
    [1,1,1,1,1,2,1,1,0,1,1,1,1,1,1,1,1,1,1,0,1,1,2,1,1,1,1,1], // 18
    [1,2,2,2,2,2,2,2,2,2,2,2,2,1,1,2,2,2,2,2,2,2,2,2,2,2,2,1], // 19
    [1,2,1,1,1,2,1,1,1,1,1,2,1,1,1,2,1,1,1,1,1,2,1,1,1,1,2,1], // 20
    [1,2,1,1,1,2,1,1,1,1,1,2,1,1,1,2,1,1,1,1,1,2,1,1,1,1,2,1], // 21
    [1,3,2,2,1,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,1,2,2,1], // 22
    [1,1,1,2,1,2,1,1,2,1,1,1,1,1,1,1,1,1,1,2,1,1,2,1,1,1,1,1], // 23
    [1,1,1,2,1,2,1,1,2,1,1,1,1,1,1,1,1,1,1,2,1,1,2,1,1,1,1,1], // 24
    [1,2,2,2,2,2,2,2,2,2,2,2,2,1,1,2,2,2,2,2,2,2,2,2,2,2,2,1], // 25
    [1,2,1,1,1,1,1,2,1,1,1,1,1,1,1,1,1,1,1,2,1,1,1,1,1,1,2,1], // 26
    [1,2,1,1,1,1,1,2,1,1,1,1,1,1,1,1,1,1,1,2,1,1,1,1,1,1,2,1], // 27
    [1,2,2,2,2,2,2,2,2,2,2,2,2,1,1,2,2,2,2,2,2,2,2,2,2,2,2,1], // 28
    [1,2,1,1,1,1,1,1,1,1,1,2,1,1,1,2,1,1,1,1,1,1,1,1,1,1,2,1], // 29
    [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1], // 30
];

pub struct Map {
    pub tiles: [[u8; COLS]; ROWS],
    pub total_dots: u32,
}

impl Map {
    pub fn new() -> Self {
        let tiles = MAP_DATA;
        let total_dots = tiles.iter().flatten()
            .filter(|&&t| t == 2 || t == 3)
            .count() as u32;
        Map { tiles, total_dots }
    }

    pub fn is_wall(&self, row: i32, col: i32) -> bool {
        if row < 0 || row >= ROWS as i32 || col < 0 || col >= COLS as i32 {
            return true;
        }
        self.tiles[row as usize][col as usize] == 1
    }

    pub fn is_passable(&self, row: i32, col: i32) -> bool {
        if row < 0 || row >= ROWS as i32 { return false; }
        let col = ((col % COLS as i32) + COLS as i32) as usize % COLS;
        let t = self.tiles[row as usize][col];
        t != 1
    }

    pub fn is_dot(&self, row: i32, col: i32) -> bool {
        if row < 0 || row >= ROWS as i32 || col < 0 || col >= COLS as i32 { return false; }
        let t = self.tiles[row as usize][col as usize];
        t == 2 || t == 3
    }

    pub fn is_power(&self, row: i32, col: i32) -> bool {
        if row < 0 || row >= ROWS as i32 || col < 0 || col >= COLS as i32 { return false; }
        self.tiles[row as usize][col as usize] == 3
    }

    /// Returns score: 10 for dot, 50 for power pellet, 0 for nothing
    pub fn eat_dot(&mut self, row: i32, col: i32) -> u32 {
        if row < 0 || row >= ROWS as i32 || col < 0 || col >= COLS as i32 { return 0; }
        match self.tiles[row as usize][col as usize] {
            2 => { self.tiles[row as usize][col as usize] = 0; 10 }
            3 => { self.tiles[row as usize][col as usize] = 0; 50 }
            _ => 0
        }
    }

    pub fn remaining_dots(&self) -> u32 {
        self.tiles.iter().flatten().filter(|&&t| t == 2 || t == 3).count() as u32
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d, cell: f64, offset_y: f64) {
        for row in 0..ROWS {
            for col in 0..COLS {
                let x = col as f64 * cell;
                let y = row as f64 * cell + offset_y;
                match self.tiles[row][col] {
                    1 => {
                        ctx.set_fill_style_str("#000080");
                        ctx.fill_rect(x, y, cell, cell);
                        ctx.set_fill_style_str("#0000cd");
                        ctx.fill_rect(x + 1.0, y + 1.0, cell - 2.0, cell - 2.0);
                    }
                    2 => {
                        ctx.begin_path();
                        ctx.set_fill_style_str("white");
                        let _ = ctx.arc(x + cell / 2.0, y + cell / 2.0, cell * 0.12, 0.0, std::f64::consts::TAU);
                        ctx.fill();
                    }
                    3 => {
                        ctx.begin_path();
                        ctx.set_fill_style_str("white");
                        let _ = ctx.arc(x + cell / 2.0, y + cell / 2.0, cell * 0.3, 0.0, std::f64::consts::TAU);
                        ctx.fill();
                    }
                    _ => {}
                }
            }
        }
    }
}
