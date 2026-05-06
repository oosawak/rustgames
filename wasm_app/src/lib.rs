use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use bytemuck::{Pod, Zeroable};
use std::cell::RefCell;

// ─── Constants ────────────────────────────────────────────────────────────────

const MAZE_W: usize = 9;
const MAZE_H: usize = 9;
const WALL_H: f32 = 1.3;
const EYE_H:  f32 = 0.55;
const MAX_VERTS: usize = 12288;
const MAX_IDX:   usize = 20480;

const N: u8 = 1; const E: u8 = 2; const S: u8 = 4; const W_DIR: u8 = 8;

// ─── WGSL Shader ─────────────────────────────────────────────────────────────
//
//  col.a encodes material:
//    1.0  = normal (floor/ceiling/pillar)
//    2.0  = animated wall  → horizontal scan lines
//    3.0  = particle       → always bright, no fog floor
//
//  Uniforms: vp(64) + time(4) + warp(4) + pad(8) = 80 bytes

const SHADER: &str = r#"
struct Uni {
    vp   : mat4x4<f32>,
    time : f32,
    warp : f32,
    pad0 : f32,
    pad1 : f32,
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
    @location(2) world_y    : f32,
}

@vertex
fn vs_main(v: VIn) -> VOut {
    var o: VOut;
    let c = u.vp * vec4<f32>(v.pos, 1.0);
    o.clip    = c;
    o.col     = v.col;
    o.depth   = c.w;
    o.world_y = v.pos.y;
    return o;
}

@fragment
fn fs_main(v: VOut) -> @location(0) vec4<f32> {
    let fog_min = select(0.06, 0.55, v.col.a > 2.5); // particles brighter
    let fog = max(clamp(1.0 - v.depth / 14.0, 0.0, 1.0), fog_min);
    var rgb = v.col.rgb;

    // ── Animated wall: scan lines ──
    if v.col.a > 1.5 && v.col.a < 2.5 {
        // slow base pulse + fast bright scan line traveling upward
        let slow = sin(v.world_y * 5.0 - u.time * 2.0) * 0.08 + 0.92;
        let scan = max(0.0, sin(v.world_y * 18.0 - u.time * 9.0)) * 0.35;
        rgb = rgb * slow + vec3<f32>(0.0, scan * 0.3, scan);
    }

    rgb = min(rgb * fog, vec3<f32>(1.0));

    // ── Warp effect: chromatic distortion + white flash ──
    if u.warp > 0.01 {
        let flicker  = sin(u.time * 35.0) * 0.5 + 0.5;
        let flash    = u.warp * flicker * 0.45;
        let shift    = u.warp * sin(v.depth * 0.5 + u.time * 12.0) * 0.12;
        rgb = rgb + vec3<f32>(flash + shift, flash * 0.3, flash - shift);
        rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    }

    return vec4<f32>(rgb, 1.0);
}
"#;

// ─── GPU types ────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 3],
    _p:  f32,        // padding → col at offset 16
    col: [f32; 4],   // col.a = material (1=normal, 2=wall-anim, 3=particle)
}
const STRIDE: u64 = 32;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uni {
    vp:   [[f32; 4]; 4], // 64 bytes
    time: f32,
    warp: f32,
    pad:  [f32; 2],      // → 80 bytes total
}

// ─── Math ─────────────────────────────────────────────────────────────────────

type M4 = [[f32; 4]; 4];

fn mat_mul(a: M4, b: M4) -> M4 {
    let mut r = [[0f32; 4]; 4];
    for c in 0..4 { for row in 0..4 {
        r[c][row] = (0..4).map(|k| a[k][row] * b[c][k]).sum();
    }}
    r
}
fn perspective(fov: f32, asp: f32, n: f32, f: f32) -> M4 {
    let t = 1.0 / (fov * 0.5).tan();
    [[t/asp,0.0,0.0,0.0],[0.0,t,0.0,0.0],
     [0.0,0.0,f/(n-f),-1.0],[0.0,0.0,n*f/(n-f),0.0]]
}
fn norm3(v:[f32;3])->[f32;3]{let l=(v[0]*v[0]+v[1]*v[1]+v[2]*v[2]).sqrt();if l<1e-7{[0.0,0.0,1.0]}else{[v[0]/l,v[1]/l,v[2]/l]}}
fn sub3(a:[f32;3],b:[f32;3])->[f32;3]{[a[0]-b[0],a[1]-b[1],a[2]-b[2]]}
fn cross(a:[f32;3],b:[f32;3])->[f32;3]{[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]]}
fn dot3(a:[f32;3],b:[f32;3])->f32{a[0]*b[0]+a[1]*b[1]+a[2]*b[2]}
fn look_at(eye:[f32;3],ctr:[f32;3],up:[f32;3])->M4{
    let f=norm3(sub3(ctr,eye));let r=norm3(cross(f,norm3(up)));let u=cross(r,f);
    [[r[0],u[0],-f[0],0.0],[r[1],u[1],-f[1],0.0],[r[2],u[2],-f[2],0.0],
     [-dot3(r,eye),-dot3(u,eye),dot3(f,eye),1.0]]
}

// ─── RNG ──────────────────────────────────────────────────────────────────────

fn lcg(s: &mut u64) -> usize {
    *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*s >> 33) as usize
}
fn lcg_f(s: &mut u64) -> f32 { lcg(s) as f32 / (u32::MAX as f32) }

// ─── Maze ────────────────────────────────────────────────────────────────────

struct Maze { cells: [u8; MAZE_W * MAZE_H] }

impl Maze {
    fn new(seed: u64) -> Self {
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
    fn wall(&self,cx:usize,cz:usize,dir:u8)->bool{ self.cells[cz*MAZE_W+cx]&dir!=0 }
    fn can_move(&self,cx:usize,cz:usize,dir:u8)->Option<(usize,usize)>{
        if self.wall(cx,cz,dir){return None;}
        match dir{
            d if d==N=>if cz>0       {Some((cx,cz-1))}else{None},
            d if d==S=>if cz+1<MAZE_H{Some((cx,cz+1))}else{None},
            d if d==E=>if cx+1<MAZE_W{Some((cx+1,cz))}else{None},
            _        =>if cx>0       {Some((cx-1,cz))}else{None},
        }
    }
}

// ─── Particles ────────────────────────────────────────────────────────────────

struct Particle {
    pos:  [f32; 3],
    vel:  [f32; 3],
    life: f32,   // 1.0 = fresh, 0.0 = dead
}

// ─── Geometry ────────────────────────────────────────────────────────────────

const WALL_COL:  [f32;4] = [0.0, 0.85, 1.0, 2.0]; // a=2 → animated
const FLOOR_COL: [f32;4] = [0.0, 0.04, 0.12, 1.0];
const CEIL_COL:  [f32;4] = [0.0, 0.01, 0.04, 1.0];

fn quad(vs:&mut Vec<Vertex>,ix:&mut Vec<u32>,
        v0:[f32;3],v1:[f32;3],v2:[f32;3],v3:[f32;3],col:[f32;4]){
    let b=vs.len() as u32;
    for p in[v0,v1,v2,v3]{vs.push(Vertex{pos:p,_p:0.0,col});}
    ix.extend_from_slice(&[b,b+1,b+2,b,b+2,b+3]);
}

fn pillar(vs:&mut Vec<Vertex>,ix:&mut Vec<u32>,
          cx:f32,cz:f32,r:f32,h:f32,col:[f32;4]){
    for &(ax,az,bx,bz) in &[
        (cx-r,cz-r,cx+r,cz-r),(cx+r,cz+r,cx-r,cz+r),
        (cx+r,cz-r,cx+r,cz+r),(cx-r,cz+r,cx-r,cz-r),
    ]{ quad(vs,ix,[ax,0.0,az],[bx,0.0,bz],[bx,h,bz],[ax,h,az],col); }
}

/// Render a particle as a cross-shape (two perpendicular quads)
fn particle_cross(vs:&mut Vec<Vertex>,ix:&mut Vec<u32>,
                  p:&Particle){
    let size = 0.10 * p.life;
    let h = size * 0.5;
    let [x,y,z] = p.pos;
    let brightness = p.life * 3.0;  // emissive, survives fog
    let col = [0.2 * brightness, brightness, brightness, 3.0]; // a=3 → particle

    // X-aligned spark
    quad(vs,ix,[x-h,y-h,z],[x+h,y-h,z],[x+h,y+h,z],[x-h,y+h,z],col);
    // Z-aligned spark
    quad(vs,ix,[x,y-h,z-h],[x,y-h,z+h],[x,y+h,z+h],[x,y+h,z-h],col);
}

fn build_scene(maze:&Maze, time:f32, particles:&[Particle]) -> (Vec<Vertex>,Vec<u32>){
    let mut vs:Vec<Vertex>=Vec::with_capacity(2048);
    let mut ix:Vec<u32>   =Vec::with_capacity(4096);
    let mw=MAZE_W as f32; let mh=MAZE_H as f32;

    // Floor / ceiling
    quad(&mut vs,&mut ix,[0.0,0.0,0.0],[mw,0.0,0.0],[mw,0.0,mh],[0.0,0.0,mh],FLOOR_COL);
    quad(&mut vs,&mut ix,[0.0,WALL_H,mh],[mw,WALL_H,mh],[mw,WALL_H,0.0],[0.0,WALL_H,0.0],CEIL_COL);

    // Outer walls
    quad(&mut vs,&mut ix,[mw,0.0,0.0],[mw,WALL_H,0.0],[0.0,WALL_H,0.0],[0.0,0.0,0.0],WALL_COL);
    quad(&mut vs,&mut ix,[0.0,0.0,mh],[0.0,WALL_H,mh],[mw,WALL_H,mh],[mw,0.0,mh],WALL_COL);
    quad(&mut vs,&mut ix,[0.0,0.0,0.0],[0.0,WALL_H,0.0],[0.0,WALL_H,mh],[0.0,0.0,mh],WALL_COL);
    quad(&mut vs,&mut ix,[mw,0.0,mh],[mw,WALL_H,mh],[mw,WALL_H,0.0],[mw,0.0,0.0],WALL_COL);

    // Interior walls (both faces)
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

    // START pillar (bright green emissive)
    pillar(&mut vs,&mut ix, 0.5,0.5, 0.13,WALL_H*1.05, [0.2,3.0,0.5,1.0]);

    // GOAL pillar (pulsing magenta emissive)
    let p = (time*2.5).sin()*0.5+0.5;
    let(gx,gz)=((MAZE_W-1) as f32+0.5,(MAZE_H-1) as f32+0.5);
    pillar(&mut vs,&mut ix, gx,gz, 0.15,WALL_H*1.1, [3.0,p*0.4,3.0,1.0]);
    // GOAL floor glow
    let r2=0.45f32;
    quad(&mut vs,&mut ix,
        [gx-r2,0.02,gz-r2],[gx+r2,0.02,gz-r2],
        [gx+r2,0.02,gz+r2],[gx-r2,0.02,gz+r2],[2.0,p*0.3,2.0,1.0]);

    // Particles
    for p in particles.iter().filter(|p| p.life > 0.0) {
        particle_cross(&mut vs, &mut ix, p);
    }

    (vs,ix)
}

// ─── GPU State ────────────────────────────────────────────────────────────────

struct GpuState {
    surface:    wgpu::Surface<'static>,
    device:     wgpu::Device,
    queue:      wgpu::Queue,
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
        let (w,h) = (canvas.width(), canvas.height());
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL, ..Default::default()
        });
        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|e| e.to_string())?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface:Some(&surface),
            power_preference:wgpu::PowerPreference::None,
            force_fallback_adapter:false,
        }).await.ok_or("no adapter")?;
        let (device,queue) = adapter.request_device(&wgpu::DeviceDescriptor{
            label:None,
            required_features:wgpu::Features::empty(),
            required_limits:wgpu::Limits::downlevel_webgl2_defaults()
                .using_resolution(adapter.limits()),
        },None).await.map_err(|e| e.to_string())?;

        let caps = surface.get_capabilities(&adapter);
        let fmt  = caps.formats.iter().find(|f| f.is_srgb()).copied().unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration{
            usage:wgpu::TextureUsages::RENDER_ATTACHMENT,format:fmt,
            width:w,height:h,present_mode:wgpu::PresentMode::Fifo,
            alpha_mode:wgpu::CompositeAlphaMode::Opaque,
            view_formats:vec![],desired_maximum_frame_latency:2,
        };
        surface.configure(&device,&config);
        let depth_view = make_depth(&device,w,h);

        let uni_buf = device.create_buffer(&wgpu::BufferDescriptor{
            label:Some("uni"),size:std::mem::size_of::<Uni>() as u64,
            usage:wgpu::BufferUsages::UNIFORM|wgpu::BufferUsages::COPY_DST,
            mapped_at_creation:false,
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label:None,entries:&[wgpu::BindGroupLayoutEntry{
                binding:0,visibility:wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty:wgpu::BindingType::Buffer{
                    ty:wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset:false,min_binding_size:None,
                },count:None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            label:None,layout:&bgl,
            entries:&[wgpu::BindGroupEntry{binding:0,resource:uni_buf.as_entire_binding()}],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label:Some("maze3d"),source:wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label:None,bind_group_layouts:&[&bgl],push_constant_ranges:&[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label:Some("main"),layout:Some(&pll),
            vertex:wgpu::VertexState{
                module:&shader,entry_point:"vs_main",
                buffers:&[wgpu::VertexBufferLayout{
                    array_stride:STRIDE,step_mode:wgpu::VertexStepMode::Vertex,
                    attributes:&[
                        wgpu::VertexAttribute{offset:0, shader_location:0,format:wgpu::VertexFormat::Float32x3},
                        wgpu::VertexAttribute{offset:16,shader_location:1,format:wgpu::VertexFormat::Float32x4},
                    ],
                }],
            },
            fragment:Some(wgpu::FragmentState{
                module:&shader,entry_point:"fs_main",
                targets:&[Some(wgpu::ColorTargetState{
                    format:fmt,blend:None,write_mask:wgpu::ColorWrites::ALL,
                })],
            }),
            primitive:wgpu::PrimitiveState{
                topology:wgpu::PrimitiveTopology::TriangleList,
                front_face:wgpu::FrontFace::Ccw,cull_mode:None,
                ..Default::default()
            },
            depth_stencil:Some(wgpu::DepthStencilState{
                format:wgpu::TextureFormat::Depth32Float,
                depth_write_enabled:true,depth_compare:wgpu::CompareFunction::Less,
                stencil:wgpu::StencilState::default(),bias:wgpu::DepthBiasState::default(),
            }),
            multisample:wgpu::MultisampleState::default(),multiview:None,
        });
        let vert_buf = device.create_buffer(&wgpu::BufferDescriptor{
            label:Some("verts"),size:(MAX_VERTS*STRIDE as usize) as u64,
            usage:wgpu::BufferUsages::VERTEX|wgpu::BufferUsages::COPY_DST,
            mapped_at_creation:false,
        });
        let idx_buf = device.create_buffer(&wgpu::BufferDescriptor{
            label:Some("idxs"),size:(MAX_IDX*4) as u64,
            usage:wgpu::BufferUsages::INDEX|wgpu::BufferUsages::COPY_DST,
            mapped_at_creation:false,
        });
        Ok(GpuState{surface,device,queue,pipeline,
                    uni_buf,bind_group,vert_buf,idx_buf,depth_view,width:w,height:h})
    }

    fn render(&self, verts:&[Vertex], idxs:&[u32], uni:&Uni){
        self.queue.write_buffer(&self.uni_buf,0,bytemuck::bytes_of(uni));
        if verts.len()>MAX_VERTS||idxs.len()>MAX_IDX{return;}
        self.queue.write_buffer(&self.vert_buf,0,bytemuck::cast_slice(verts));
        self.queue.write_buffer(&self.idx_buf, 0,bytemuck::cast_slice(idxs));
        let frame=match self.surface.get_current_texture(){Ok(f)=>f,Err(_)=>return};
        let view =frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut enc=self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{label:None});
        {
            let mut pass=enc.begin_render_pass(&wgpu::RenderPassDescriptor{
                label:None,
                color_attachments:&[Some(wgpu::RenderPassColorAttachment{
                    view:&view,resolve_target:None,
                    ops:wgpu::Operations{
                        load:wgpu::LoadOp::Clear(wgpu::Color{r:0.0,g:0.0,b:0.02,a:1.0}),
                        store:wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment:Some(wgpu::RenderPassDepthStencilAttachment{
                    view:&self.depth_view,
                    depth_ops:Some(wgpu::Operations{
                        load:wgpu::LoadOp::Clear(1.0),store:wgpu::StoreOp::Discard,
                    }),
                    stencil_ops:None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0,&self.bind_group,&[]);
            pass.set_vertex_buffer(0,self.vert_buf.slice(..));
            pass.set_index_buffer(self.idx_buf.slice(..),wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..idxs.len() as u32,0,0..1);
        }
        self.queue.submit(std::iter::once(enc.finish()));
        frame.present();
    }
}

fn make_depth(device:&wgpu::Device,w:u32,h:u32)->wgpu::TextureView{
    device.create_texture(&wgpu::TextureDescriptor{
        label:Some("depth"),
        size:wgpu::Extent3d{width:w,height:h,depth_or_array_layers:1},
        mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,
        format:wgpu::TextureFormat::Depth32Float,
        usage:wgpu::TextureUsages::RENDER_ATTACHMENT,view_formats:&[],
    }).create_view(&wgpu::TextureViewDescriptor::default())
}

// ─── Game State ───────────────────────────────────────────────────────────────

struct GameState {
    gpu:         GpuState,
    maze:        Maze,
    px: usize, pz: usize,
    facing: u8,
    steps: u32,
    total_steps: u32,
    level: u32,
    level_clear: bool,
    warp_timer: f32,    // counts up when level_clear
    particles: Vec<Particle>,
    time: f32,
    prev_ts: f64,
}

impl GameState {
    async fn new(canvas_id: &str) -> Result<Self, String> {
        let doc    = web_sys::window().ok_or("no window")?.document().ok_or("no doc")?;
        let canvas = doc.get_element_by_id(canvas_id).ok_or("no canvas")?
            .dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| "not canvas")?;
        let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
        Ok(GameState{
            gpu: GpuState::new(canvas).await?,
            maze: Maze::new(seed),
            px:0, pz:0, facing:S,
            steps:0, total_steps:0, level:1,
            level_clear:false, warp_timer:0.0,
            particles: Vec::new(),
            time:0.0, prev_ts:0.0,
        })
    }

    fn tick(&mut self, ts: f64) {
        let dt = ((ts - self.prev_ts) / 1000.0).clamp(0.0, 0.1) as f32;
        self.prev_ts = ts;
        self.time    = (ts / 1000.0) as f32;

        // Update particles (gravity + fade)
        for p in &mut self.particles {
            if p.life <= 0.0 { continue; }
            p.pos[0] += p.vel[0];
            p.pos[1] += p.vel[1];
            p.pos[2] += p.vel[2];
            p.vel[1]  -= 0.003; // gravity
            p.life    -= dt * 1.8;
            if p.life < 0.0 { p.life = 0.0; }
        }
        // Remove dead particles occasionally to keep vec small
        if self.particles.len() > 200 {
            self.particles.retain(|p| p.life > 0.0);
        }

        // Warp timer
        if self.level_clear { self.warp_timer += dt; }

        let warp = self.warp_amount();
        let (verts,idxs) = build_scene(&self.maze, self.time, &self.particles);

        let eye = [self.px as f32+0.5, EYE_H, self.pz as f32+0.5];
        let fwd:[f32;3] = match self.facing {
            d if d==N=>[0.0,0.0,-1.0], d if d==S=>[0.0,0.0,1.0],
            d if d==E=>[1.0,0.0,0.0],  _=>[-1.0,0.0,0.0],
        };
        let ctr = [eye[0]+fwd[0],eye[1]+fwd[1],eye[2]+fwd[2]];
        let view = look_at(eye,ctr,[0.0,1.0,0.0]);
        let proj = perspective(70.0f32.to_radians(),
            self.gpu.width as f32/self.gpu.height as f32, 0.05, 50.0);
        let uni = Uni{ vp:mat_mul(proj,view), time:self.time, warp, pad:[0.0;2] };
        self.gpu.render(&verts,&idxs,&uni);
    }

    fn warp_amount(&self) -> f32 {
        if !self.level_clear { return 0.0; }
        // bell curve: 0→1→0 over 1.5 s
        let t = (self.warp_timer / 1.5).clamp(0.0, 1.0);
        (t * std::f32::consts::PI).sin()
    }

    // action: 0=forward 1=turn_left 2=turn_right 3=backward
    fn act(&mut self, action: i32) {
        if self.level_clear { return; }
        match action {
            1 => self.facing=match self.facing{d if d==N=>W_DIR,d if d==W_DIR=>S,d if d==S=>E,_=>N},
            2 => self.facing=match self.facing{d if d==N=>E,d if d==E=>S,d if d==S=>W_DIR,_=>N},
            0 => { let f=self.facing; self.try_move(f); }
            3 => {
                let b=match self.facing{d if d==N=>S,d if d==S=>N,d if d==E=>W_DIR,_=>E};
                self.try_move(b);
            }
            _ => {}
        }
    }

    fn try_move(&mut self, dir: u8) {
        match self.maze.can_move(self.px, self.pz, dir) {
            Some((nx,nz)) => {
                self.px=nx; self.pz=nz; self.steps+=1;
                if self.px==MAZE_W-1 && self.pz==MAZE_H-1 {
                    self.total_steps+=self.steps;
                    self.level_clear=true;
                    self.warp_timer=0.0;
                    // spawn goal burst particles
                    self.spawn_goal_particles();
                }
            }
            None => {
                // Hit a wall → spawn collision particles
                self.spawn_hit_particles(dir);
            }
        }
    }

    fn spawn_hit_particles(&mut self, dir: u8) {
        let mut rng = (self.time * 100000.0) as u64 | 1;
        // Wall face center position
        let (wx, wz) = match dir {
            d if d==N => (self.px as f32+0.5, self.pz as f32),
            d if d==S => (self.px as f32+0.5, self.pz as f32+1.0),
            d if d==E => (self.px as f32+1.0, self.pz as f32+0.5),
            _         => (self.px as f32,      self.pz as f32+0.5),
        };
        for _ in 0..12 {
            let angle = lcg_f(&mut rng) * std::f32::consts::TAU;
            let speed = 0.025 + lcg_f(&mut rng) * 0.04;
            self.particles.push(Particle {
                pos:  [wx, 0.2 + lcg_f(&mut rng) * 0.6, wz],
                vel:  [angle.cos()*speed, 0.03+lcg_f(&mut rng)*0.04, angle.sin()*speed],
                life: 1.0,
            });
        }
    }

    fn spawn_goal_particles(&mut self) {
        let mut rng = (self.time * 99999.0) as u64 | 3;
        let (gx,gz) = ((MAZE_W-1) as f32+0.5, (MAZE_H-1) as f32+0.5);
        for _ in 0..30 {
            let angle = lcg_f(&mut rng) * std::f32::consts::TAU;
            let speed = 0.04 + lcg_f(&mut rng) * 0.06;
            self.particles.push(Particle {
                pos:  [gx, 0.5 + lcg_f(&mut rng) * 0.8, gz],
                vel:  [angle.cos()*speed, 0.05+lcg_f(&mut rng)*0.06, angle.sin()*speed],
                life: 1.0,
            });
        }
    }

    fn next_level(&mut self) {
        let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
        self.maze=Maze::new(seed);
        self.px=0; self.pz=0; self.facing=S; self.steps=0;
        self.level+=1; self.level_clear=false; self.warp_timer=0.0;
        self.particles.clear();
    }

    fn reset(&mut self) {
        let seed = (js_sys::Math::random() * u64::MAX as f64) as u64;
        self.maze=Maze::new(seed);
        self.px=0; self.pz=0; self.facing=S;
        self.steps=0; self.total_steps=0; self.level=1;
        self.level_clear=false; self.warp_timer=0.0;
        self.particles.clear();
    }
}

// ─── WASM exports ─────────────────────────────────────────────────────────────

thread_local! {
    static STATE: RefCell<Option<GameState>> = RefCell::new(None);
}

#[wasm_bindgen]
pub async fn init_maze3d(canvas_id: &str) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    let s = GameState::new(canvas_id).await.map_err(|e| JsValue::from_str(&e))?;
    STATE.with(|st| *st.borrow_mut() = Some(s));
    Ok(())
}
#[wasm_bindgen] pub fn tick_maze3d(ts:f64){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.tick(ts);}}); }
#[wasm_bindgen] pub fn move_maze3d(a:i32){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.act(a);}}); }
#[wasm_bindgen] pub fn next_level_maze3d(){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.next_level();}}); }
#[wasm_bindgen] pub fn reset_maze3d(){ STATE.with(|s|{if let Some(g)=s.borrow_mut().as_mut(){g.reset();}}); }
#[wasm_bindgen] pub fn steps_maze3d()->u32       { STATE.with(|s|s.borrow().as_ref().map(|g|g.steps).unwrap_or(0)) }
#[wasm_bindgen] pub fn total_steps_maze3d()->u32 { STATE.with(|s|s.borrow().as_ref().map(|g|g.total_steps).unwrap_or(0)) }
#[wasm_bindgen] pub fn level_maze3d()->u32       { STATE.with(|s|s.borrow().as_ref().map(|g|g.level).unwrap_or(1)) }
#[wasm_bindgen] pub fn level_clear_maze3d()->bool{ STATE.with(|s|s.borrow().as_ref().map(|g|g.level_clear).unwrap_or(false)) }
#[wasm_bindgen] pub fn warp_maze3d()->f32        { STATE.with(|s|s.borrow().as_ref().map(|g|g.warp_amount()).unwrap_or(0.0)) }
#[wasm_bindgen] pub fn warp_done_maze3d()->bool  { STATE.with(|s|s.borrow().as_ref().map(|g|g.level_clear && g.warp_timer>=1.5).unwrap_or(false)) }
