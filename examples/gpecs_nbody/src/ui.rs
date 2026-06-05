use std::fmt::{self, Debug};

use egui::{
    Context, OrderedViewportIdMap, Ui, ViewportOutput,
    epaint::{ClippedShape, textures::TexturesDelta},
    viewport::ViewportId,
};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use egui_winit::{EventResponse, State};
use wgpu::{
    CommandEncoder, Device, LoadOp, Operations, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};
use winit::{event::WindowEvent, window::Window};

pub struct UiState {
    inner: State,
}

impl UiState {
    pub fn new(window: &Window) -> Self {
        #[expect(clippy::cast_possible_truncation)]
        let native_pixels_per_point = window.scale_factor() as f32;

        let inner = State::new(
            Context::default(),
            ViewportId::ROOT,
            &window,
            Some(native_pixels_per_point),
            None,
            None,
        );
        Self { inner }
    }

    pub fn context(&self) -> &Context {
        let Self { inner } = self;
        inner.egui_ctx()
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) -> EventResponse {
        let Self { inner } = self;
        inner.on_window_event(window, event)
    }

    pub fn run(&mut self, window: &Window, f: impl FnMut(&mut Ui)) -> RenderOutput {
        let Self { inner } = self;

        let input = inner.take_egui_input(window);
        let output = inner.egui_ctx().run_ui(input, f);
        inner.handle_platform_output(window, output.platform_output);

        RenderOutput {
            textures_delta: output.textures_delta,
            shapes: output.shapes,
            pixels_per_point: output.pixels_per_point,
            viewport_output: output.viewport_output,
        }
    }
}

impl Debug for UiState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UiState").finish_non_exhaustive()
    }
}

#[derive(Clone, Default)]
pub struct RenderOutput {
    pub textures_delta: TexturesDelta,
    pub shapes: Vec<ClippedShape>,
    pub pixels_per_point: f32,
    pub viewport_output: OrderedViewportIdMap<ViewportOutput>,
}

pub struct UiRenderer {
    inner: Renderer,
}

impl UiRenderer {
    pub fn new(device: &Device, output_color_format: TextureFormat) -> Self {
        let inner = Renderer::new(device, output_color_format, RendererOptions::PREDICTABLE);
        Self { inner }
    }

    #[expect(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        context: &Context,
        output: RenderOutput,
        width: u32,
        height: u32,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        render_target: &TextureView,
    ) {
        let Self { inner } = self;
        let RenderOutput {
            textures_delta,
            shapes,
            pixels_per_point,
            #[expect(unused)]
            viewport_output,
        } = output;

        let paint_jobs = context.tessellate(shapes, pixels_per_point);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point,
        };

        for (id, image_delta) in &textures_delta.set {
            inner.update_texture(device, queue, *id, image_delta);
        }
        inner.update_buffers(device, queue, encoder, &paint_jobs, &screen_descriptor);

        let render_pass_desc = RenderPassDescriptor {
            label: Some("`gpecs` n-body simulation example UI render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: render_target,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        };
        let render_pass = encoder.begin_render_pass(&render_pass_desc);

        {
            let mut render_pass = render_pass.forget_lifetime();
            inner.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        for id in &textures_delta.free {
            inner.free_texture(id);
        }
    }
}

impl Debug for UiRenderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UiRenderer").finish_non_exhaustive()
    }
}
