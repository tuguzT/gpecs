use gpecs::{prelude::*, soa::identity::Identity};

use crate::common::{Mass, Position};

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
#[cfg_attr(miri, ignore)]
fn register_data() {
    init();

    let instance_desc = wgpu::InstanceDescriptor::default();
    let instance = wgpu::Instance::new(&instance_desc);

    let adapter_options = wgpu::RequestAdapterOptions::default();
    let adapter = pollster::block_on(instance.request_adapter(&adapter_options))
        .expect("failed to create WGPU adapter");
    println!("Running on {:#?}", adapter.get_info());

    let downlevel_capabilities = adapter.get_downlevel_capabilities();
    if !downlevel_capabilities
        .flags
        .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    {
        panic!("adapter does not support compute shaders");
    }

    let device_desc = wgpu::DeviceDescriptor {
        label: Some("gpecs test device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults(),
        memory_hints: wgpu::MemoryHints::MemoryUsage,
    };
    let (device, _queue) = pollster::block_on(adapter.request_device(&device_desc, None))
        .expect("failed to create device & queue");
    println!("Limits of the current device are {:#?}", device.limits());

    let mut context = Context::new();
    let mut executor = GpuExecutor::new(&mut context);

    let component_id = executor.register_component::<Position>();
    assert_eq!(component_id.into_inner(), 0);

    let archetype_id = executor
        .register_archetype::<Identity<Mass>>()
        .expect("archetype of just `Mass` should contain unique component ids");
    assert_eq!(archetype_id.into_inner(), 0);

    let component_id = executor
        .component_id::<Mass>()
        .expect("`Mass` component should be registered after registering archetype");
    assert_eq!(component_id.into_inner(), 1);

    executor.execute();
}
