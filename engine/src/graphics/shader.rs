pub struct ShaderModule {
    pub vertex_shader: wgpu::ShaderModule,
    pub fragment_shader: wgpu::ShaderModule,
}

impl ShaderModule {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(VERTEX_SHADER)),
        });
        
        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(FRAGMENT_SHADER)),
        });
        
        ShaderModule {
            vertex_shader,
            fragment_shader,
        }
    }
}

const VERTEX_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 1.0);
    output.color = input.color;
    output.normal = input.normal;
    return output;
}
"#;

const FRAGMENT_SHADER: &str = r#"
struct FragmentInput {
    @location(0) color: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

@fragment
fn main(input: FragmentInput) -> @location(0) vec4<f32> {
    let light_direction = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(input.normal, light_direction), 0.2);
    return vec4<f32>(input.color * diffuse, 1.0);
}
"#;
