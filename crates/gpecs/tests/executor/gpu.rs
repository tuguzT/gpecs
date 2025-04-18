use gpecs::{prelude::*, soa::identity::Identity};
use renderdoc::{RenderDoc, V141};

use crate::common::{Mass, Position};

fn init() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();
}

#[test]
#[cfg_attr(miri, ignore)]
fn register_data() {
    init();

    let instance_desc = wgpu::InstanceDescriptor {
        ..Default::default()
    };
    let instance = wgpu::Instance::new(&instance_desc);

    let adapter_options = wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    };
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
        label: Some("`gpecs` integration test device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::downlevel_defaults(),
        memory_hints: wgpu::MemoryHints::Performance,
    };
    let (device, _queue) = pollster::block_on(adapter.request_device(&device_desc, None))
        .expect("failed to create device & queue");
    println!("Limits of the current device are {:#?}", device.limits());

    let mut renderdoc = RenderDoc::<V141>::new();
    match renderdoc.as_mut() {
        Ok(renderdoc) => {
            log::info!("RenderDoc version: {:?}", renderdoc.get_api_version());
            log::info!("Starting RenderDoc capture...");
            renderdoc.start_frame_capture(std::ptr::null(), std::ptr::null());
        }
        Err(error) => {
            log::warn!("{error}");
        }
    }

    let mut context = Context::new();
    let mut executor = GpuExecutor::new(&mut context, device.clone());

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

    let archetype_info = executor
        .get_archetype_info(archetype_id)
        .expect("archetype info should be present");
    let buffer = unsafe { archetype_info.storage().buffer() };
    println!(
        "{archetype_id:?} buffer size is {}, its usage is {:?}",
        buffer.size(),
        buffer.usage(),
    );

    executor.execute();

    if let Ok(renderdoc) = renderdoc.as_mut() {
        log::info!("Ending RenderDoc capture...");
        renderdoc.end_frame_capture(std::ptr::null(), std::ptr::null());
    }
}
