use std::{fs, mem::transmute, os::raw::c_void, ptr::null, slice};

use gpecs::prelude::*;
use renderdoc::{RenderDoc, V141};

use self::common::{Mass, Position, Tag};

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
    log::info!("Running on:\n{:#?}", adapter.get_info());
    log::info!("Adapter features:\n{:#?}", adapter.features());

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
        required_limits: adapter.limits(),
        memory_hints: wgpu::MemoryHints::Performance,
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&device_desc, None))
        .expect("failed to create device & queue");
    log::info!("Limits of the current device:\n{:#?}", device.limits());

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
    for i in 0..24 {
        let entity = context.spawn();
        match i % 3 {
            0 => {
                let position = Position {
                    x: i as f32,
                    y: -(i as f32),
                    z: 0.0,
                };
                context
                    .insert_bundle::<(Tag, Position)>(entity, (Tag, position))
                    .expect("entity should exist & archetype should be valid");
            }
            1 => {
                let mass = Mass { value: i };
                context
                    .insert_bundle::<(Mass, Tag)>(entity, (mass, Tag))
                    .expect("entity should exist & archetype should be valid");
            }
            _ => {
                let position = Position {
                    x: i as f32,
                    y: -(i as f32),
                    z: 0.0,
                };
                let mass = Mass { value: i };
                context
                    .insert_bundle::<(Position, Mass)>(entity, (position, mass))
                    .expect("entity should exist & archetype should be valid");
            }
        }
    }
    let position_id = context.register_component::<Position>();
    let tag_id = context.register_component::<Tag>();
    let mass_id = context.register_component::<Mass>();
    let position_tag_archetype_id = context
        .register_archetype::<(Position, Tag)>()
        .expect("archetype of `Position` and `Tag` should contain unique component ids");
    let mass_archetype_id = context
        .register_archetype::<(Mass,)>()
        .expect("archetype of just `Mass` should contain unique component ids");
    let position_mass_archetype_id = context
        .register_archetype::<(Position, Mass)>()
        .expect("archetype of `Position` and `Mass` should contain unique component ids");

    let mut executor = GpuExecutor::new(&mut context, device.clone());

    let position_gpu_id = executor.register_component::<Position>();
    assert_eq!(position_gpu_id.into_u32(), position_id.into_u32());

    let tag_gpu_id = executor.register_component::<Tag>();
    assert_eq!(tag_gpu_id.into_u32(), tag_id.into_u32());

    let position_mass_gpu_archetype_id = executor
        .register_archetype::<(Position, Mass)>()
        .expect("archetype of `Position` and `Mass` should contain unique component ids");
    assert_eq!(
        position_mass_gpu_archetype_id.into_u32(),
        position_mass_archetype_id.into_u32(),
    );

    let mass_gpu_archetype_id = executor
        .register_archetype::<(Mass,)>()
        .expect("archetype of just `Mass` should contain unique component ids");
    assert_eq!(
        mass_gpu_archetype_id.into_u32(),
        mass_archetype_id.into_u32(),
    );

    let position_tag_gpu_archetype_id = executor
        .register_archetype::<(Position, Tag)>()
        .expect("archetype of just `Position` should contain unique component ids");
    assert_eq!(
        position_tag_gpu_archetype_id.into_u32(),
        position_tag_archetype_id.into_u32(),
    );

    const PATH: &str = env!("gpecs_shader_example.spv");
    log::info!("Loading shader from {PATH}");

    let data = fs::read(PATH).expect("SPIR-V shader file should exist");
    let shader_desc = wgpu::ShaderModuleDescriptor {
        label: Some("`gpecs` example shader"),
        source: wgpu::util::make_spirv(&data),
    };
    let shader_module = device.create_shader_module(shader_desc);
    let shader_compilation_info = pollster::block_on(shader_module.get_compilation_info());
    log::info!("Shader compilation info:\n{shader_compilation_info:#?}");

    let positions_gpu_system_id = executor
        .register_system(
            shader_module,
            Some("copy_entity_indices"),
            true,
            [position_gpu_id],
        )
        .expect("GPU system by shader module should be registered");
    executor.add_system(positions_gpu_system_id);

    let positions_gpu_system_info = executor
        .get_system_info(positions_gpu_system_id)
        .expect("just registered GPU system should be present");
    log::info!("GPU system {positions_gpu_system_id:?} info:\n{positions_gpu_system_info:#?}");

    let mass_gpu_id = executor
        .component_id::<Mass>()
        .expect("`Mass` component should be registered after registering archetype");
    assert_eq!(mass_gpu_id.into_u32(), mass_id.into_u32());

    let mass_gpu_archetype_info = executor
        .get_archetype_info(mass_gpu_archetype_id)
        .expect("archetype info should be present");
    let mass_storage_buffer_bindings =
        unsafe { mass_gpu_archetype_info.storage().storage_buffer_bindings() };
    log::info!("{mass_gpu_archetype_id:?} buffer bindings:\n{mass_storage_buffer_bindings:#?}");

    let position_mass_archetype_info = executor
        .context()
        .archetypes()
        .get_archetype_info(position_mass_archetype_id)
        .expect("archetype info should be present");
    let position_mass_entities = position_mass_archetype_info.storage().entities();
    log::info!("{position_mass_archetype_id:?} has entities:\n{position_mass_entities:#?}");

    let position_mass_gpu_archetype_info = executor
        .get_archetype_info(position_mass_gpu_archetype_id)
        .expect("archetype info should be present");
    let position_mass_storage_buffer_bindings = unsafe {
        position_mass_gpu_archetype_info
            .storage()
            .storage_buffer_bindings()
    };
    log::info!("{position_mass_gpu_archetype_id:?} buffer bindings:\n{position_mass_storage_buffer_bindings:#?}");

    let position_tag_archetype_info = executor
        .context()
        .archetypes()
        .get_archetype_info(position_tag_archetype_id)
        .expect("archetype info should be present");
    let position_tag_entities = position_tag_archetype_info.storage().entities();
    log::info!("{position_tag_archetype_id:?} has entities:\n{position_tag_entities:#?}");

    let position_tag_gpu_archetype_info = executor
        .get_archetype_info(position_tag_gpu_archetype_id)
        .expect("archetype info should be present");
    let position_tag_storage_buffer_bindings = unsafe {
        position_tag_gpu_archetype_info
            .storage()
            .storage_buffer_bindings()
    };
    log::info!("{position_tag_gpu_archetype_id:?} buffer bindings:\n{position_tag_storage_buffer_bindings:#?}");

    let position_tag_entities_binding = position_tag_storage_buffer_bindings.entities;
    let position_tag_positions_binding = position_tag_storage_buffer_bindings
        .components
        .get(&position_id)
        .cloned()
        .flatten();
    let position_tag_tags_binding = position_tag_storage_buffer_bindings
        .components
        .get(&tag_id)
        .cloned()
        .flatten();
    assert!(position_tag_tags_binding.is_none());

    let command_encoder_desc = wgpu::CommandEncoderDescriptor {
        label: Some("`gpecs` example command encoder"),
    };
    let mut command_encoder = device.create_command_encoder(&command_encoder_desc);

    let compute_pass_desc = wgpu::ComputePassDescriptor {
        label: Some("`gpecs` example compute pass"),
        timestamp_writes: None,
    };
    let mut compute_pass = command_encoder.begin_compute_pass(&compute_pass_desc);

    let mut position_tag_download_buffer = None;
    if let Some((position_tag_entities_binding, position_tag_positions_binding)) =
        position_tag_entities_binding.zip(position_tag_positions_binding.clone())
    {
        let entities_bind_group_entry = wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(position_tag_entities_binding),
        };
        let positions_bind_group_entry = wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Buffer(position_tag_positions_binding.clone()),
        };
        let bind_group_desc = wgpu::BindGroupDescriptor {
            label: Some("`gpecs` example (`Position`, `Tag`) bind group"),
            layout: positions_gpu_system_info.shader().bind_group_layout(),
            entries: &[entities_bind_group_entry, positions_bind_group_entry],
        };
        let bind_group = device.create_bind_group(&bind_group_desc);

        compute_pass.set_pipeline(positions_gpu_system_info.shader().compute_pipeline());
        compute_pass.set_bind_group(0, &bind_group, &[]);

        let workgroup_count = position_tag_entities
            .len()
            .div_ceil(64)
            .try_into()
            .expect("workgroup count should fit into `u32`");
        compute_pass.dispatch_workgroups(workgroup_count, 1, 1);

        let position_tag_download_buffer_desc = wgpu::BufferDescriptor {
            label: Some("`gpecs` example (`Position`, `Tag`) download buffer"),
            size: position_tag_positions_binding
                .size
                .expect("component binding never uses the whole buffer")
                .get(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        };
        position_tag_download_buffer = device
            .create_buffer(&position_tag_download_buffer_desc)
            .into();
    }

    let position_mass_entities_binding = position_mass_storage_buffer_bindings.entities;
    let position_mass_positions_binding = position_mass_storage_buffer_bindings
        .components
        .get(&position_id)
        .cloned()
        .flatten();
    let position_mass_masses_binding = position_mass_storage_buffer_bindings
        .components
        .get(&mass_id)
        .cloned()
        .flatten();
    assert!(position_mass_masses_binding.is_some());

    let mut position_mass_download_buffer = None;
    if let Some((position_mass_entities_binding, position_mass_positions_binding)) =
        position_mass_entities_binding.zip(position_mass_positions_binding.clone())
    {
        let entities_bind_group_entry = wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(position_mass_entities_binding),
        };
        let positions_bind_group_entry = wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Buffer(position_mass_positions_binding.clone()),
        };
        let bind_group_desc = wgpu::BindGroupDescriptor {
            label: Some("`gpecs` example (`Position`, `Mass`) bind group"),
            layout: positions_gpu_system_info.shader().bind_group_layout(),
            entries: &[entities_bind_group_entry, positions_bind_group_entry],
        };
        let bind_group = device.create_bind_group(&bind_group_desc);

        compute_pass.set_pipeline(positions_gpu_system_info.shader().compute_pipeline());
        compute_pass.set_bind_group(0, &bind_group, &[]);

        let workgroup_count = position_mass_entities
            .len()
            .div_ceil(64)
            .try_into()
            .expect("workgroup count should fit into `u32`");
        compute_pass.dispatch_workgroups(workgroup_count, 1, 1);

        let position_mass_download_buffer_desc = wgpu::BufferDescriptor {
            label: Some("`gpecs` example (`Position`, `Mass`) download buffer"),
            size: position_mass_positions_binding
                .size
                .expect("component binding never uses the whole buffer")
                .get(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        };
        position_mass_download_buffer = device
            .create_buffer(&position_mass_download_buffer_desc)
            .into();
    }

    drop(compute_pass);

    if let Some((position_tag_download_buffer, position_tag_positions_binding)) =
        position_tag_download_buffer
            .clone()
            .zip(position_tag_positions_binding)
    {
        command_encoder.copy_buffer_to_buffer(
            position_tag_positions_binding.buffer,
            position_tag_positions_binding.offset,
            &position_tag_download_buffer,
            0,
            position_tag_positions_binding.size.unwrap().get(),
        );
    }
    if let Some((position_mass_download_buffer, position_mass_positions_binding)) =
        position_mass_download_buffer
            .clone()
            .zip(position_mass_positions_binding)
    {
        command_encoder.copy_buffer_to_buffer(
            position_mass_positions_binding.buffer,
            position_mass_positions_binding.offset,
            &position_mass_download_buffer,
            0,
            position_mass_positions_binding.size.unwrap().get(),
        );
    }

    executor.execute(&mut command_encoder);

    let command_buffer = command_encoder.finish();
    queue.submit([command_buffer]);

    if let Some(position_tag_download_buffer) = position_tag_download_buffer.clone() {
        let position_tag_download_slice = position_tag_download_buffer.slice(..);
        position_tag_download_slice.map_async(wgpu::MapMode::Read, |_| {});
    }
    if let Some(position_mass_download_buffer) = position_mass_download_buffer.clone() {
        let position_mass_download_slice = position_mass_download_buffer.slice(..);
        position_mass_download_slice.map_async(wgpu::MapMode::Read, |_| {});
    }

    device.poll(wgpu::Maintain::Wait).panic_on_timeout();

    if let Some(position_tag_download_buffer) = position_tag_download_buffer {
        let position_tag_archetype_info = executor
            .context()
            .archetypes()
            .get_archetype_info(position_tag_archetype_id)
            .expect("archetype info should be present");
        let position_tag_entities = position_tag_archetype_info.storage().entities();

        let position_tag_positions: &[Position] = unsafe {
            slice::from_raw_parts(
                position_tag_download_buffer
                    .slice(..)
                    .get_mapped_range()
                    .as_ptr()
                    .cast(),
                position_tag_entities.len(),
            )
        };
        log::info!("Compute output:\n{position_tag_positions:#?}");

        itertools::assert_equal(
            position_tag_entities.iter().map(|entity| Position {
                x: entity.index() as f32,
                y: (entity.index() as f32) / 2.0,
                z: -(entity.index() as f32) / 2.0,
            }),
            position_tag_positions.iter().copied(),
        );
    }
    if let Some(position_mass_download_buffer) = position_mass_download_buffer {
        let position_mass_archetype_info = executor
            .context()
            .archetypes()
            .get_archetype_info(position_mass_archetype_id)
            .expect("archetype info should be present");
        let position_mass_entities = position_mass_archetype_info.storage().entities();

        let position_mass_positions: &[Position] = unsafe {
            slice::from_raw_parts(
                position_mass_download_buffer
                    .slice(..)
                    .get_mapped_range()
                    .as_ptr()
                    .cast(),
                position_mass_entities.len(),
            )
        };
        log::info!("Compute output:\n{position_mass_positions:#?}");

        itertools::assert_equal(
            position_mass_entities.iter().map(|entity| Position {
                x: entity.index() as f32,
                y: (entity.index() as f32) / 2.0,
                z: -(entity.index() as f32) / 2.0,
            }),
            position_mass_positions.iter().copied(),
        );
    }

    if let Ok(renderdoc) = renderdoc.as_mut() {
        log::info!("Ending RenderDoc capture...");
        renderdoc.end_frame_capture(device_raw, window_raw);
    }
}
