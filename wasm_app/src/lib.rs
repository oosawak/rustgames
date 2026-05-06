use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

const W: usize = 9;
const H: usize = 9;

fn xor64(s: &mut u64) -> usize {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s as usize
}

// Wall bits: N=1 E=2 S=4 W=8
struct Maze {
    cells: [u8; W * H],
}

impl Maze {
    fn generate() -> Self {
        let mut seed = (js_sys::Math::random() * 4_294_967_296.0) as u64;
        seed = seed.wrapping_add(0xdead_cafe_1234);

        let mut cells = [0b1111u8; W * H];
        let mut vis = [false; W * H];
        let mut stk: Vec<(usize, usize)> = vec![(0, 0)];
        vis[0] = true;

        while let Some(&(cx, cy)) = stk.last() {
            let mut nb = [(0usize, 0usize, 0u8, 0u8); 4];
            let mut n = 0;
            if cy > 0 && !vis[(cy - 1) * W + cx] { nb[n] = (cx, cy - 1, 1, 4); n += 1; }
            if cx + 1 < W && !vis[cy * W + cx + 1] { nb[n] = (cx + 1, cy, 2, 8); n += 1; }
            if cy + 1 < H && !vis[(cy + 1) * W + cx] { nb[n] = (cx, cy + 1, 4, 1); n += 1; }
            if cx > 0 && !vis[cy * W + cx - 1] { nb[n] = (cx - 1, cy, 8, 2); n += 1; }

            if n == 0 {
                stk.pop();
            } else {
                let i = xor64(&mut seed) % n;
                let (nx, ny, rc, rn) = nb[i];
                cells[cy * W + cx] &= !rc;
                cells[ny * W + nx] &= !rn;
                vis[ny * W + nx] = true;
                stk.push((nx, ny));
            }
        }
        Maze { cells }
    }

    fn wn(&self, x: usize, y: usize) -> bool { self.cells[y * W + x] & 1 != 0 }
    fn we(&self, x: usize, y: usize) -> bool { self.cells[y * W + x] & 2 != 0 }
    fn ws(&self, x: usize, y: usize) -> bool { self.cells[y * W + x] & 4 != 0 }
    fn ww(&self, x: usize, y: usize) -> bool { self.cells[y * W + x] & 8 != 0 }

    fn can_go(&self, x: usize, y: usize, dx: i32, dy: i32) -> bool {
        match (dx, dy) {
            (0, -1) => !self.wn(x, y),
            (1, 0) => !self.we(x, y),
            (0, 1) => !self.ws(x, y),
            (-1, 0) => !self.ww(x, y),
            _ => false,
        }
    }
}

fn draw_walls(maze: &Maze, ctx: &CanvasRenderingContext2d, ox: f64, oy: f64, cell: f64) {
    for y in 0..H {
        for x in 0..W {
            if maze.wn(x, y) {
                ctx.begin_path();
                ctx.move_to(ox + x as f64 * cell, oy + y as f64 * cell);
                ctx.line_to(ox + (x + 1) as f64 * cell, oy + y as f64 * cell);
                ctx.stroke();
            }
            if maze.ww(x, y) {
                ctx.begin_path();
                ctx.move_to(ox + x as f64 * cell, oy + y as f64 * cell);
                ctx.line_to(ox + x as f64 * cell, oy + (y + 1) as f64 * cell);
                ctx.stroke();
            }
            if x == W - 1 && maze.we(x, y) {
                ctx.begin_path();
                ctx.move_to(ox + W as f64 * cell, oy + y as f64 * cell);
                ctx.line_to(ox + W as f64 * cell, oy + (y + 1) as f64 * cell);
                ctx.stroke();
            }
            if y == H - 1 && maze.ws(x, y) {
                ctx.begin_path();
                ctx.move_to(ox + x as f64 * cell, oy + H as f64 * cell);
                ctx.line_to(ox + (x + 1) as f64 * cell, oy + H as f64 * cell);
                ctx.stroke();
            }
        }
    }
}

#[wasm_bindgen]
pub struct NeonMaze {
    maze: Maze,
    px: usize,
    py: usize,
    moves: u32,
    won: bool,
    elapsed: f64,
    last_ts: f64,
}

#[wasm_bindgen]
impl NeonMaze {
    #[wasm_bindgen(constructor)]
    pub fn new() -> NeonMaze {
        NeonMaze {
            maze: Maze::generate(),
            px: 0,
            py: 0,
            moves: 0,
            won: false,
            elapsed: 0.0,
            last_ts: 0.0,
        }
    }

    pub fn tick(&mut self, ts: f64) {
        if self.last_ts > 0.0 && !self.won {
            self.elapsed += (ts - self.last_ts) / 1000.0;
        }
        self.last_ts = ts;
    }

    pub fn move_player(&mut self, dx: i32, dy: i32) {
        if self.won { return; }
        if self.maze.can_go(self.px, self.py, dx, dy) {
            self.px = (self.px as i32 + dx) as usize;
            self.py = (self.py as i32 + dy) as usize;
            self.moves += 1;
            if self.px == W - 1 && self.py == H - 1 {
                self.won = true;
            }
        }
    }

    pub fn reset(&mut self) {
        *self = NeonMaze::new();
    }

    pub fn is_won(&self) -> bool { self.won }
    pub fn get_moves(&self) -> u32 { self.moves }
    pub fn get_elapsed(&self) -> f64 { self.elapsed }

    pub fn render(&self, canvas_id: &str) -> Result<(), JsValue> {
        let doc = web_sys::window().unwrap().document().unwrap();
        let canvas: HtmlCanvasElement = doc
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("canvas not found"))?
            .dyn_into()?;
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("no 2d ctx"))?
            .dyn_into()?;

        let cw = canvas.width() as f64;
        let ch = canvas.height() as f64;

        // Background
        ctx.set_fill_style(&JsValue::from_str("#020212"));
        ctx.fill_rect(0.0, 0.0, cw, ch);

        let cell = (cw / W as f64).min(ch / H as f64) * 0.96;
        let ox = (cw - cell * W as f64) / 2.0;
        let oy = (ch - cell * H as f64) / 2.0;

        ctx.set_line_cap("round");

        // Glow layer
        ctx.set_shadow_color("rgba(0,180,255,1.0)");
        ctx.set_shadow_blur(18.0);
        ctx.set_stroke_style(&JsValue::from_str("rgba(0,100,220,0.5)"));
        ctx.set_line_width(cell * 0.18);
        draw_walls(&self.maze, &ctx, ox, oy, cell);

        // Core layer
        ctx.set_shadow_color("cyan");
        ctx.set_shadow_blur(6.0);
        ctx.set_stroke_style(&JsValue::from_str("#00eeff"));
        ctx.set_line_width(cell * 0.05);
        draw_walls(&self.maze, &ctx, ox, oy, cell);

        // Goal – pulsing magenta square at bottom-right
        let pulse = (self.elapsed * 3.0).sin() * 0.5 + 0.5;
        let gcx = ox + (W as f64 - 0.5) * cell;
        let gcy = oy + (H as f64 - 0.5) * cell;
        let gs = cell * 0.36;
        ctx.set_shadow_color("magenta");
        ctx.set_shadow_blur(30.0 * pulse);
        let g_col = format!("rgb(255,{},255)", (50.0 + pulse * 150.0) as u32);
        ctx.set_fill_style(&JsValue::from_str(&g_col));
        ctx.fill_rect(gcx - gs, gcy - gs, gs * 2.0, gs * 2.0);

        // Player – yellow square
        let pcx = ox + (self.px as f64 + 0.5) * cell;
        let pcy = oy + (self.py as f64 + 0.5) * cell;
        let ps = cell * 0.3;
        if self.won {
            let flash = (self.elapsed * 8.0).sin() * 0.5 + 0.5;
            ctx.set_shadow_color("yellow");
            ctx.set_shadow_blur(50.0);
            let win_col = format!("rgba(255,255,0,{})", 0.4 + flash * 0.5);
            ctx.set_fill_style(&JsValue::from_str(&win_col));
            ctx.fill_rect(pcx - ps * 2.2, pcy - ps * 2.2, ps * 4.4, ps * 4.4);
        }
        ctx.set_shadow_color("yellow");
        ctx.set_shadow_blur(20.0);
        ctx.set_fill_style(&JsValue::from_str("#ffee22"));
        ctx.fill_rect(pcx - ps, pcy - ps, ps * 2.0, ps * 2.0);
        ctx.set_shadow_blur(0.0);

        // Win overlay
        if self.won {
            ctx.set_fill_style(&JsValue::from_str("rgba(0,0,0,0.55)"));
            ctx.fill_rect(0.0, 0.0, cw, ch);

            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");

            ctx.set_shadow_color("yellow");
            ctx.set_shadow_blur(30.0);
            ctx.set_fill_style(&JsValue::from_str("#ffee00"));
            ctx.set_font(&format!("bold {}px monospace", (cw * 0.13) as u32));
            ctx.fill_text("GOAL!", cw / 2.0, ch / 2.0 - cw * 0.08)?;

            ctx.set_shadow_blur(10.0);
            ctx.set_fill_style(&JsValue::from_str("white"));
            ctx.set_font(&format!("{}px monospace", (cw * 0.065) as u32));
            ctx.fill_text(
                &format!("{} moves  {:.1}s", self.moves, self.elapsed),
                cw / 2.0,
                ch / 2.0 + cw * 0.06,
            )?;

            ctx.set_shadow_blur(0.0);
        }

        Ok(())
    }
}
