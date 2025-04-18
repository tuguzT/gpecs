use std::{mem::transmute, os::raw::c_void, ptr::null};

use gpecs::{prelude::*, soa::identity::Identity};
use renderdoc::{RenderDoc, V141};

use self::common::{Mass, Position};

mod common;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let instance_desc = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
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

    let device_raw = unsafe {
        device.as_hal::<wgpu::hal::api::Vulkan, _, _>(|device| {
            device
                .map(|device| transmute(device.raw_device().handle()))
                .unwrap_or(null::<c_void>())
        })
    };
    let window_raw = null::<c_void>();

    let mut renderdoc = RenderDoc::<V141>::new();
    match renderdoc.as_mut() {
        Ok(renderdoc) => {
            log::info!("RenderDoc version: {:?}", renderdoc.get_api_version());
            log::info!("Starting RenderDoc capture...");
            renderdoc.start_frame_capture(device_raw, window_raw);
        }
        Err(error) => {
            log::warn!("{error}");
        }
    }

    let mut context = Context::new();
    for i in 0..12 {
        let entity = context.spawn();
        if i % 2 == 0 {
            let x = i as f32;
            let y = -(i as f32);
            let z = 0.0;
            let position = Position { x, y, z };
            context
                .insert_bundle(entity, (position,))
                .expect("entity should exist & archetype of just `Position` should be valid");
        } else {
            let mass = Mass { value: i };
            context
                .insert_bundle(entity, (mass,))
                .expect("entity should exist & archetype of just `Mass` should be valid");
        }
    }
    let position_id = context.register_component::<Position>();
    let mass_id = context.register_component::<Mass>();
    let position_archetype_id = context
        .register_archetype::<Identity<Position>>()
        .expect("`Position` archetype should contain unique component ids");
    let mass_archetype_id = context
        .register_archetype::<Identity<Mass>>()
        .expect("`Mass` archetype should contain unique component ids");

    let mut executor = GpuExecutor::new(&mut context, device.clone());

    let position_gpu_id = executor.register_component::<Position>();
    assert_eq!(position_gpu_id.into_u32(), position_id.into_u32());

    let mass_gpu_archetype_id = executor
        .register_archetype::<Identity<Mass>>()
        .expect("archetype of just `Mass` should contain unique component ids");
    assert_eq!(
        mass_gpu_archetype_id.into_u32(),
        mass_archetype_id.into_u32(),
    );

    let position_gpu_archetype_id = executor
        .register_archetype::<Identity<Position>>()
        .expect("archetype of just `Position` should contain unique component ids");
    assert_eq!(
        position_gpu_archetype_id.into_u32(),
        position_archetype_id.into_u32(),
    );

    let mass_gpu_id = executor
        .component_id::<Mass>()
        .expect("`Mass` component should be registered after registering archetype");
    assert_eq!(mass_gpu_id.into_u32(), mass_id.into_u32());

    let mass_gpu_archetype_info = executor
        .get_archetype_info(mass_gpu_archetype_id)
        .expect("archetype info should be present");
    let buffer_bindings = unsafe { mass_gpu_archetype_info.storage().buffer_bindings() };
    println!("{mass_gpu_archetype_id:?} buffer bindings are {buffer_bindings:#?}");

    let position_gpu_archetype_info = executor
        .get_archetype_info(position_gpu_archetype_id)
        .expect("archetype info should be present");
    let buffer_bindings = unsafe { position_gpu_archetype_info.storage().buffer_bindings() };
    println!("{position_gpu_archetype_id:?} buffer bindings are {buffer_bindings:#?}");

    executor.execute();

    if let Ok(renderdoc) = renderdoc.as_mut() {
        log::info!("Ending RenderDoc capture...");
        renderdoc.end_frame_capture(device_raw, window_raw);
    }
}
