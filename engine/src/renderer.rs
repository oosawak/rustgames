use wgpu::*;
use log::info;

use crate::scene::Scene;
use crate::types::Color;

#[derive(Clone, Debug)]
pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

impl Default for RendererConfig {
    fn default() -> Self {
        RendererConfig {
            width: 1280,
            height: 720,
            title: "Rust Game Engine".to_string(),
        }
    }
}

pub struct Renderer {
    pub device: Device,
    pub queue: Queue,
    pub surface: Option<Surface<'static>>,
    pub config: SurfaceConfiguration,
    pub size: (u32, u32),
    pub clear_color: Color,
}

impl Renderer {
    pub async fn new(config: RendererConfig) -> Result<Self, String> {
        let size = (config.width, config.height);
        info!("Creating renderer with size: {:?}", size);
        
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .ok_or("Failed to find GPU adapter")?;
        
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("Game Device"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to create device: {}", e))?;
        
        info!("GPU adapter and device created successfully");
        
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8UnormSrgb,
            width: config.width,
            height: config.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        Ok(Renderer {
            device,
            queue,
            surface: None,
            config: surface_config,
            size,
            clear_color: Color::black(),
        })
    }
    
    pub async fn render(&self, _scene: &Scene) -> Result<(), String> {
        info!("Rendering frame");
        Ok(())
    }
    
    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }
    
    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            info!("Renderer resized to: {:?}", new_size);
        }
    }
}
