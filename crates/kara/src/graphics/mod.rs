mod controls;
mod scene;
mod vertex;

use audio_utils::fft::spectrum;
use iced_wgpu::{wgpu, Backend, Color, Renderer, Settings, Viewport};
use iced_winit::{
    conversion, futures, program, renderer,
    winit::{
        dpi::PhysicalPosition,
        event::*,
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    },
    Clipboard, Debug, Size,
};
use tracing::trace;

use self::{controls::Controls, scene::Scene};

pub async fn run() -> anyhow::Result<()> {
    let device_name: Option<&str> = None;
    let (stream_opts, _stream) = mic_rec::StreamOpts::new(device_name).unwrap();
    let audio_feed = stream_opts.audio_feed().clone();
    let sample_rate = stream_opts.sample_rate();

    _stream.start_stream()?;
    tokio::task::spawn_blocking(move || {
        while let Ok(audio_buf) = stream_opts.audio_feed().recv() {
            let _transciption_data =
                audio_utils::resample_i16_mono(&audio_buf, stream_opts.channel_count());
        }
    });

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    let _refresh_rate_millis = (1.0 / monitor_refresh_rate(&window) * 1000.0) as u64;

    let physical_size = window.inner_size();

    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );
    let mut cursor_position = PhysicalPosition::new(-1.0, -1.0);
    let mut modifiers = ModifiersState::default();
    let mut clipboard = Clipboard::connect(&window);

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    trace!("created instance");

    let surface = unsafe { instance.create_surface(&window) };

    let (format, (device, queue)) = futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("no suitable GPU adapters");
        trace!("located suitable adapter");

        let adapter_features = adapter.features();

        let needed_limits = wgpu::Limits::default();

        (
            surface.get_supported_formats(&adapter)[0],
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        features: adapter_features & wgpu::Features::default(),
                        limits: needed_limits,
                    },
                    None,
                )
                .await
                .expect("Request device"),
        )
    });

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: physical_size.width,
        height: physical_size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
    };

    surface.configure(&device, &config);

    let mut resized = false;

    let mut staging_belt = wgpu::util::StagingBelt::new(5 * 1024);

    // Initialize scene and GUI controls
    let mut scene = Scene::new(&device, format);
    let controls = Controls::new();

    // Initialize iced
    let mut debug = Debug::new();
    let mut renderer = Renderer::new(Backend::new(&device, Settings::default(), format));

    let mut state =
        program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            if resized {
                let size = window.inner_size();

                viewport = Viewport::with_physical_size(
                    Size::new(size.width, size.height),
                    window.scale_factor(),
                );

                surface.configure(
                    &device,
                    &wgpu::SurfaceConfiguration {
                        format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        width: size.width,
                        height: size.height,
                        present_mode: wgpu::PresentMode::AutoVsync,
                        alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    },
                );

                resized = false;
            }

            match surface.get_current_texture() {
                Ok(frame) => {
                    let mut encoder = device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    let program = state.program();

                    let audio_buf = audio_feed.recv().unwrap_or_else(|_| vec![0.0; 1024]);

                    // A vec containing a tuple of frequencies (0) respective amplitude(1)
                    let data = spectrum(&audio_buf, sample_rate as u16);

                    let (vertices, indices) = vertex::prepare_data(data);
                    scene.update_buffers(&device, vertices, indices);

                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

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
                    staging_belt.recall();
                }
                Err(error) => match error {
                    wgpu::SurfaceError::OutOfMemory => {
                        panic!("Swapchain error: {}. Rendering cannot continue.", error)
                    }
                    _ => {
                        // Try rendering again next frame.
                        window.request_redraw();
                    }
                },
            }
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,

                WindowEvent::CursorMoved { position, .. } => {
                    cursor_position = *position;
                }
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    modifiers = *new_modifiers;
                }
                WindowEvent::Resized(_) => {
                    resized = true;
                }
                _ => {}
            }

            // Map window event to iced event
            if let Some(event) =
                iced_winit::conversion::window_event(event, window.scale_factor(), modifiers)
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
                    &iced_wgpu::Theme::Dark,
                    &renderer::Style {
                        text_color: Color::WHITE,
                    },
                    &mut clipboard,
                    &mut debug,
                );

                // and request a redraw
                window.request_redraw();
            }
        }
        _ => {}
    });
}

fn monitor_refresh_rate(window: &Window) -> f32 {
    let mut monitor: Vec<_> = window
        .available_monitors()
        .into_iter()
        .filter_map(|f| f.refresh_rate_millihertz().map(|f| f / 1000))
        .collect();
    let refresh_rate = if monitor.is_empty() {
        60000_f32
    } else {
        monitor.sort();
        monitor[0] as f32
    };
    trace!(refresh_rate = refresh_rate);
    refresh_rate
}
