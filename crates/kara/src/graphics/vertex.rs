use iced_wgpu::wgpu;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    position: [f32; 3],
    colour: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub(crate) fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub(crate) const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.5, 0.5, 0.0],
        colour: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.0],
        colour: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        colour: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        colour: [0.0, 1.0, 0.0],
    },
];

pub(crate) const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];
