// GPUモジュール: WebGPUデバイス・パイプライン・バッファの初期化とレンダリングを担当する

use crate::constants::*;
use crate::shader::SHADER;
use crate::geometry::{Vertex, Uni, STRIDE};

pub struct GpuState {
    pub surface:    wgpu::Surface<'static>,
    pub device:     wgpu::Device,
    pub queue:      wgpu::Queue,
    pub pipeline:   wgpu::RenderPipeline,
    pub uni_buf:    wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub vert_buf:   wgpu::Buffer,
    pub idx_buf:    wgpu::Buffer,
    pub depth_view: wgpu::TextureView,
    pub width:      u32,
    pub height:     u32,
}

impl GpuState {
    pub async fn new(canvas: web_sys::HtmlCanvasElement) -> Result<Self, String> {
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

    pub fn render(&self, verts:&[Vertex], idxs:&[u32], uni:&Uni){
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

/// デプステクスチャビューを生成するヘルパー関数
pub fn make_depth(device:&wgpu::Device,w:u32,h:u32)->wgpu::TextureView{
    device.create_texture(&wgpu::TextureDescriptor{
        label:Some("depth"),
        size:wgpu::Extent3d{width:w,height:h,depth_or_array_layers:1},
        mip_level_count:1,sample_count:1,dimension:wgpu::TextureDimension::D2,
        format:wgpu::TextureFormat::Depth32Float,
        usage:wgpu::TextureUsages::RENDER_ATTACHMENT,view_formats:&[],
    }).create_view(&wgpu::TextureViewDescriptor::default())
}
