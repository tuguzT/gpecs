use std::{error::Error, mem, sync::Arc, time::Instant};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

use crate::{
    graphics::Graphics,
    state::State,
    ui::{UiRenderer, UiState},
};

pub struct App {
    state: AppState,
}

enum AppState {
    Init {
        event_loop_proxy: EventLoopProxy<Graphics<'static>>,
    },
    ForReplace,
    Ready {
        window: Arc<Window>,
        ui: Box<UiState>,
        graphics: GraphicsState,
    },
}

enum GraphicsState {
    WaitForEvent,
    Ready {
        state: Box<State>,
        graphics: Box<Graphics<'static>>,
        ui_renderer: Box<UiRenderer>,
    },
}

impl App {
    pub fn new(event_loop: &EventLoop<Graphics>) -> Self {
        let event_loop_proxy = event_loop.create_proxy();
        let state = AppState::Init { event_loop_proxy };
        Self { state }
    }

    fn handle_window_event_ui(&mut self, event: &WindowEvent) {
        let Self { state } = self;

        let AppState::Ready {
            ref window,
            ref mut ui,
            ..
        } = *state
        else {
            return;
        };
        let _ = ui.handle_input(window, event);
    }

    fn handle_window_event_state(&mut self, event: &WindowEvent) {
        let Self { state } = self;

        let AppState::Ready {
            ref window,
            ref mut graphics,
            ..
        } = *state
        else {
            return;
        };
        let GraphicsState::Ready { state, .. } = graphics else {
            return;
        };

        state.handle_window_event(window, event);
    }

    fn handle_device_event_state(&mut self, event: &DeviceEvent) {
        let Self { state } = self;

        let AppState::Ready { graphics, .. } = state else {
            return;
        };
        let GraphicsState::Ready { state, .. } = graphics else {
            return;
        };

        state.handle_device_event(event);
    }

    fn resized(&mut self, new_size: PhysicalSize<u32>) {
        let Self { state } = self;

        let AppState::Ready { graphics, .. } = state else {
            return;
        };
        let GraphicsState::Ready { graphics, .. } = graphics else {
            return;
        };

        let PhysicalSize { width, height } = new_size;
        graphics.resize(width, height);
    }

    fn redraw_requested(&mut self) {
        let Self { state } = self;

        let AppState::Ready {
            ref window,
            ref mut ui,
            ref mut graphics,
        } = *state
        else {
            return;
        };
        let GraphicsState::Ready {
            ref graphics,
            ref mut state,
            ref mut ui_renderer,
        } = *graphics
        else {
            return;
        };

        let ui_output = ui.run(window, |ui| state.update(ui));

        graphics.draw(|device, queue, render_target| {
            let command_encoder_desc = wgpu::CommandEncoderDescriptor {
                label: Some("`gpecs` n-body simulation example command encoder"),
            };
            let mut encoder = device.create_command_encoder(&command_encoder_desc);

            state.draw(device, queue, render_target, &mut encoder);

            let surface_config = graphics.surface_config();
            ui_renderer.draw(
                ui.context(),
                ui_output,
                surface_config.width,
                surface_config.height,
                device,
                queue,
                &mut encoder,
                render_target,
            );

            let command_buffer = encoder.finish();
            queue.submit([command_buffer]);
        });
    }

    fn request_redraw(&self) {
        let Self { state } = self;

        let AppState::Ready { window, .. } = state else {
            return;
        };
        window.request_redraw();
    }
}

impl ApplicationHandler<Graphics<'static>> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Self { state } = self;

        match mem::replace(state, AppState::ForReplace) {
            AppState::Init { event_loop_proxy } => {
                let window_attributes = Window::default_attributes()
                    .with_title("`gpecs` n-body simulation example")
                    .with_visible(false);
                let window = event_loop.create_window(window_attributes).unwrap();

                let window = Arc::new(window);
                let surface_target = Arc::clone(&window);
                let ui = UiState::new(&window);
                *state = AppState::Ready {
                    window,
                    ui: Box::new(ui),
                    graphics: GraphicsState::WaitForEvent,
                };

                let future = async move || -> Result<(), Box<dyn Error>> {
                    let PhysicalSize { width, height } = surface_target.inner_size();
                    let graphics = Graphics::new(surface_target, width, height).await?;
                    event_loop_proxy.send_event(graphics)?;
                    Ok(())
                };
                pollster::block_on(future()).unwrap();
            }
            old_state => *state = old_state,
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, graphics: Graphics<'static>) {
        let Self { state } = self;

        let AppState::Ready {
            ref window,
            graphics: ref mut state,
            ..
        } = *state
        else {
            return;
        };

        let device = graphics.device();
        let format = graphics.surface_config().format;

        let start_time = Instant::now();
        let PhysicalSize { width, height } = window.inner_size();
        let other_state = State::new(device, width, height, format, start_time);

        let ui_renderer = UiRenderer::new(device, format);
        *state = GraphicsState::Ready {
            state: Box::new(other_state),
            graphics: Box::new(graphics),
            ui_renderer: Box::new(ui_renderer),
        };

        window.set_visible(true);
        window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        self.handle_window_event_ui(&event);
        self.handle_window_event_state(&event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.resized(size),
            WindowEvent::RedrawRequested => self.redraw_requested(),
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.handle_device_event_state(&event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.request_redraw();
    }
}
