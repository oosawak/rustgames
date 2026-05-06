use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures;
use bytemuck::{Pod, Zeroable};
use std::cell::RefCell;

// ─── Constants ────────────────────────────────────────────────────────────────

const MAZE_W: usize = 9;
const MAZE_H: usize = 9;
const WALL_H: f32 = 1.3;
const EYE_H: f32 = 0.55;
const FOG_END: f32 = 9.0;
const MAX_VERTS: usize = 8192;
const MAX_IDX: usize = 16384;

// Wall bitmask: N=1 E=2 S=4 W=8
const N: u8 = 1;
const E: u8 = 2;
const S: u8 = 4;
const W_DIR: u8 = 8;

// ─── WGSL Shader ─────────────────────────────────────────────────────────────

const SHADER: &str = r#"
struct Uni {
    vp   : mat4x4<f32>,
    time : f32,
    pad0 : f32,
    pad1 : f32,
    pad2 : f32,
}
@group(0) @binding(0) var<uniform> u: Uni;

struct VIn {
    @location(0) pos : vec3<f32>,
    @location(1) col : vec4<f32>,
}
struct VOut {
    @builtin(position) clip : vec4<f32>,
    @location(0) col        : vec4<f32>,
    @location(1) depth      : f32,
}

@vertex
fn vs_main(v: VIn) -> VOut {
    var o: VOut;
    let c = u.vp * vec4<f32>(v.pos, 1.0);
    o.clip  = c;
    o.col   = v.col;
    o.depth = c.w;
    return o;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
    let fog = clamp(1.0 - v.depth / 9.0, 0.0, 1.0);
    return vec4<f32>(v.col.rgb * fog, v.col.a);
}
"#;

// ─── GPU types ────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 3],
    _p:  f32,       // padding so col is at offset 16
    col: [f32; 4],
}

// Total: 32 bytes (stride)
const STRIDE: u64 = 32;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uni {
    vp:   [[f32; 4]; 4], // 64 bytes at offset 0
    time: f32,           // 4 bytes at offset 64
    pad:  [f32; 3],      // 12 bytes padding → total 80 bytes
}

// ─── Math ─────────────────────────────────────────────────────────────────────

type M4 = [[f32; 4]; 4];

fn mat_mul(a: M4, b: M4) -> M4 {
    let mut r = [[0f32; 4]; 4];
    for c in 0..4 {
        for row in 0..4 {
            r[c][row] = (0..4).map(|k| a[k][row] * b[c][k]).sum();
        }
    }
    r
}

fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> M4 {
    let f = 1.0 / (fov_y * 0.5).tan();
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, far / (near - far), -1.0],
        [0.0, 0.0, near * far / (near - far), 0.0],
    ]
}

fn norm3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
    if l < 1e-7 { [0.0, 0.0, 1.0] } else { [v[0]/l, v[1]/l, v[2]/l] }
}
fn sub3(a: [f32;3], b: [f32;3]) -> [f32;3] { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }
fn cross(a: [f32;3], b: [f32;3]) -> [f32;3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}
fn dot3(a: [f32;3], b: [f32;3]) -> f32 { a[0]*b[0]+a[1]*b[1]+a[2]*b[2] }

fn look_at(eye: [f32;3], ctr: [f32;3], up: [f32;3]) -> M4 {
    let f = norm3(sub3(ctr, eye));
    let r = norm3(cross(f, norm3(up)));
    let u = cross(r, f);
    [
        [r[0], u[0], -f[0], 0.0],
        [r[1], u[1], -f[1], 0.0],
        [r[2], u[2], -f[2], 0.0],
        [-dot3(r,eye), -dot3(u,eye), dot3(f,eye), 1.0],
    ]
}

// ─── Maze generation (recursive backtracker) ─────────────────────────────────

struct Maze {
    cells: [u8; MAZE_W * MAZE_H], // bitmask: walls that EXIST (1=blocked)
}

impl Maze {
    fn new(seed: u64) -> Self {
        let mut rng = seed | 1;
        let mut cells = [N | E | S | W_DIR; MAZE_W * MAZE_H];
        let mut vis = [false; MAZE_W * MAZE_H];
        let mut stack: Vec<(usize, usize)> = vec![(0, 0)];
        vis[0] = true;

        while let Some(&(cx, cz)) = stack.last() {
            let mut nb: Vec<(usize, usize, u8, u8)> = Vec::new();
            if cz > 0 && !vis[(cz-1)*MAZE_W+cx] { nb.push((cx, cz-1, N, S)); }
            if cz < MAZE_H-1 && !vis[(cz+1)*MAZE_W+cx] { nb.push((cx, cz+1, S, N)); }
            if cx > 0 && !vis[cz*MAZE_W+cx-1] { nb.push((cx-1, cz, W_DIR, E)); }
            if cx < MAZE_W-1 && !vis[cz*MAZE_W+cx+1] { nb.push((cx+1, cz, E, W_DIR)); }

            if nb.is_empty() {
                stack.pop();
            } else {
                rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let (nx, nz, from_dir, to_dir) = nb[((rng >> 33) as usize) % nb.len()];
                cells[cz*MAZE_W+cx]  &= !from_dir;
                cells[nz*MAZE_W+nx]  &= !to_dir;
                vis[nz*MAZE_W+nx] = true;
                stack.push((nx, nz));
            }
        }
        Maze { cells }
    }

    fn wall(&self, cx: usize, cz: usize, dir: u8) -> bool {
        self.cells[cz * MAZE_W + cx] & dir != 0
    }

    fn can_move(&self, cx: usize, cz: usize, dir: u8) -> Option<(usize, usize)> {
        if self.wall(cx, cz, dir) { return None; }
        match dir {
            d if d == N     => if cz > 0        { Some((cx, cz-1)) } else { None },
            d if d == S     => if cz+1 < MAZE_H { Some((cx, cz+1)) } else { None },
            d if d == E     => if cx+1 < MAZE_W { Some((cx+1, cz)) } else { None },
            _/*W_DIR*/      => if cx > 0        { Some((cx-1, cz)) } else { None },
        }
    }
}

// ─── Geometry builder ─────────────────────────────────────────────────────────

fn quad(vs: &mut Vec<Vertex>, ix: &mut Vec<u32>,
        v0:[f32;3], v1:[f32;3], v2:[f32;3], v3:[f32;3], col:[f32;4]) {
    let b = vs.len() as u32;
    for p in [v0,v1,v2,v3] { vs.push(Vertex { pos:p, _p:0.0, col }); }
    ix.extend_from_slice(&[b,b+1,b+2, b,b+2,b+3]);
}

fn build_scene(maze: &Maze, time: f32) -> (Vec<Vertex>, Vec<u32>) {
    let mut vs: Vec<Vertex> = Vec::with_capacity(1024);
    let mut ix: Vec<u32>    = Vec::with_capacity(2048);

    let wc  = [0.0f32, 0.85, 1.0, 1.0]; // cyan wall
    let fc  = [0.0f32, 0.04, 0.12, 1.0]; // dark navy floor
    let cc  = [0.0f32, 0.01, 0.04, 1.0]; // near-black ceiling
    let mw  = MAZE_W as f32;
    let mh  = MAZE_H as f32;

    // Floor (full maze)
    quad(&mut vs, &mut ix,
        [0.0,0.0,0.0], [mw,0.0,0.0], [mw,0.0,mh], [0.0,0.0,mh], fc);
    // Ceiling
    quad(&mut vs, &mut ix,
        [0.0,WALL_H,mh],[mw,WALL_H,mh],[mw,WALL_H,0.0],[0.0,WALL_H,0.0], cc);

    // Outer boundary walls (always present, single face pointing inward)
    // North (z=0, facing south)
    quad(&mut vs, &mut ix,
        [mw,0.0,0.0],[mw,WALL_H,0.0],[0.0,WALL_H,0.0],[0.0,0.0,0.0], wc);
    // South (z=H, facing north)
    quad(&mut vs, &mut ix,
        [0.0,0.0,mh],[0.0,WALL_H,mh],[mw,WALL_H,mh],[mw,0.0,mh], wc);
    // West (x=0, facing east)
    quad(&mut vs, &mut ix,
        [0.0,0.0,0.0],[0.0,WALL_H,0.0],[0.0,WALL_H,mh],[0.0,0.0,mh], wc);
    // East (x=W, facing west)
    quad(&mut vs, &mut ix,
        [mw,0.0,mh],[mw,WALL_H,mh],[mw,WALL_H,0.0],[mw,0.0,0.0], wc);

    // Interior walls (render both faces so player sees from either side)
    for cz in 0..MAZE_H {
        for cx in 0..MAZE_W {
            let x = cx as f32;
            let z = cz as f32;

            // South wall (z+1 boundary between rows cz and cz+1)
            if cz + 1 < MAZE_H && maze.wall(cx, cz, S) {
                quad(&mut vs, &mut ix,
                    [x,0.0,z+1.0],[x+1.0,0.0,z+1.0],[x+1.0,WALL_H,z+1.0],[x,WALL_H,z+1.0], wc);
                quad(&mut vs, &mut ix,
                    [x,0.0,z+1.0],[x,WALL_H,z+1.0],[x+1.0,WALL_H,z+1.0],[x+1.0,0.0,z+1.0], wc);
            }
            // East wall (x+1 boundary between cols cx and cx+1)
            if cx + 1 < MAZE_W && maze.wall(cx, cz, E) {
                quad(&mut vs, &mut ix,
                    [x+1.0,0.0,z],[x+1.0,0.0,z+1.0],[x+1.0,WALL_H,z+1.0],[x+1.0,WALL_H,z], wc);
                quad(&mut vs, &mut ix,
                    [x+1.0,0.0,z],[x+1.0,WALL_H,z],[x+1.0,WALL_H,z+1.0],[x+1.0,0.0,z+1.0], wc);
            }
        }
    }

    // Goal pillar (pulsing magenta at cell (W-1, H-1))
    let p = (time * 2.5).sin() * 0.4 + 0.7;
    let gc = [1.0f32, p * 0.2, 1.0, 1.0];
    let gx = (MAZE_W-1) as f32 + 0.5;
    let gz = (MAZE_H-1) as f32 + 0.5;
    let r = 0.14f32;
    // 4 faces of goal pillar (all faces visible → no culling needed)
    for &(ax, az, bx, bz) in &[
        (gx-r, gz-r, gx+r, gz-r), // south face
        (gx+r, gz+r, gx-r, gz+r), // north face
        (gx+r, gz-r, gx+r, gz+r), // east face
        (gx-r, gz+r, gx-r, gz-r), // west face
    ] {
        quad(&mut vs, &mut ix,
            [ax,0.0,az],[bx,0.0,bz],[bx,WALL_H*1.1,bz],[ax,WALL_H*1.1,az], gc);
    }

    // Start marker (yellow glow at 0,0)
    let sc = [1.0f32, 0.9, 0.0, 1.0];
    let sr = 0.08f32;
    quad(&mut vs, &mut ix,
        [0.5-sr, 0.01, 0.5-sr], [0.5+sr, 0.01, 0.5-sr],
        [0.5+sr, 0.01, 0.5+sr], [0.5-sr, 0.01, 0.5+sr], sc);

    (vs, ix)
}

// ─── GPU State ────────────────────────────────────────────────────────────────

struct GpuState {
    surface:    wgpu::Surface<'static>,
    device:     wgpu::Device,
    queue:      wgpu::Queue,
    config:     wgpu::SurfaceConfiguration,
    pipeline:   wgpu::RenderPipeline,
    uni_buf:    wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    vert_buf:   wgpu::Buffer,
    idx_buf:    wgpu::Buffer,
    depth_view: wgpu::TextureView,
    width:      u32,
    height:     u32,
}

impl GpuState {
    async fn new(canvas: web_sys::HtmlCanvasElement) -> Result<Self, String> {
        let w = canvas.width();
        let h = canvas.height();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|e| e.to_string())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "WebGL adapter not found".to_string())?;

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
        let fmt = caps.formats.iter().find(|f| f.is_srgb()).copied()
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,
            format:       fmt,
            width:        w,
            height:       h,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode:   wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let depth_view = make_depth(&device, w, h);

        let uni_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("uni"),
            size:               std::mem::size_of::<Uni>() as u64,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label:   None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding:    0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty:                 wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size:   None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   None,
            layout:  &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding:  0,
                resource: uni_buf.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("maze3d"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label:                None,
            bind_group_layouts:   &[&bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label:  Some("main"),
            layout: Some(&pll),
            vertex: wgpu::VertexState {
                module:      &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: STRIDE,
                    step_mode:    wgpu::VertexStepMode::Vertex,
                    attributes:   &[
                        wgpu::VertexAttribute { offset: 0,  shader_location: 0, format: wgpu::VertexFormat::Float32x3 },
                        wgpu::VertexAttribute { offset: 16, shader_location: 1, format: wgpu::VertexFormat::Float32x4 },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module:      &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format:     fmt,
                    blend:      None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology:           wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face:         wgpu::FrontFace::Ccw,
                cull_mode:          None,
                polygon_mode:       wgpu::PolygonMode::Fill,
                unclipped_depth:    false,
                conservative:       false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format:              wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare:       wgpu::CompareFunction::Less,
                stencil:             wgpu::StencilState::default(),
                bias:                wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview:   None,
        });

        let vert_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("verts"),
            size:               (MAX_VERTS * STRIDE as usize) as u64,
            usage:              wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let idx_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("idxs"),
            size:               (MAX_IDX * 4) as u64,
            usage:              wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(GpuState { surface, device, queue, config, pipeline,
                       uni_buf, bind_group, vert_buf, idx_buf, depth_view,
                       width: w, height: h })
    }

    fn render(&self, verts: &[Vertex], idxs: &[u32], uni: &Uni) {
        self.queue.write_buffer(&self.uni_buf, 0, bytemuck::bytes_of(uni));

        if verts.len() > MAX_VERTS || idxs.len() > MAX_IDX { return; }
        self.queue.write_buffer(&self.vert_buf, 0, bytemuck::cast_slice(verts));
        self.queue.write_buffer(&self.idx_buf,  0, bytemuck::cast_slice(idxs));

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => return,
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view:           &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load:  wgpu::LoadOp::Clear(wgpu::Color { r:0.0, g:0.0, b:0.02, a:1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load:  wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.vert_buf.slice(..));
            pass.set_index_buffer(self.idx_buf.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..idxs.len() as u32, 0, 0..1);
        }

        self.queue.submit(std::iter::once(enc.finish()));
        frame.present();
    }
}

fn make_depth(device: &wgpu::Device, w: u32, h: u32) -> wgpu::TextureView {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size:  wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count:    1,
        dimension:       wgpu::TextureDimension::D2,
        format:          wgpu::TextureFormat::Depth32Float,
        usage:           wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats:    &[],
    })
    .create_view(&wgpu::TextureViewDescriptor::default())
}

// ─── Game State ───────────────────────────────────────────────────────────────

struct GameState {
    gpu:     GpuState,
    maze:    Maze,
    px:      usize, // player cell X
    pz:      usize, // player cell Z
    facing:  u8,    // N E S W_DIR
    steps:   u32,
    time:    f32,
    won:     bool,
}

impl GameState {
    async fn new(canvas_id: &str) -> Result<Self, String> {
        let window = web_sys::window().ok_or("no window")?;
        let doc    = window.document().ok_or("no document")?;
        let el     = doc.get_element_by_id(canvas_id).ok_or("canvas not found")?;
        let canvas = el.dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| "element is not canvas")?;

        let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
        let maze = Maze::new(seed);
        let gpu  = GpuState::new(canvas).await?;

        Ok(GameState { gpu, maze, px: 0, pz: 0, facing: S, steps: 0, time: 0.0, won: false })
    }

    fn tick(&mut self, ts: f64) {
        self.time = (ts / 1000.0) as f32;

        let (verts, idxs) = build_scene(&self.maze, self.time);

        // Camera
        let eye = [self.px as f32 + 0.5, EYE_H, self.pz as f32 + 0.5];
        let fwd: [f32;3] = match self.facing {
            d if d == N     => [0.0, 0.0, -1.0],
            d if d == S     => [0.0, 0.0,  1.0],
            d if d == E     => [1.0, 0.0,  0.0],
            _/*W_DIR*/      => [-1.0, 0.0, 0.0],
        };
        let ctr = [eye[0]+fwd[0], eye[1]+fwd[1], eye[2]+fwd[2]];
        let view = look_at(eye, ctr, [0.0, 1.0, 0.0]);
        let proj = perspective(
            70.0f32.to_radians(),
            self.gpu.width as f32 / self.gpu.height as f32,
            0.05, 50.0);
        let vp = mat_mul(proj, view);

        let uni = Uni { vp, time: self.time, pad: [0.0; 3] };
        self.gpu.render(&verts, &idxs, &uni);
    }

    // action: 0=forward 1=turn_left 2=turn_right 3=backward
    fn act(&mut self, action: i32) {
        if self.won { return; }
        match action {
            1 => self.facing = match self.facing {
                d if d == N => W_DIR, d if d == W_DIR => S,
                d if d == S => E, _ => N },
            2 => self.facing = match self.facing {
                d if d == N => E, d if d == E => S,
                d if d == S => W_DIR, _ => N },
            0 => { self.try_move(self.facing); }
            3 => {
                let back = match self.facing {
                    d if d == N => S, d if d == S => N,
                    d if d == E => W_DIR, _ => E };
                self.try_move(back);
            }
            _ => {}
        }
    }

    fn try_move(&mut self, dir: u8) {
        let (nx, nz) = match self.maze.can_move(self.px, self.pz, dir) {
            Some(p) => p,
            None    => return,
        };
        self.px    = nx;
        self.pz    = nz;
        self.steps += 1;
        if self.px == MAZE_W-1 && self.pz == MAZE_H-1 {
            self.won = true;
        }
    }

    fn reset(&mut self) {
        let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
        self.maze  = Maze::new(seed);
        self.px    = 0;
        self.pz    = 0;
        self.facing = S;
        self.steps  = 0;
        self.time   = 0.0;
        self.won    = false;
    }
}

// ─── Thread-local state & WASM exports ───────────────────────────────────────

thread_local! {
    static STATE: RefCell<Option<GameState>> = RefCell::new(None);
}

#[wasm_bindgen]
pub async fn init_maze3d(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let state = GameState::new(canvas_id).await
        .map_err(|e| JsValue::from_str(&e))?;
    STATE.with(|s| *s.borrow_mut() = Some(state));
    Ok(())
}

#[wasm_bindgen]
pub fn tick_maze3d(ts: f64) {
    STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() { g.tick(ts); }
    });
}

#[wasm_bindgen]
pub fn move_maze3d(action: i32) {
    STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() { g.act(action); }
    });
}

#[wasm_bindgen]
pub fn reset_maze3d() {
    STATE.with(|s| {
        if let Some(g) = s.borrow_mut().as_mut() { g.reset(); }
    });
}

#[wasm_bindgen]
pub fn steps_maze3d() -> u32 {
    STATE.with(|s| s.borrow().as_ref().map(|g| g.steps).unwrap_or(0))
}

#[wasm_bindgen]
pub fn won_maze3d() -> bool {
    STATE.with(|s| s.borrow().as_ref().map(|g| g.won).unwrap_or(false))
}
