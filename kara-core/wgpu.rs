use iced_wgpu::{
    wgpu::{
        util::{backend_bits_from_env, initialize_adapter_from_env_or_default, StagingBelt},
        Backends, CommandEncoderDescriptor, DeviceDescriptor, Features, Instance, Limits,
        PresentMode, SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor,
    },
    Backend, Renderer, Settings, Viewport,
};
use iced_winit::{
    conversion,
    futures::{
        executor::{self, LocalPool},
        task::SpawnExt,
    },
    program,
    winit::{
        dpi::PhysicalPosition,
        event::{Event, ModifiersState, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::Window,
    },
    Clipboard, Debug, Size,
};

use self::{controls::Controls, scene::Scene};

pub fn start() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop)?;
    let physical_size = window.inner_size();
    let mut viewport = Viewport::with_physical_size(
        iced_winit::Size {
            width: physical_size.width,
            height: physical_size.height,
        },
        window.scale_factor(),
    );
    let mut cursor_position = PhysicalPosition::new(-1.0, -1.0);
    let mut modifiers = ModifiersState::default();
    let mut clipboard = Clipboard::connect(&window);

    // Initialise wgpu
    let default_backend = Backends::PRIMARY;
    let backend = backend_bits_from_env().unwrap_or(default_backend);
    let instance = Instance::new(backend);
    let surface = unsafe { instance.create_surface(&window) };
    let (format, (device, queue)) = executor::block_on(async {
        let adapter = initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
            .await
            .expect("No suitable GPU adapters found in the system");
        let adapter_features = adapter.features();

        let needed_limits = Limits::default();
        (
            surface
                .get_preferred_format(&adapter)
                .expect("get preferred format"),
            adapter
                .request_device(
                    &DeviceDescriptor {
                        label: None,
                        features: adapter_features & Features::default(),
                        limits: needed_limits,
                    },
                    None,
                )
                .await
                .expect("request device"),
        )
    });
    surface.configure(
        &device,
        &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: PresentMode::Mailbox,
        },
    );

    let mut resized = false;

    // Initialise staging belt and local pool
    let mut staging_belt = StagingBelt::new(5 * 1024);
    let mut local_pool = LocalPool::new();

    // Initialise scene
    let scene = Scene::new(&device, format);
    let controls = Controls::new();

    // Initialise iced
    let mut debug = Debug::new();
    let mut renderer = Renderer::new(Backend::new(&device, Settings::default(), format));

    let mut state =
        program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);

    // Run event_loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        cursor_position = position;
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers;
                    }
                    WindowEvent::Resized(new_size) => {
                        viewport = Viewport::with_physical_size(
                            Size::new(new_size.width, new_size.height),
                            window.scale_factor(),
                        );

                        resized = true;
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }

                // Map window event to iced event
                if let Some(event) =
                    iced_winit::conversion::window_event(&event, window.scale_factor(), modifiers)
                {
                    state.queue_event(event);
                }
            }
            Event::MainEventsCleared => {
                // If there are events pending
                if !state.is_queue_empty() {
                    // We update iced
                    let _ = state.update(
                        viewport.logical_size(),
                        conversion::cursor_position(cursor_position, viewport.scale_factor()),
                        &mut renderer,
                        &mut clipboard,
                        &mut debug,
                    );

                    // and request a redraw
                    window.request_redraw();
                }
            }
            Event::RedrawRequested(_) => {
                if resized {
                    let size = window.inner_size();

                    surface.configure(
                        &device,
                        &SurfaceConfiguration {
                            usage: TextureUsages::RENDER_ATTACHMENT,
                            format,
                            width: size.width,
                            height: size.height,
                            present_mode: PresentMode::Mailbox,
                        },
                    );

                    resized = false;
                }

                match surface.get_current_texture() {
                    Ok(frame) => {
                        let mut encoder = device
                            .create_command_encoder(&CommandEncoderDescriptor { label: None });

                        let program = state.program();

                        let view = frame.texture.create_view(&TextureViewDescriptor::default());

                        {
                            // We clear the frame
                            let mut render_pass =
                                scene.clear(&view, &mut encoder, program.background_color());

                            // Draw the scene
                            scene.draw(&mut render_pass);
                        }

                        // And then iced on top
                        renderer.with_primitives(|backend, primitive| {
                            backend.present(
                                &device,
                                &mut staging_belt,
                                &mut encoder,
                                &view,
                                primitive,
                                &viewport,
                                &debug.overlay(),
                            );
                        });

                        // Then we submit the work
                        staging_belt.finish();
                        queue.submit(Some(encoder.finish()));
                        frame.present();

                        // Update the mouse cursor
                        window.set_cursor_icon(iced_winit::conversion::mouse_interaction(
                            state.mouse_interaction(),
                        ));

                        // And recall staging buffers
                        local_pool
                            .spawner()
                            .spawn(staging_belt.recall())
                            .expect("Recall staging buffers");

                        local_pool.run_until_stalled();
                    }
                    Err(error) => match error {
                        SurfaceError::OutOfMemory => {
                            panic!("Swapchain error: {}. Rendering cannot continue.", error)
                        }
                        _ => {
                            // Try rendering again next frame.
                            window.request_redraw();
                        }
                    },
                }
            }
            _ => {}
        }
    });
}
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
    use iced_wgpu::{wgpu, Color};

    pub struct Scene {
        pipeline: wgpu::RenderPipeline,
    }

    impl Scene {
        pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
            Self {
                pipeline: build_pipeline(device, texture_format),
            }
        }
        pub fn clear<'a>(
            &self,
            target: &'a wgpu::TextureView,
            encoder: &'a mut wgpu::CommandEncoder,
            background_color: Color,
        ) -> wgpu::RenderPass<'a> {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear({
                            let [r, g, b, a] = background_color.into_linear();

                            wgpu::Color {
                                r: r as f64,
                                g: g as f64,
                                b: b as f64,
                                a: a as f64,
                            }
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            })
        }

        pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..3, 0..1);
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
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fragment_main",
                targets: &[texture_format.into()],
            }),
            multiview: None,
        })
    }
}
