pub mod renderer;
pub mod scene;
pub mod input;
pub mod math;
pub mod types;
pub mod graphics;

pub use renderer::{Renderer, RendererConfig};
pub use scene::{Scene, GameObject};
pub use input::InputState;
pub use math::{Vector3, Matrix4, Quaternion};
pub use graphics::Vertex;

use std::sync::Arc;
use log::info;

pub struct Engine {
    renderer: Arc<tokio::sync::Mutex<Renderer>>,
    scene: Arc<tokio::sync::Mutex<Scene>>,
    input_state: Arc<tokio::sync::Mutex<InputState>>,
    running: bool,
}

impl Engine {
    pub async fn new(config: RendererConfig) -> Result<Self, String> {
        info!("Initializing Game Engine");
        
        let renderer = Renderer::new(config).await?;
        let scene = Scene::new();
        let input_state = InputState::new();
        
        Ok(Engine {
            renderer: Arc::new(tokio::sync::Mutex::new(renderer)),
            scene: Arc::new(tokio::sync::Mutex::new(scene)),
            input_state: Arc::new(tokio::sync::Mutex::new(input_state)),
            running: true,
        })
    }
    
    pub async fn update(&mut self, delta_time: f32) {
        let mut scene = self.scene.lock().await;
        scene.update(delta_time);
    }
    
    pub async fn render(&self) -> Result<(), String> {
        let renderer = self.renderer.lock().await;
        let scene = self.scene.lock().await;
        renderer.render(&scene).await
    }
    
    pub fn is_running(&self) -> bool {
        self.running
    }
    
    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }
    
    pub fn get_input_state(&self) -> Arc<tokio::sync::Mutex<InputState>> {
        Arc::clone(&self.input_state)
    }
    
    pub fn get_scene(&self) -> Arc<tokio::sync::Mutex<Scene>> {
        Arc::clone(&self.scene)
    }
    
    pub fn get_renderer(&self) -> Arc<tokio::sync::Mutex<Renderer>> {
        Arc::clone(&self.renderer)
    }
}
