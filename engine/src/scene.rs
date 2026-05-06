use crate::math::Vector3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameObject {
    pub id: u32,
    pub name: String,
    #[serde(skip)]
    pub position: Vector3,
    #[serde(skip)]
    pub rotation: Vector3,
    #[serde(skip)]
    pub scale: Vector3,
    pub active: bool,
}

impl GameObject {
    pub fn new(id: u32, name: String) -> Self {
        GameObject {
            id,
            name,
            position: Vector3::ZERO,
            rotation: Vector3::ZERO,
            scale: Vector3::ONE,
            active: true,
        }
    }
    
    pub fn set_position(&mut self, position: Vector3) {
        self.position = position;
    }
    
    pub fn set_rotation(&mut self, rotation: Vector3) {
        self.rotation = rotation;
    }
    
    pub fn set_scale(&mut self, scale: Vector3) {
        self.scale = scale;
    }
}

pub struct Scene {
    pub objects: HashMap<u32, GameObject>,
    pub camera_position: Vector3,
    pub camera_target: Vector3,
    pub next_id: u32,
}

impl Scene {
    pub fn new() -> Self {
        let mut scene = Scene {
            objects: HashMap::new(),
            camera_position: Vector3::new(0.0, 5.0, 10.0),
            camera_target: Vector3::ZERO,
            next_id: 1,
        };
        
        let mut camera = GameObject::new(0, "Main Camera".to_string());
        camera.position = scene.camera_position;
        scene.objects.insert(0, camera);
        
        scene
    }
    
    pub fn add_object(&mut self, object: GameObject) -> u32 {
        let id = object.id;
        self.objects.insert(id, object);
        self.next_id = self.next_id.max(id + 1);
        id
    }
    
    pub fn create_object(&mut self, name: String) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let object = GameObject::new(id, name);
        self.objects.insert(id, object);
        id
    }
    
    pub fn get_object(&self, id: u32) -> Option<&GameObject> {
        self.objects.get(&id)
    }
    
    pub fn get_object_mut(&mut self, id: u32) -> Option<&mut GameObject> {
        self.objects.get_mut(&id)
    }
    
    pub fn remove_object(&mut self, id: u32) -> Option<GameObject> {
        self.objects.remove(&id)
    }
    
    pub fn set_camera(&mut self, position: Vector3, target: Vector3) {
        self.camera_position = position;
        self.camera_target = target;
        if let Some(camera) = self.objects.get_mut(&0) {
            camera.position = position;
        }
    }
    
    pub fn update(&mut self, _delta_time: f32) {
        // Scene update logic (animations, physics, etc.)
    }
    
    pub fn list_objects(&self) -> Vec<(u32, String)> {
        self.objects
            .iter()
            .map(|(id, obj)| (*id, obj.name.clone()))
            .collect()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
