use std::time::{Duration, Instant};

use egui::{Rgba, RichText, Ui};
use wgpu::{CommandEncoder, Device, Queue, TextureView};

#[derive(Debug)]
#[expect(clippy::struct_field_names, reason = "soon this will change")]
pub struct State {
    start_time: Instant,
    last_update_time: Instant,
    total_time: Duration,
    delta_time: Duration,
}

impl State {
    pub fn new(start_time: Instant) -> Self {
        Self {
            start_time,
            last_update_time: start_time,
            total_time: Duration::ZERO,
            delta_time: Duration::ZERO,
        }
    }

    pub fn update(&mut self, ui: &mut Ui) {
        let Self {
            start_time,
            ref mut last_update_time,
            ref mut total_time,
            ref mut delta_time,
        } = *self;

        let now = Instant::now();
        *total_time = now.duration_since(start_time);
        *delta_time = now.duration_since(*last_update_time);
        *last_update_time = now;

        let dt_raw = delta_time.as_secs_f64();
        let fps = 1.0 / dt_raw;
        let text = format!("Total time: {total_time:?}, delta time: {dt_raw}, FPS: {fps}");
        let color = Rgba::from_rgb(0.0, 1.0 - total_time.as_secs_f32().fract(), 0.0);
        ui.label(RichText::new(text).color(color));
    }

    pub fn draw(
        &self,
        _device: &Device,
        _queue: &Queue,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        let Self { total_time, .. } = self;

        let clear_color = wgpu::Color {
            r: 0.0,
            g: total_time.as_secs_f64().fract(),
            b: 0.0,
            a: 1.0,
        };
        let render_pass_desc = wgpu::RenderPassDescriptor {
            label: Some("`gpecs` n-body simulation example clear render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        };
        let _ = encoder.begin_render_pass(&render_pass_desc);
    }
}
