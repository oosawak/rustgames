use super::Vertex;
use wgpu::util::DeviceExt;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

pub struct MeshData {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Mesh { vertices, indices }
    }
    
    pub fn to_mesh_data(self, device: &wgpu::Device) -> MeshData {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        let index_count = self.indices.len() as u32;
        
        MeshData {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
}

pub fn create_cube() -> Mesh {
    let vertices = vec![
        // Front face
        Vertex::new([-0.5, -0.5,  0.5], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        Vertex::new([ 0.5, -0.5,  0.5], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        Vertex::new([ 0.5,  0.5,  0.5], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        Vertex::new([-0.5,  0.5,  0.5], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        
        // Back face
        Vertex::new([-0.5, -0.5, -0.5], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]),
        Vertex::new([-0.5,  0.5, -0.5], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]),
        Vertex::new([ 0.5,  0.5, -0.5], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]),
        Vertex::new([ 0.5, -0.5, -0.5], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]),
        
        // Top face
        Vertex::new([-0.5,  0.5, -0.5], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
        Vertex::new([-0.5,  0.5,  0.5], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
        Vertex::new([ 0.5,  0.5,  0.5], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
        Vertex::new([ 0.5,  0.5, -0.5], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
        
        // Bottom face
        Vertex::new([-0.5, -0.5, -0.5], [1.0, 1.0, 0.0], [0.0, -1.0, 0.0]),
        Vertex::new([ 0.5, -0.5, -0.5], [1.0, 1.0, 0.0], [0.0, -1.0, 0.0]),
        Vertex::new([ 0.5, -0.5,  0.5], [1.0, 1.0, 0.0], [0.0, -1.0, 0.0]),
        Vertex::new([-0.5, -0.5,  0.5], [1.0, 1.0, 0.0], [0.0, -1.0, 0.0]),
        
        // Right face
        Vertex::new([ 0.5, -0.5, -0.5], [1.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
        Vertex::new([ 0.5,  0.5, -0.5], [1.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
        Vertex::new([ 0.5,  0.5,  0.5], [1.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
        Vertex::new([ 0.5, -0.5,  0.5], [1.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
        
        // Left face
        Vertex::new([-0.5, -0.5, -0.5], [0.0, 1.0, 1.0], [-1.0, 0.0, 0.0]),
        Vertex::new([-0.5, -0.5,  0.5], [0.0, 1.0, 1.0], [-1.0, 0.0, 0.0]),
        Vertex::new([-0.5,  0.5,  0.5], [0.0, 1.0, 1.0], [-1.0, 0.0, 0.0]),
        Vertex::new([-0.5,  0.5, -0.5], [0.0, 1.0, 1.0], [-1.0, 0.0, 0.0]),
    ];
    
    let indices = vec![
        // Front
        0, 1, 2, 0, 2, 3,
        // Back
        4, 6, 5, 4, 7, 6,
        // Top
        8, 9, 10, 8, 10, 11,
        // Bottom
        12, 14, 13, 12, 15, 14,
        // Right
        16, 18, 17, 16, 19, 18,
        // Left
        20, 22, 21, 20, 23, 22,
    ];
    
    Mesh::new(vertices, indices)
}
