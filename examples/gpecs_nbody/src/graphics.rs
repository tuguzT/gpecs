use std::error::Error;

use wgpu::{
    Adapter, CurrentSurfaceTexture, Device, DeviceDescriptor, ExperimentalFeatures, Features,
    Instance, InstanceDescriptor, MemoryHints, PowerPreference, Queue, RequestAdapterOptions,
    Surface, SurfaceConfiguration, SurfaceTarget, TextureView, TextureViewDescriptor, Trace,
};

#[derive(Debug)]
pub struct Graphics<'window> {
    #[expect(dead_code)]
    instance: Instance,
    #[expect(dead_code)]
    adapter: Adapter,
    surface: Surface<'window>,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
}

impl<'window> Graphics<'window> {
    pub async fn new(
        surface_target: impl Into<SurfaceTarget<'window>>,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let instance_desc = InstanceDescriptor::new_without_display_handle();
        let instance = Instance::new(instance_desc);

        let width = u32::max(width, 1);
        let height = u32::max(height, 1);
        let surface = instance.create_surface(surface_target)?;

        let adapter_options = RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            ..Default::default()
        };
        let adapter = instance.request_adapter(&adapter_options).await?;

        let device_desc = DeviceDescriptor {
            label: Some("`gpecs` n-body simulation example GPU device"),
            required_features: Features::empty(),
            required_limits: adapter.limits(),
            experimental_features: ExperimentalFeatures::disabled(),
            memory_hints: MemoryHints::Manual {
                suballocated_device_memory_block_size: 0..0,
            },
            trace: Trace::Off,
        };
        let (device, queue) = adapter.request_device(&device_desc).await?;

        let surface_config = surface
            .get_default_config(&adapter, width, height)
            .expect("surface should be supported by this adapter");
        surface.configure(&device, &surface_config);

        let me = Self {
            instance,
            adapter,
            surface,
            surface_config,
            device,
            queue,
        };
        Ok(me)
    }

    pub fn device(&self) -> &Device {
        let Self { device, .. } = self;
        device
    }

    pub fn surface_config(&self) -> &SurfaceConfiguration {
        let Self { surface_config, .. } = self;
        surface_config
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        let Self {
            ref device,
            ref surface,
            ref mut surface_config,
            ..
        } = *self;

        surface_config.width = new_width.max(1);
        surface_config.height = new_height.max(1);
        surface.configure(device, surface_config);
    }

    pub fn draw(&self, f: impl FnOnce(&Device, &Queue, &TextureView)) {
        let Self {
            surface,
            surface_config,
            device,
            queue,
            ..
        } = self;

        let frame = match surface.get_current_texture() {
            CurrentSurfaceTexture::Success(frame) => frame,
            CurrentSurfaceTexture::Outdated | CurrentSurfaceTexture::Suboptimal(_) => {
                surface.configure(device, surface_config);
                return;
            }
            CurrentSurfaceTexture::Timeout | CurrentSurfaceTexture::Occluded => return,
            CurrentSurfaceTexture::Lost | CurrentSurfaceTexture::Validation => todo!(),
        };

        let view_desc = TextureViewDescriptor {
            label: Some("`gpecs` n-body simulation example surface texture view"),
            ..Default::default()
        };
        let view = frame.texture.create_view(&view_desc);

        f(device, queue, &view);
        frame.present();
    }
}
