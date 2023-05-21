#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

pub const SQUARE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1., 1., 0.],
        tex_coords: [0., 1.],
    },
    Vertex {
        position: [-1., -1., 0.],
        tex_coords: [0., 0.],
    },
    Vertex {
        position: [1., 1., 0.],
        tex_coords: [1., 1.],
    },
    Vertex {
        position: [1., 1., 0.],
        tex_coords: [1., 1.],
    },
    Vertex {
        position: [-1., -1., 0.],
        tex_coords: [0., 0.],
    },
    Vertex {
        position: [1., -1., 0.],
        tex_coords: [1., 0.],
    },
];

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
