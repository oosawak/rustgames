use cgmath::Vector3;

#[derive(Clone, Debug)]
pub struct Particle {
    pub position: Vector3<f32>,
    pub velocity: Vector3<f32>,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: (f32, f32, f32, f32),
    pub size: f32,
}

impl Particle {
    pub fn new(
        position: Vector3<f32>,
        velocity: Vector3<f32>,
        lifetime: f32,
        color: (f32, f32, f32, f32),
        size: f32,
    ) -> Self {
        Particle {
            position,
            velocity,
            lifetime,
            max_lifetime: lifetime,
            color,
            size,
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        self.lifetime -= delta_time;
        self.position += self.velocity * delta_time;
    }
    
    pub fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem {
            particles: Vec::new(),
        }
    }
    
    pub fn emit(&mut self, particle: Particle) {
        self.particles.push(particle);
    }
    
    pub fn emit_burst(
        &mut self,
        position: Vector3<f32>,
        count: usize,
        velocity_magnitude: f32,
        lifetime: f32,
        color: (f32, f32, f32, f32),
    ) {
        for i in 0..count {
            let angle = (i as f32 / count as f32) * std::f32::consts::PI * 2.0;
            let velocity = Vector3::new(
                velocity_magnitude * angle.cos(),
                velocity_magnitude * 0.5,
                velocity_magnitude * angle.sin(),
            );
            
            let particle = Particle::new(position, velocity, lifetime, color, 0.1);
            self.emit(particle);
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        for particle in &mut self.particles {
            particle.update(delta_time);
        }
        self.particles.retain(|p| p.is_alive());
    }
    
    pub fn get_particles(&self) -> &[Particle] {
        &self.particles
    }
    
    pub fn clear(&mut self) {
        self.particles.clear();
    }
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new()
    }
}
