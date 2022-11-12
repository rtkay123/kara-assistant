mod audio;
mod controls;
mod scene;
mod vertex;

use std::sync::{Arc, Mutex};

use iced_wgpu::{wgpu, Backend, Color, Renderer, Settings, Viewport};
use iced_winit::{
    conversion, futures, program, renderer,
    winit::{
        dpi::PhysicalPosition,
        event::*,
        event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
        window::{Window, WindowBuilder},
    },
    Clipboard, Debug, Size,
};
use tracing::trace;

use crate::{
    config::{read_config_file, Visualiser},
    events::KaraEvent,
    graphics::controls::map_colour,
};

use self::{controls::Controls, scene::Scene};

pub async fn run() -> anyhow::Result<()> {
    let event_loop: EventLoop<KaraEvent> = EventLoopBuilder::with_user_event().build();

    let event_loop_proxy = Arc::new(Mutex::new(event_loop.create_proxy()));

    let config_file = read_config_file(Arc::clone(&event_loop_proxy));

    let window_settings = &config_file.window;

    let window = WindowBuilder::new()
        .with_title(&window_settings.title)
        .with_transparent(true)
        .with_decorations(window_settings.decorations)
        .build(&event_loop)?;

    let controls = Controls::new(&config_file);

    let config_file = Arc::new(Mutex::new(config_file));

    let device_name: Option<&str> = None;
    let (stream_opts, _stream) = mic_rec::StreamOpts::new(device_name).unwrap();

    _stream.start_stream()?;

    let (tx, rx) = crossbeam_channel::unbounded();
    audio::visualise(
        monitor_refresh_rate(&window),
        Arc::clone(&config_file),
        rx,
        tx.clone(),
    );
    let inner_audio = tx.clone();
    tokio::task::spawn_blocking(move || {
        while let Ok(audio_buf) = stream_opts.audio_feed().recv() {
            let _transciption_data =
                audio_utils::resample_i16_mono(&audio_buf, stream_opts.channel_count());
            inner_audio.send(audio::Event::SendData(audio_buf)).unwrap();
        }
    });

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

    // Initialize iced
    let mut debug = Debug::new();
    let mut renderer = Renderer::new(Backend::new(&device, Settings::default(), format));

    let mut state =
        program::State::new(controls, viewport.logical_size(), &mut renderer, &mut debug);

    event_loop.run(move |event, _, control_flow| {
        match event {
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
                        let mut encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        let program = state.program();

                        let (ntx, nrx) = crossbeam_channel::unbounded();
                        tx.send(audio::Event::RequestData(ntx)).unwrap();

                        let mut buffer = nrx.recv().unwrap();

                        for i in 0..buffer.len() {
                            buffer.insert(0, buffer[i * 2]);
                        }

                        let conf = config_file.lock().unwrap();
                        let vis = match &conf.audio {
                            Some(audio) => audio.visualiser.clone(),
                            None => Visualiser::default(),
                        };
                        drop(conf);
                        let (tr, tg, tb) =
                            map_colour(&vis.top_colour, controls::ColourType::Foreground);
                        let (br, bg, bb) =
                            map_colour(&vis.bottom_colour, controls::ColourType::Foreground);

                        let (top_color, bottom_color) = ([tr, tg, tb], [br, bg, bb]);

                        let (vertices, indices) = vertex::prepare_data(
                            buffer,
                            1.5,
                            top_color,
                            bottom_color,
                            [
                                window.inner_size().width as f32 * 0.001,
                                window.inner_size().height as f32 * 0.001,
                            ],
                            vis.radius,
                        );

                        scene.update_buffers(&device, vertices, indices);

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        {
                            // We clear the frame
                            let mut render_pass =
                                scene.clear(&view, &mut encoder, program.background_colour());

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
                window.request_redraw();
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
            Event::UserEvent(event) => {
                if let KaraEvent::ReloadConfiguration(new_config) = &event {
                    let mut config = config_file.lock().unwrap();
                    *config = new_config.clone();
                    window.set_title(&new_config.window.title);
                    window.set_decorations(new_config.window.decorations);
                    window.request_redraw();
                }
                state.queue_message(event);
            }
            _ => {}
        }
    });
}

fn monitor_refresh_rate(window: &Window) -> u16 {
    let mut monitor: Vec<_> = window
        .available_monitors()
        .into_iter()
        .filter_map(|f| f.refresh_rate_millihertz().map(|f| f / 1000))
        .collect();
    let refresh_rate = if monitor.is_empty() {
        60
    } else {
        monitor.sort();
        monitor[0]
    };
    trace!(refresh_rate = refresh_rate);
    refresh_rate as u16
}
