use std::{fs, mem::transmute, os::raw::c_void, path, ptr::null, slice};

use gpecs::{prelude::*, soa::prelude::*};
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
    for i in 0..12 {
        let entity = context.spawn();
        if i % 2 == 0 {
            let position = Position {
                x: i as f32,
                y: -(i as f32),
                z: 0.0,
                _padding: 0.0,
            };
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
    log::info!("{mass_gpu_archetype_id:?} buffer bindings:\n{buffer_bindings:#?}");

    let position_archetype_info = executor
        .context()
        .archetypes()
        .get_archetype_info(position_archetype_id)
        .expect("archetype info should be present");
    let position_entities = position_archetype_info.storage().entities();
    log::info!("{position_archetype_id:?} has entities:\n{position_entities:#?}");

    let position_gpu_archetype_info = executor
        .get_archetype_info(position_gpu_archetype_id)
        .expect("archetype info should be present");
    let buffer_bindings = unsafe { position_gpu_archetype_info.storage().buffer_bindings() };
    log::info!("{position_gpu_archetype_id:?} buffer bindings:\n{buffer_bindings:#?}");

    let entities_binding = buffer_bindings.entities;
    let positions_binding = buffer_bindings
        .components
        .get(&position_id)
        .cloned()
        .flatten();
    if let Some((entities_binding, positions_binding)) = entities_binding.zip(positions_binding) {
        let path = path::absolute(env!("gpecs_shader_example.spv")).expect("path should be valid");
        log::info!("Loading shader from {path:?}");

        let data = fs::read(path).expect("SPIR-V shader file should exist");
        let shader_desc = wgpu::ShaderModuleDescriptor {
            label: Some("`gpecs` example shader"),
            source: wgpu::util::make_spirv(&data),
        };
        let shader_module = device.create_shader_module(shader_desc);
        let shader_compilation_info = pollster::block_on(shader_module.get_compilation_info());
        log::info!("Shader compilation info:\n{shader_compilation_info:#?}");

        let download_buffer_desc = wgpu::BufferDescriptor {
            label: Some("`gpecs` example download buffer"),
            size: (position_entities.len() * size_of::<Position>())
                .try_into()
                .expect("buffer size should fit into `u64`"),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        };
        let download_buffer = device.create_buffer(&download_buffer_desc);

        let entities_bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                min_binding_size: Some(
                    u64::try_from(size_of::<Entity>())
                        .unwrap()
                        .try_into()
                        .unwrap(),
                ),
                has_dynamic_offset: false,
            },
            count: None,
        };
        let positions_bind_group_layout_entry = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                min_binding_size: Some(
                    u64::try_from(size_of::<Position>())
                        .unwrap()
                        .try_into()
                        .unwrap(),
                ),
                has_dynamic_offset: false,
            },
            count: None,
        };
        let bind_group_layout_desc = wgpu::BindGroupLayoutDescriptor {
            label: Some("`gpecs` example bind group layout"),
            entries: &[
                entities_bind_group_layout_entry,
                positions_bind_group_layout_entry,
            ],
        };
        let bind_group_layout = device.create_bind_group_layout(&bind_group_layout_desc);

        let entities_bind_group_entry = wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(entities_binding),
        };
        let positions_bind_group_entry = wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Buffer(positions_binding.clone()),
        };
        let bind_group_desc = wgpu::BindGroupDescriptor {
            label: Some("`gpecs` example bind group"),
            layout: &bind_group_layout,
            entries: &[entities_bind_group_entry, positions_bind_group_entry],
        };
        let bind_group = device.create_bind_group(&bind_group_desc);

        let pipeline_layout_desc = wgpu::PipelineLayoutDescriptor {
            label: Some("`gpecs` example compute pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        };
        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_desc);

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("`gpecs` example compute pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: Some("copy_entity_indices"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let command_encoder_desc = wgpu::CommandEncoderDescriptor {
            label: Some("`gpecs` example command encoder"),
        };
        let mut command_encoder = device.create_command_encoder(&command_encoder_desc);

        {
            let compute_pass_desc = wgpu::ComputePassDescriptor {
                label: Some("`gpecs` example compute pass"),
                timestamp_writes: None,
            };
            let mut compute_pass = command_encoder.begin_compute_pass(&compute_pass_desc);

            compute_pass.set_pipeline(&pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);

            let workgroup_count = position_entities
                .len()
                .div_ceil(64)
                .try_into()
                .expect("workgroup count should fit into `u32`");
            compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
        }

        command_encoder.copy_buffer_to_buffer(
            positions_binding.buffer,
            positions_binding.offset,
            &download_buffer,
            0,
            positions_binding.size.unwrap().get(),
        );

        let command_buffer = command_encoder.finish();
        queue.submit([command_buffer]);

        let download_slice = download_buffer.slice(..);
        download_slice.map_async(wgpu::MapMode::Read, |_| {});

        device.poll(wgpu::Maintain::Wait).panic_on_timeout();
        let positions = &download_slice.get_mapped_range()[..];
        let positions: &[Position] = unsafe {
            slice::from_raw_parts(
                positions.as_ptr() as *const Position,
                position_entities.len(),
            )
        };
        log::info!("Compute output:\n{positions:#?}");

        itertools::assert_equal(
            position_entities.iter().map(|entity| Position {
                x: entity.index() as f32,
                y: (entity.index() as f32) / 2.0,
                z: -(entity.index() as f32) / 2.0,
                _padding: 0.0,
            }),
            positions.iter().copied(),
        );
    }

    executor.execute();

    if let Ok(renderdoc) = renderdoc.as_mut() {
        log::info!("Ending RenderDoc capture...");
        renderdoc.end_frame_capture(device_raw, window_raw);
    }
}
