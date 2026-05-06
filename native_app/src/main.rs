use engine::{Engine, RendererConfig};
use game_logic::GameState;
use std::time::Instant;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // Create window
    let event_loop = EventLoop::new()?;
    let _window = WindowBuilder::new()
        .with_title("3D Puzzle Game - Native")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)?;
    
    // Initialize engine
    let config = RendererConfig {
        width: 1280,
        height: 720,
        title: "Rust Game - Native".to_string(),
    };
    
    let mut engine = Engine::new(config).await?;
    let mut game_state = GameState::new();
    
    let mut last_frame = Instant::now();
    
    println!("Game initialized successfully!");
    println!("Controls: WASD to move, ESC to quit");
    
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { window_id: _, event } => {
                match event {
                    WindowEvent::Resized(_size) => {
                        // Window resized
                    }
                    WindowEvent::CloseRequested => {
                        println!("Window close requested");
                        elwt.exit();
                    }
                    WindowEvent::KeyboardInput { .. } => {
                        // Handle keyboard input
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                let now = Instant::now();
                let delta_time = now.duration_since(last_frame).as_secs_f32();
                last_frame = now;
                
                // Update game logic
                game_state.update(delta_time);
            }
            _ => {}
        }
    })?;
    
    Ok(())
}
