use std::{ffi::c_void, fs, mem::transmute, ptr::null, slice};

use glam::Vec3;
use gpecs::prelude::*;
use gpecs_simple_types::*;
use renderdoc::{RenderDoc, V141};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut context = Context::new();
    setup_context(&mut context);

    let mut executor = CpuExecutor::new(&mut context);
    let check_positions_system = executor.register_system(check_positions);
    let check_masses_system = executor.register_system(check_masses);
    let check_tags_system = executor.register_system(check_tags);

    // Setup order of systems' execution
    executor.add_system(check_positions_system);
    executor.add_system(check_tags_system);
    executor.add_system(check_masses_system);

    executor.execute();

    // Return context from the executor
    let mut context = executor.into_context();

    let (device, queue) = init_wgpu();

    let mut renderdoc = init_renderdoc();
    renderdoc_start_frame_capture(renderdoc.as_mut(), &device);

    let mut executor = GpuExecutor::new(&mut context, device.clone());

    executor
        .register_archetype::<(Position, Mass)>()
        .expect("archetype of `Position` and `Mass` should contain unique component ids");
    let position_tag_gpu_archetype_id = executor
        .register_archetype::<(Position, Tag)>()
        .expect("archetype of `Position` and `Tag` should contain unique component ids");

    let shader_module = init_wgpu_shader(&device);

    let position_gpu_id = executor.register_component::<Position>();
    let positions_gpu_system_id = executor
        .register_system(
            shader_module.clone(),
            Some(64),
            Some("update_entity_position"),
            true,
            [position_gpu_id],
        )
        .expect("GPU system by shader module should be registered");

    let mass_gpu_id = executor.register_component::<Mass>();
    let mass_gpu_system_id = executor
        .register_system(
            shader_module,
            Some(64),
            Some("update_entity_mass"),
            true,
            [mass_gpu_id],
        )
        .expect("GPU system by shader module should be registered");

    let tag_gpu_id = executor.register_component::<Tag>();

    executor.add_system(positions_gpu_system_id);
    executor.add_system(mass_gpu_system_id);

    // Create download buffer for archetype of `Position` and `Tag`
    let position_tag_download_buffer = init_wgpu_position_tag_download_buffer(
        &executor,
        position_tag_gpu_archetype_id,
        position_gpu_id,
        tag_gpu_id,
    );

    let mut command_encoder = init_wgpu_command_encoder(&device);
    executor.execute(&mut command_encoder);

    // Push commands to copy data into the download buffer
    wgpu_copy_into_position_tag_download_buffer(
        &executor,
        position_tag_download_buffer.as_ref(),
        &mut command_encoder,
        position_tag_gpu_archetype_id,
        position_gpu_id,
        tag_gpu_id,
    );

    let command_buffer = command_encoder.finish();
    queue.submit([command_buffer]);

    // Map download buffer to CPU memory
    let position_tag_download_slice =
        wgpu_map_position_tag_download_buffer(position_tag_download_buffer.as_ref());

    device.poll(wgpu::Maintain::Wait).panic_on_timeout();

    // Check data inside of the download buffer
    if let Some(position_tag_download_slice) = position_tag_download_slice {
        let position_tag_archetype_info = executor
            .context()
            .archetypes()
            .get_archetype_info(position_tag_gpu_archetype_id.into_id())
            .expect("archetype info should be present");
        let position_tag_entities = position_tag_archetype_info.storage().entities();

        let position_tag_download_slice_mapped_range =
            position_tag_download_slice.get_mapped_range();
        let position_tag_positions: &[Position] = unsafe {
            slice::from_raw_parts(
                position_tag_download_slice_mapped_range.as_ptr().cast(),
                position_tag_entities.len(),
            )
        };
        log::info!("Positions of {position_tag_gpu_archetype_id:?}:\n{position_tag_positions:#?}");

        itertools::assert_equal(
            position_tag_entities.iter().map(|entity| Position {
                data: Vec3 {
                    x: entity.index() as f32,
                    y: (entity.index() as f32) / 2.0,
                    z: -(entity.index() as f32) / 2.0,
                },
            }),
            position_tag_positions.iter().copied(),
        );
    }

    renderdoc_end_frame_capture(renderdoc.as_mut(), &device);
}

fn setup_context(context: &mut Context) {
    log::info!("Filling context with data to process...");
    for i in 0..24 {
        let entity = context.spawn();

        let position = Position {
            data: Vec3 {
                x: i as f32,
                y: -(i as f32),
                z: 0.0,
            },
        };
        let mass = Mass { value: i };
        match i % 3 {
            0 => {
                context
                    .insert_bundle::<(Tag, Position)>(entity, (Tag, position))
                    .expect("entity should exist & archetype should be valid");
            }
            1 => {
                context
                    .insert_bundle::<(Mass, Tag)>(entity, (mass, Tag))
                    .expect("entity should exist & archetype should be valid");
            }
            _ => {
                context
                    .insert_bundle::<(Position, Mass)>(entity, (position, mass))
                    .expect("entity should exist & archetype should be valid");
            }
        }
    }
}

fn check_positions(positions: BundlesMut<(Position,)>) {
    log::info!("Hello from the CPU system working with positions!");

    let mut positions_count = 0;
    for (entity, (position,)) in positions {
        assert!(matches!(entity.index() % 3, 0 | 2));
        assert_eq!(position.data.x, entity.index() as f32);
        assert_eq!(position.data.y, -(entity.index() as f32));
        assert_eq!(position.data.z, 0.0);

        log::info!("{entity} has position of {}", position.data);
        positions_count += 1;
    }
    assert_eq!(positions_count, 16);
}

fn check_masses(context: &mut Context) {
    log::info!("Hello from the CPU system working with masses!");

    let mut masses_count = 0;
    let masses = context
        .bundles_mut::<(Mass,)>()
        .expect("archetype of `Mass` should exist");
    for (entity, (mass,)) in masses {
        assert!(matches!(entity.index() % 3, 1 | 2));
        assert_eq!(mass.value, entity.index());

        log::info!("{entity} has mass of {}", mass.value);
        masses_count += 1;
    }
    assert_eq!(masses_count, 16);
}

fn check_tags(tags: Bundles<(Tag,)>) {
    log::info!("Hello from the CPU system working with tags!");

    let mut tags_count = 0;
    for (entity, (tag,)) in tags {
        assert_eq!(tag, &Tag);

        log::info!("{entity} has {tag:?}");
        tags_count += 1;
    }
    assert_eq!(tags_count, 16);
}

fn init_wgpu() -> (wgpu::Device, wgpu::Queue) {
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
        .expect("failed to create adapter");
    log::info!("Running on:\n{:#?}", adapter.get_info());
    log::info!("Adapter features:\n{:#?}", adapter.features());

    let downlevel_capabilities = adapter.get_downlevel_capabilities();
    if !downlevel_capabilities
        .flags
        .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS)
    {
        panic!("adapter does not support compute shaders, which are required");
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

    (device, queue)
}

fn init_wgpu_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    const PATH: &str = env!("gpecs_simple_shader.spv");
    log::info!("Loading shader from {PATH}");

    let data = fs::read(PATH).expect("SPIR-V shader file should exist");
    let shader_desc = wgpu::ShaderModuleDescriptor {
        label: Some("`gpecs` simple example shader"),
        source: wgpu::util::make_spirv(&data),
    };
    let shader_module = device.create_shader_module(shader_desc);
    let shader_compilation_info = pollster::block_on(shader_module.get_compilation_info());
    log::info!("Shader compilation info:\n{shader_compilation_info:#?}");

    shader_module
}

fn init_wgpu_command_encoder(device: &wgpu::Device) -> wgpu::CommandEncoder {
    let command_encoder_desc = wgpu::CommandEncoderDescriptor {
        label: Some("`gpecs` simple example command encoder"),
    };
    device.create_command_encoder(&command_encoder_desc)
}

fn init_wgpu_position_tag_download_buffer(
    executor: &GpuExecutor,
    position_tag_gpu_archetype_id: GpuArchetypeId,
    position_gpu_id: GpuComponentId,
    tag_gpu_id: GpuComponentId,
) -> Option<wgpu::Buffer> {
    let position_tag_gpu_archetype_info = executor
        .get_archetype_info(position_tag_gpu_archetype_id)
        .expect("archetype info should be present");
    let position_tag_storage_buffer_bindings = unsafe {
        position_tag_gpu_archetype_info
            .storage()
            .storage_buffer_bindings()
    };
    log::info!("{position_tag_gpu_archetype_id:?} buffer bindings:\n{position_tag_storage_buffer_bindings:#?}");

    let position_tag_positions_binding = position_tag_storage_buffer_bindings
        .components
        .get(&position_gpu_id.into_id())
        .cloned()
        .flatten()?;
    let position_tag_tags_binding = position_tag_storage_buffer_bindings
        .components
        .get(&tag_gpu_id.into_id())
        .cloned()
        .flatten();
    assert!(position_tag_tags_binding.is_none());

    let position_tag_download_buffer_label =
        format!("`gpecs` {position_tag_gpu_archetype_id:?} download buffer");
    let position_tag_download_buffer_desc = wgpu::BufferDescriptor {
        label: Some(&position_tag_download_buffer_label),
        size: position_tag_positions_binding
            .size
            .expect("component binding never uses the whole buffer")
            .get(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    };
    executor
        .device()
        .create_buffer(&position_tag_download_buffer_desc)
        .into()
}

fn wgpu_copy_into_position_tag_download_buffer(
    executor: &GpuExecutor,
    position_tag_download_buffer: Option<&wgpu::Buffer>,
    command_encoder: &mut wgpu::CommandEncoder,
    position_tag_gpu_archetype_id: GpuArchetypeId,
    position_gpu_id: GpuComponentId,
    tag_gpu_id: GpuComponentId,
) {
    let position_tag_gpu_archetype_info = executor
        .get_archetype_info(position_tag_gpu_archetype_id)
        .expect("archetype info should be present");
    let position_tag_storage_buffer_bindings = unsafe {
        position_tag_gpu_archetype_info
            .storage()
            .storage_buffer_bindings()
    };
    log::info!("{position_tag_gpu_archetype_id:?} buffer bindings:\n{position_tag_storage_buffer_bindings:#?}");

    let position_tag_positions_binding = position_tag_storage_buffer_bindings
        .components
        .get(&position_gpu_id.into_id())
        .cloned()
        .flatten();
    let position_tag_tags_binding = position_tag_storage_buffer_bindings
        .components
        .get(&tag_gpu_id.into_id())
        .cloned()
        .flatten();
    assert!(position_tag_tags_binding.is_none());

    if let Some((position_tag_download_buffer, position_tag_positions_binding)) =
        position_tag_download_buffer.zip(position_tag_positions_binding)
    {
        command_encoder.copy_buffer_to_buffer(
            position_tag_positions_binding.buffer,
            position_tag_positions_binding.offset,
            &position_tag_download_buffer,
            0,
            position_tag_positions_binding.size.unwrap().get(),
        );
    }
}

fn wgpu_map_position_tag_download_buffer(
    position_tag_download_buffer: Option<&wgpu::Buffer>,
) -> Option<wgpu::BufferSlice<'_>> {
    let position_tag_download_slice = position_tag_download_buffer?.slice(..);
    position_tag_download_slice.map_async(wgpu::MapMode::Read, |_| {});
    position_tag_download_slice.into()
}

fn init_renderdoc() -> Option<RenderDoc<V141>> {
    match RenderDoc::<V141>::new() {
        Ok(renderdoc) => {
            log::info!("RenderDoc version: {:?}", renderdoc.get_api_version());
            Some(renderdoc)
        }
        Err(error) => {
            log::warn!("{error}");
            None
        }
    }
}

fn wgpu_raw_device_window(device: &wgpu::Device) -> (*const c_void, *const c_void) {
    let device_raw = unsafe {
        device.as_hal::<wgpu::hal::api::Vulkan, _, _>(|device| {
            device
                .map(|device| transmute(device.raw_device().handle()))
                .unwrap_or(null::<c_void>())
        })
    };
    let window_raw = null::<c_void>();
    (device_raw, window_raw)
}

fn renderdoc_start_frame_capture(renderdoc: Option<&mut RenderDoc<V141>>, device: &wgpu::Device) {
    let Some(renderdoc) = renderdoc else {
        return;
    };

    log::info!("Starting RenderDoc capture...");
    let (device_raw, window_raw) = wgpu_raw_device_window(device);
    renderdoc.start_frame_capture(device_raw, window_raw);
}

fn renderdoc_end_frame_capture(renderdoc: Option<&mut RenderDoc<V141>>, device: &wgpu::Device) {
    let Some(renderdoc) = renderdoc else {
        return;
    };

    log::info!("Ending RenderDoc capture...");
    let (device_raw, window_raw) = wgpu_raw_device_window(device);
    renderdoc.end_frame_capture(device_raw, window_raw);
}
