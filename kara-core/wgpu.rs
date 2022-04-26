pub mod scene {
    use iced_wgpu::wgpu;

    pub struct Scene {
        pipeline: wgpu::RenderPipeline,
    }

    impl Scene {
        pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
            Self {
                pipeline: build_pipeline(device, texture_format),
            }
        }
    }

    fn build_pipeline(
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader_module =
            device.create_shader_module(&wgpu::include_wgsl!("../kara-assets/shader.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vertex_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: 0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fragment_main",
                targets: &[texture_format.into()],
            }),
            multiview: None,
        })
    }
}
