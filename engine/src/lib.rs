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

use std::sync::{Arc, Mutex};
use log::info;

pub struct Engine {
    #[cfg(not(target_arch = "wasm32"))]
    renderer: Arc<tokio::sync::Mutex<Renderer>>,
    #[cfg(target_arch = "wasm32")]
    renderer: Arc<Mutex<Renderer>>,
    scene: Arc<Mutex<Scene>>,
    input_state: Arc<Mutex<InputState>>,
    running: bool,
}

impl Engine {
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn new(config: RendererConfig) -> Result<Self, String> {
        info!("Initializing Game Engine");
        
        let renderer = Renderer::new(config).await?;
        let scene = Scene::new();
        let input_state = InputState::new();
        
        Ok(Engine {
            renderer: Arc::new(tokio::sync::Mutex::new(renderer)),
            scene: Arc::new(Mutex::new(scene)),
            input_state: Arc::new(Mutex::new(input_state)),
            running: true,
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn new(_config: RendererConfig) -> Result<Self, String> {
        info!("Initializing Game Engine (WASM mode - renderer stub)");
        
        let scene = Scene::new();
        let input_state = InputState::new();
        
        // WASM doesn't support actual renderer initialization
        // Use a stub implementation
        let renderer = Renderer::new_stub();
        
        Ok(Engine {
            renderer: Arc::new(Mutex::new(renderer)),
            scene: Arc::new(Mutex::new(scene)),
            input_state: Arc::new(Mutex::new(input_state)),
            running: true,
        })
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn update(&mut self, delta_time: f32) {
        let mut scene = self.scene.lock().unwrap();
        scene.update(delta_time);
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn update(&mut self, delta_time: f32) {
        let mut scene = self.scene.lock().unwrap();
        scene.update(delta_time);
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn render(&self) -> Result<(), String> {
        let renderer = self.renderer.lock().unwrap();
        let scene = self.scene.lock().unwrap();
        renderer.render(&scene).await
    }

    #[cfg(target_arch = "wasm32")]
    pub async fn render(&self) -> Result<(), String> {
        // WASM stub - actual rendering handled by JavaScript
        Ok(())
    }
    
    pub fn is_running(&self) -> bool {
        self.running
    }
    
    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }
    
    pub fn get_input_state(&self) -> Arc<Mutex<InputState>> {
        Arc::clone(&self.input_state)
    }
    
    pub fn get_scene(&self) -> Arc<Mutex<Scene>> {
        Arc::clone(&self.scene)
    }
    
    pub fn get_renderer(&self) -> Arc<Mutex<Renderer>> {
        Arc::clone(&self.renderer)
    }
}
