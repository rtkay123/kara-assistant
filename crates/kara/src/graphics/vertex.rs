use std::f32::consts::PI;

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

#[rustfmt::skip]
pub(crate) const INDICES: &[u16] = &[
    0, 1, 2,
    0, 2, 3
];

pub(crate) fn prepare_data(
    buffer: Vec<f32>,
    width: f32,
    top_color: [f32; 3],
    bottom_color: [f32; 3],
    size: [f32; 2],
    radius: f32,
) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    if buffer.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let width = width * 0.005;
    let mut last_x: f32 = 0.0;
    let mut last_y: f32 = 0.0;

    for i in 0..buffer.len() - 1 {
        let mut angle: f32 = 2.0 * PI * (i + 1) as f32 / (buffer.len() - 2) as f32;
        let degree: f32 = 2.0 * PI / 360.0;
        angle += degree * 270.0; // rotate circle 270°

        let value: f32 = buffer[i];

        let x: f32 = angle.cos() * (value + radius) / size[0];
        let y: f32 = angle.sin() * (value + radius) / size[1];

        let r: f32 = (top_color[0] * value) + (bottom_color[0] * (1.0 / value));
        let g: f32 = (top_color[1] * value) + (bottom_color[1] * (1.0 / value));
        let b: f32 = (top_color[2] * value) + (bottom_color[2] * (1.0 / value));

        let color: [f32; 3] = [r, g, b];

        if i != 0 {
            let (mut vertices2, mut indices2) = draw_line(
                [last_x, last_y],
                [x, y],
                width,
                color,
                vertices.len() as u16,
                size,
            );
            vertices.append(&mut vertices2);
            indices.append(&mut indices2);
        }
        last_x = x;
        last_y = y;
    }
    (vertices, indices)
}

fn draw_line(
    point1: [f32; 2],
    point2: [f32; 2],
    width: f32,
    colour: [f32; 3],
    vertex_len: u16,
    size: [f32; 2],
) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    let x1: f32 = point1[0];
    let x2: f32 = point2[0];
    let y1: f32 = point1[1];
    let y2: f32 = point2[1];

    let dx = x2 - x1;
    let dy = y2 - y1;
    let l = dx.hypot(dy);
    let u = dx * width * 0.5 / l / size[1];
    let v = dy * width * 0.5 / l / size[0];

    vertices.push(Vertex {
        position: [x1 + v, y1 - u, 0.0],
        colour,
    });
    vertices.push(Vertex {
        position: [x1 - v, y1 + u, 0.0],
        colour,
    });
    vertices.push(Vertex {
        position: [x2 - v, y2 + u, 0.0],
        colour,
    });
    vertices.push(Vertex {
        position: [x2 + v, y2 - u, 0.0],
        colour,
    });

    indices.push(vertex_len + 2);
    indices.push(vertex_len + 1);
    indices.push(vertex_len);
    indices.push(vertex_len + 2);
    indices.push(vertex_len);
    indices.push(vertex_len + 3);

    (vertices, indices)
}
