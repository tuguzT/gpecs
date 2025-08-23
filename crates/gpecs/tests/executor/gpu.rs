use gpecs::prelude::*;

#[test]
#[cfg_attr(miri, ignore)]
fn execute_simple() {
    let mut context = Context::new();

    let (device, _queue) = init_wgpu();
    let _executor = GpuExecutor::new(&mut context, device);

    // TODO: find a way to safely map data from GPU to CPU
}

fn init_wgpu() -> (wgpu::Device, wgpu::Queue) {
    let instance_desc = wgpu::InstanceDescriptor::default();
    let instance = wgpu::Instance::new(&instance_desc);

    let adapter_options = wgpu::RequestAdapterOptions::default();
    let adapter = pollster::block_on(instance.request_adapter(&adapter_options))
        .expect("failed to create adapter");

    let device_desc = wgpu::DeviceDescriptor {
        label: Some("`gpecs` test device"),
        ..Default::default()
    };
    pollster::block_on(adapter.request_device(&device_desc))
        .expect("failed to create device & queue")
}
