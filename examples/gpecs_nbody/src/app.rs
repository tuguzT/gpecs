use std::{error::Error, mem, sync::Arc, time::Instant};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
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
        state: State,
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

    fn handle_ui_input(&mut self, event: &WindowEvent) {
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

        graphics.draw(|device, queue, view| {
            let command_encoder_desc = wgpu::CommandEncoderDescriptor {
                label: Some("`gpecs` n-body simulation example command encoder"),
            };
            let mut encoder = device.create_command_encoder(&command_encoder_desc);

            state.draw(device, queue, view, &mut encoder);

            let surface_config = graphics.surface_config();
            ui_renderer.draw(
                ui.context(),
                ui_output,
                surface_config.width,
                surface_config.height,
                device,
                queue,
                &mut encoder,
                view,
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
            window,
            graphics: state,
            ..
        } = state
        else {
            return;
        };

        let start_time = Instant::now();
        let ui_renderer = UiRenderer::new(graphics.device(), graphics.surface_config().format);
        *state = GraphicsState::Ready {
            state: State::new(start_time),
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
        self.handle_ui_input(&event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.resized(size),
            WindowEvent::RedrawRequested => self.redraw_requested(),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.request_redraw();
    }
}
