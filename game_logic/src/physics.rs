use cgmath::Vector3;

pub struct RigidBody {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub acceleration: Vector3<f32>,
    pub mass: f32,
}

impl RigidBody {
    pub fn new(position: Vector3<f32>, mass: f32) -> Self {
        RigidBody {
            position,
            velocity: Vector3::new(0.0, 0.0, 0.0),
            acceleration: Vector3::new(0.0, -9.8, 0.0), // Gravity
            mass,
        }
    }
    
    pub fn apply_force(&mut self, force: Vector3<f32>) {
        self.acceleration += force / self.mass;
    }
    
    pub fn update(&mut self, delta_time: f32) {
        self.velocity += self.acceleration * delta_time;
        self.position += self.velocity * delta_time;
        
        // Reset acceleration for next frame
        self.acceleration = Vector3::new(0.0, -9.8, 0.0);
    }
}

pub struct PhysicsWorld {
    bodies: Vec<RigidBody>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        PhysicsWorld {
            bodies: Vec::new(),
        }
    }
    
    pub fn add_body(&mut self, body: RigidBody) {
        self.bodies.push(body);
    }
    
    pub fn update(&mut self, delta_time: f32) {
        for body in &mut self.bodies {
            body.update(delta_time);
        }
    }
    
    pub fn get_bodies(&self) -> &[RigidBody] {
        &self.bodies
    }
    
    pub fn get_bodies_mut(&mut self) -> &mut [RigidBody] {
        &mut self.bodies
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}
