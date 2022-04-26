mod controls {
    use iced_wgpu::Renderer;
    use iced_winit::{
        widget::{Column, Container, Row, Text},
        Color, Program,
    };

    pub struct Controls {
        background_color: Color,
        text: String,
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        TextChanged(String),
    }

    impl Controls {
        pub fn new() -> Self {
            Self {
                background_color: Color {
                    r: 0.4,
                    g: 0.4,
                    b: 0.4,
                    a: 1.0,
                },
                text: String::from("Hey there"),
            }
        }

        pub fn background_color(&self) -> Color {
            self.background_color
        }
    }

    impl Program for Controls {
        type Renderer = Renderer;

        type Message = Message;

        fn update(&mut self, message: Self::Message) -> iced_winit::Command<Self::Message> {
            match message {
                Message::TextChanged(val) => self.text = val,
            }
            iced_winit::Command::none()
        }

        fn view(&mut self) -> iced_winit::Element<'_, Self::Message, Self::Renderer> {
            let content = Row::new()
                .width(iced_winit::Length::Fill)
                .height(iced_winit::Length::Fill)
                .push(Column::new().width(iced_winit::Length::Fill).push(
                    Text::new(&self.text).color(Color::new(
                        0.949_019_6,
                        0.898_039_2,
                        0.737_254_9,
                        1.0,
                    )),
                ));
            Container::new(content).into()
        }
    }
}
mod scene {
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
