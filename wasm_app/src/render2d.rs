// render2d.rs — wgpu 2D バッチ描画層（正射影・色付き三角形バッチ）
//
// rustgames の既存タイトルは wgpu 3D パイプライン（gpu.rs）を使うが、
// CubeBoy / LINE BOY のような 2D ピクセルゲーム向けに、
//   - 仮想解像度（例: 128x128 / 512x512）をキャンバスへレターボックス拡大
//   - 矩形・線・三角形・円を 1 ドローコールにバッチ化
//   - 5x7 ビットマップフォントによる ASCII テキスト
// を提供する最小レイヤー。テクスチャは使わず、全て色付き頂点で描く
// （＝「色矩形＋ピクセルデータで見た目を再現」する方式）。

use bytemuck::{Pod, Zeroable};

// ── 色 ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba(pub [f32; 4]);

impl Rgba {
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Rgba([r, g, b, 1.0])
    }
    /// 0xRRGGBB から作る
    pub const fn hex(v: u32) -> Self {
        Rgba([
            ((v >> 16) & 0xff) as f32 / 255.0,
            ((v >> 8) & 0xff) as f32 / 255.0,
            (v & 0xff) as f32 / 255.0,
            1.0,
        ])
    }
    pub const fn with_alpha(self, a: f32) -> Self {
        Rgba([self.0[0], self.0[1], self.0[2], a])
    }
}

// ── 5x7 ビットマップフォント（行ごとに 5bit、bit4 が左端） ───────────────────

fn glyph(ch: char) -> [u8; 7] {
    match ch {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
        'X' => [0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b01010, 0b10001],
        'Y' => [0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        ':' => [0, 0b00100, 0b00100, 0, 0b00100, 0b00100, 0],
        '/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
        ',' => [0, 0, 0, 0, 0b00100, 0b00100, 0b01000],
        '.' => [0, 0, 0, 0, 0, 0, 0b00100],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0, 0b00100],
        '-' => [0, 0, 0, 0b01110, 0, 0, 0],
        '+' => [0, 0b00100, 0b00100, 0b01110, 0b00100, 0b00100, 0],
        '\'' => [0b00100, 0b00100, 0b01000, 0, 0, 0, 0],
        '%' => [0b11001, 0b11010, 0b00010, 0b00100, 0b01000, 0b01011, 0b10011],
        _ => [0; 7],
    }
}

// ── 頂点 ────────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 2],
    col: [f32; 4],
}

// ── 描画コマンドの蓄積（1 フレーム分） ──────────────────────────────────────

#[derive(Default)]
pub struct Draw2d {
    verts: Vec<Vertex>,
    idx: Vec<u32>,
    /// 全座標に加算されるオフセット（画面シェイク用）。HUD 等は (0,0) に戻して描く。
    pub offset: (f32, f32),
}

impl Draw2d {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.verts.clear();
        self.idx.clear();
    }

    fn push(&mut self, x: f32, y: f32, col: Rgba) -> u32 {
        let i = self.verts.len() as u32;
        self.verts.push(Vertex {
            pos: [x + self.offset.0, y + self.offset.1],
            col: col.0,
        });
        i
    }

    pub fn tri(&mut self, a: (f32, f32), b: (f32, f32), c: (f32, f32), col: Rgba) {
        let i0 = self.push(a.0, a.1, col);
        let i1 = self.push(b.0, b.1, col);
        let i2 = self.push(c.0, c.1, col);
        self.idx.extend_from_slice(&[i0, i1, i2]);
    }

    pub fn quad(&mut self, p: [(f32, f32); 4], col: Rgba) {
        let i0 = self.push(p[0].0, p[0].1, col);
        let i1 = self.push(p[1].0, p[1].1, col);
        let i2 = self.push(p[2].0, p[2].1, col);
        let i3 = self.push(p[3].0, p[3].1, col);
        self.idx.extend_from_slice(&[i0, i1, i2, i0, i2, i3]);
    }

    /// 塗りつぶし矩形
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, col: Rgba) {
        self.quad([(x, y), (x + w, y), (x + w, y + h), (x, y + h)], col);
    }

    /// 1 ドット
    pub fn pixel(&mut self, x: f32, y: f32, col: Rgba) {
        self.rect(x, y, 1.0, 1.0, col);
    }

    /// 線分（幅 w のクワッドとして描く）
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, w: f32, col: Rgba) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.0001 {
            self.rect(x1 - w / 2.0, y1 - w / 2.0, w, w, col);
            return;
        }
        let nx = -dy / len * (w / 2.0);
        let ny = dx / len * (w / 2.0);
        self.quad(
            [
                (x1 + nx, y1 + ny),
                (x2 + nx, y2 + ny),
                (x2 - nx, y2 - ny),
                (x1 - nx, y1 - ny),
            ],
            col,
        );
    }

    /// 矩形の枠（線幅 lw）
    pub fn rect_outline(&mut self, x: f32, y: f32, w: f32, h: f32, lw: f32, col: Rgba) {
        self.line(x, y, x + w, y, lw, col);
        self.line(x + w, y, x + w, y + h, lw, col);
        self.line(x + w, y + h, x, y + h, lw, col);
        self.line(x, y + h, x, y, lw, col);
    }

    /// 回転する矩形の枠（中心 cx,cy、半辺 half、角度 ang）
    pub fn rect_outline_rot(&mut self, cx: f32, cy: f32, half: f32, ang: f32, lw: f32, col: Rgba) {
        let (s, c) = ang.sin_cos();
        let rot = |lx: f32, ly: f32| (cx + lx * c - ly * s, cy + lx * s + ly * c);
        let p = [
            rot(-half, -half),
            rot(half, -half),
            rot(half, half),
            rot(-half, half),
        ];
        for i in 0..4 {
            let a = p[i];
            let b = p[(i + 1) % 4];
            self.line(a.0, a.1, b.0, b.1, lw, col);
        }
    }

    /// 円の枠
    pub fn circle_outline(&mut self, cx: f32, cy: f32, r: f32, lw: f32, col: Rgba) {
        let seg = 28;
        let mut prev = (cx + r, cy);
        for i in 1..=seg {
            let a = (i as f32) / (seg as f32) * std::f32::consts::TAU;
            let p = (cx + r * a.cos(), cy + r * a.sin());
            self.line(prev.0, prev.1, p.0, p.1, lw, col);
            prev = p;
        }
    }

    /// 円弧の枠（角度 a0→a1、y-down 座標系のラジアン）
    pub fn arc_outline(&mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, lw: f32, col: Rgba) {
        let seg = 20;
        let mut prev = (cx + r * a0.cos(), cy + r * a0.sin());
        for i in 1..=seg {
            let a = a0 + (a1 - a0) * (i as f32) / (seg as f32);
            let p = (cx + r * a.cos(), cy + r * a.sin());
            self.line(prev.0, prev.1, p.0, p.1, lw, col);
            prev = p;
        }
    }

    /// 塗りつぶし円（三角形ファン）
    pub fn circle_fill(&mut self, cx: f32, cy: f32, r: f32, col: Rgba) {
        self.ellipse_fill(cx, cy, r, r, col);
    }

    /// 塗りつぶし楕円
    pub fn ellipse_fill(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, col: Rgba) {
        let seg = 28;
        let mut prev = (cx + rx, cy);
        for i in 1..=seg {
            let a = (i as f32) / (seg as f32) * std::f32::consts::TAU;
            let p = (cx + rx * a.cos(), cy + ry * a.sin());
            self.tri((cx, cy), prev, p, col);
            prev = p;
        }
    }

    /// 楕円の枠
    pub fn ellipse_outline(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, lw: f32, col: Rgba) {
        let seg = 28;
        let mut prev = (cx + rx, cy);
        for i in 1..=seg {
            let a = (i as f32) / (seg as f32) * std::f32::consts::TAU;
            let p = (cx + rx * a.cos(), cy + ry * a.sin());
            self.line(prev.0, prev.1, p.0, p.1, lw, col);
            prev = p;
        }
    }

    /// テキスト幅（scale 倍、文字間隔 6*scale）
    pub fn text_width(s: &str, scale: f32) -> f32 {
        if s.is_empty() {
            0.0
        } else {
            s.chars().count() as f32 * 6.0 * scale - scale
        }
    }

    /// ビットマップフォントでテキスト描画（左上原点、scale はドット倍率）
    pub fn text(&mut self, s: &str, x: f32, y: f32, scale: f32, col: Rgba) {
        let mut cx = x;
        for ch in s.chars() {
            let g = glyph(ch.to_ascii_uppercase());
            for row in 0..7 {
                let bits = g[row];
                for col_i in 0..5 {
                    if bits & (0b10000 >> col_i) != 0 {
                        self.rect(
                            cx + col_i as f32 * scale,
                            y + row as f32 * scale,
                            scale,
                            scale,
                            col,
                        );
                    }
                }
            }
            cx += 6.0 * scale;
        }
    }

    /// 中央寄せテキスト
    pub fn text_centered(&mut self, s: &str, cx: f32, y: f32, scale: f32, col: Rgba) {
        let w = Self::text_width(s, scale);
        self.text(s, cx - w / 2.0, y, scale, col);
    }
}

// ── wgpu レンダラー本体 ─────────────────────────────────────────────────────

const SHADER_2D: &str = r#"
struct Uni { view: vec4<f32> }
@group(0) @binding(0) var<uniform> u: Uni;
struct VIn {
    @location(0) pos: vec2<f32>,
    @location(1) col: vec4<f32>,
}
struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) col: vec4<f32>,
}
@vertex
fn vs_main(v: VIn) -> VOut {
    var o: VOut;
    o.clip = vec4<f32>(v.pos.x * u.view.x + u.view.z, v.pos.y * u.view.y + u.view.w, 0.0, 1.0);
    o.col = v.col;
    return o;
}
@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
    return v.col;
}
"#;

pub struct Renderer2d {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    uni_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vert_buf: wgpu::Buffer,
    idx_buf: wgpu::Buffer,
    vert_cap: u64,
    idx_cap: u64,
    width: u32,
    height: u32,
    fmt: wgpu::TextureFormat,
    vw: f32,
    vh: f32,
}

impl Renderer2d {
    pub async fn new(canvas: web_sys::HtmlCanvasElement, vw: f32, vh: f32) -> Result<Self, String> {
        let (w, h) = (canvas.width().max(1), canvas.height().max(1));
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|e| e.to_string())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("no adapter")?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .map_err(|e| e.to_string())?;

        let caps = surface.get_capabilities(&adapter);
        let fmt = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: fmt,
            width: w,
            height: h,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("render2d"),
            source: wgpu::ShaderSource::Wgsl(SHADER_2D.into()),
        });

        let uni_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("render2d-uni"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("render2d-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render2d-bg"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uni_buf.as_entire_binding(),
            }],
        });
        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render2d-pll"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render2d-pipeline"),
            layout: Some(&pll),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: fmt,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vert_cap = 1 << 20; // 1MB
        let idx_cap = 1 << 20;
        let vert_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("render2d-vb"),
            size: vert_cap,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let idx_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("render2d-ib"),
            size: idx_cap,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            surface,
            device,
            queue,
            pipeline,
            uni_buf,
            bind_group,
            vert_buf,
            idx_buf,
            vert_cap,
            idx_cap,
            width: w,
            height: h,
            fmt,
            vw,
            vh,
        })
    }

    fn reconfigure(&self) {
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.fmt,
            width: self.width,
            height: self.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        self.surface.configure(&self.device, &config);
    }

    /// 仮想座標 → クリップ座標の変換行列（vec4: x*a+c, y*b+d）
    fn view_transform(&self) -> [f32; 4] {
        let (w, h) = (self.width as f32, self.height as f32);
        let s = (w / self.vw).min(h / self.vh);
        let ox = (w - self.vw * s) / 2.0;
        let oy = (h - self.vh * s) / 2.0;
        [
            2.0 * s / w,
            -2.0 * s / h,
            2.0 * ox / w - 1.0,
            1.0 - 2.0 * oy / h,
        ]
    }

    pub fn render(&mut self, draw: &Draw2d, clear: Rgba) {
        // バッファ容量が足りなければ作り直す
        let vbytes = (draw.verts.len() * std::mem::size_of::<Vertex>()) as u64;
        let ibytes = (draw.idx.len() * 4) as u64;
        if vbytes > self.vert_cap {
            self.vert_cap = vbytes.next_power_of_two();
            self.vert_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("render2d-vb"),
                size: self.vert_cap,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if ibytes > self.idx_cap {
            self.idx_cap = ibytes.next_power_of_two();
            self.idx_buf = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("render2d-ib"),
                size: self.idx_cap,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        if vbytes > 0 {
            self.queue
                .write_buffer(&self.vert_buf, 0, bytemuck::cast_slice(&draw.verts));
        }
        if ibytes > 0 {
            self.queue
                .write_buffer(&self.idx_buf, 0, bytemuck::cast_slice(&draw.idx));
        }
        self.queue
            .write_buffer(&self.uni_buf, 0, bytemuck::cast_slice(&self.view_transform()));

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.reconfigure();
                return;
            }
            Err(_) => return,
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render2d-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear.0[0] as f64,
                            g: clear.0[1] as f64,
                            b: clear.0[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if !draw.idx.is_empty() {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_vertex_buffer(0, self.vert_buf.slice(..));
                pass.set_index_buffer(self.idx_buf.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..draw.idx.len() as u32, 0, 0..1);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
