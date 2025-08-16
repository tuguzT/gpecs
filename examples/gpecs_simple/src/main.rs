use std::{
    ffi::c_void,
    fs,
    mem::transmute,
    ptr::null,
    slice,
    time::{Duration, Instant},
};

use glam::Vec3;
use gpecs::prelude::*;
use gpecs_simple_types::{Mass, Position, Tag};
use itertools::Itertools;
use num_traits::ToPrimitive;
use renderdoc::{RenderDoc, V141};

const ITER_COUNT: usize = 10;
const ENTITY_COUNT: u32 = if cfg!(debug_assertions) {
    2_400
} else {
    1_200_000
};

#[expect(clippy::too_many_lines)]
fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut context = Context::new();
    setup_context(&mut context);

    let mut executor = CpuExecutor::new(&mut context);
    let check_positions_system = executor.register_system(update_positions);
    let check_masses_system = executor.register_system(update_masses);
    // let check_tags_system = executor.register_system(check_tags);

    // Setup order of systems' execution
    executor.add_system(check_positions_system);
    // executor.add_system(check_tags_system);
    executor.add_system(check_masses_system);

    log::info!("Starting to execute systems on CPU...");
    for i in 0..ITER_COUNT {
        let start = Instant::now();
        executor.execute();
        let duration = start.elapsed();
        log::info!("CPU system execution {i} took {duration:?}");
    }

    // Return context from the executor
    let _ = executor.into_context();

    let (device, queue) = init_wgpu();

    let mut renderdoc = init_renderdoc();

    context.clear();
    setup_context(&mut context);

    let mut executor = GpuExecutor::new(&mut context, device.clone());

    executor
        .register_archetype::<(Position, Mass)>()
        .expect("archetype of `Position` and `Mass` should contain unique component ids");
    let _position_tag_gpu_archetype_id = executor
        .register_archetype::<(Position, Tag)>()
        .expect("archetype of `Position` and `Tag` should contain unique component ids");

    let shader_module = init_wgpu_shader(&device);

    let position_gpu_id = executor.register_component::<Position>();
    let position_gpu_system_descriptor = GpuSystemDescriptor {
        shader_module: shader_module.clone(),
        entry_point: Some("update_entity_position"),
        workgroup_count: Some(64),
        bind_entities: true,
        bind_components: [position_gpu_id],
        additional_bindings: [],
    };
    let positions_gpu_system_id = executor
        .register_system(position_gpu_system_descriptor)
        .expect("GPU system by shader module should be registered");

    let mass_gpu_id = executor.register_component::<Mass>();
    let mass_gpu_system_descriptor = GpuSystemDescriptor {
        shader_module,
        entry_point: Some("update_entity_mass"),
        workgroup_count: Some(64),
        bind_entities: true,
        bind_components: [mass_gpu_id],
        additional_bindings: [],
    };
    let mass_gpu_system_id = executor
        .register_system(mass_gpu_system_descriptor)
        .expect("GPU system by shader module should be registered");

    let _tag_gpu_id = executor.register_component::<Tag>();

    executor.add_system(positions_gpu_system_id);
    executor.add_system(mass_gpu_system_id);

    // Create download buffer for archetype of `Position` and `Tag`
    // let position_tag_download_buffer = init_wgpu_position_tag_download_buffer(
    //     &executor,
    //     position_tag_gpu_archetype_id,
    //     position_gpu_id,
    //     tag_gpu_id,
    // );

    // Create download buffer for results of timestamp queries
    let mut timestamp_query_download_buffer = None;

    log::info!("Starting to execute systems on GPU...");
    for i in 0..ITER_COUNT {
        renderdoc_start_frame_capture(renderdoc.as_mut(), &device);
        let start = Instant::now();

        let mut command_encoder = init_wgpu_command_encoder(&device);
        executor.execute(&mut command_encoder);

        // Push commands to copy data into the download buffer
        // wgpu_copy_into_position_tag_download_buffer(
        //     &executor,
        //     position_tag_download_buffer.as_ref(),
        //     &mut command_encoder,
        //     position_tag_gpu_archetype_id,
        //     position_gpu_id,
        //     tag_gpu_id,
        // );

        if timestamp_query_download_buffer.is_none() {
            timestamp_query_download_buffer = init_wgpu_timestamp_query_download_buffer(&executor);
        }

        // Push commands to copy data into the timestamp query download buffer
        wgpu_copy_into_timestamp_query_download_buffer(
            &executor,
            timestamp_query_download_buffer.as_ref(),
            &mut command_encoder,
        );

        let command_buffer = command_encoder.finish();
        queue.submit([command_buffer]);

        // Map download buffer to CPU memory
        // let position_tag_download_slice =
        //     wgpu_map_whole_buffer(position_tag_download_buffer.as_ref());

        // Map timestamp query download buffer to CPU memory
        let timestamp_query_download_slice =
            wgpu_map_whole_buffer(timestamp_query_download_buffer.as_ref());

        device
            .poll(wgpu::PollType::Wait)
            .expect("device should poll");

        let duration = start.elapsed();
        log::info!("GPU system execution {i} overall took {duration:?}");

        // Check data inside of the download buffer
        // if let Some(position_tag_download_slice) = position_tag_download_slice {
        //     let position_tag_archetype_info = executor
        //         .context()
        //         .archetypes()
        //         .get_archetype_info(position_tag_gpu_archetype_id.into_id())
        //         .expect("archetype info should be present");
        //     let position_tag_entities = position_tag_archetype_info.storage().entities();

        //     let position_tag_download_slice_mapped_range =
        //         position_tag_download_slice.get_mapped_range();
        //     let position_tag_positions: &[Position] = unsafe {
        //         slice::from_raw_parts(
        //             position_tag_download_slice_mapped_range.as_ptr().cast(),
        //             position_tag_entities.len(),
        //         )
        //     };
        //     log::debug!(
        //         "Positions of {position_tag_gpu_archetype_id:?}:\n{position_tag_positions:#?}"
        //     );

        //     itertools::assert_equal(
        //         position_tag_entities.iter().map(|entity| Position {
        //             data: Vec3 {
        //                 x: entity.index() as f32,
        //                 y: (entity.index() as f32) / 2.0,
        //                 z: -(entity.index() as f32) / 2.0,
        //             },
        //         }),
        //         position_tag_positions.iter().copied(),
        //     );
        // }
        // if let Some(position_tag_download_buffer) = position_tag_download_buffer.as_ref() {
        //     position_tag_download_buffer.unmap();
        // }

        // Check data inside of the timestamp query download buffer
        if let Some((timestamp_query_download_slice, timestamp_query_resources)) =
            timestamp_query_download_slice.zip(executor.timestamp_query_resources())
        {
            let timestamp_query_download_slice_mapped_range =
                timestamp_query_download_slice.get_mapped_range();
            let timestamp_query_raw: &[u64] = unsafe {
                slice::from_raw_parts(
                    timestamp_query_download_slice_mapped_range.as_ptr().cast(),
                    timestamp_query_resources
                        .count()
                        .get()
                        .try_into()
                        .expect("count of queries should fit into `usize`"),
                )
            };
            log::info!("Timestamp query raw data: {timestamp_query_raw:?}");

            let timestamp_period_nanos = queue.get_timestamp_period().to_u64().unwrap();
            for (index, (&first, &second)) in timestamp_query_raw.iter().tuple_windows().enumerate()
            {
                let nanos = (second - first) * timestamp_period_nanos;
                let duration = Duration::from_nanos(nanos);
                log::info!("Timestamp query {index} duration: {duration:?}");
            }
        }
        if let Some(timestamp_query_download_buffer) = timestamp_query_download_buffer.as_ref() {
            timestamp_query_download_buffer.unmap();
        }

        renderdoc_end_frame_capture(renderdoc.as_mut(), &device);
    }
}

fn setup_context(context: &mut Context) {
    log::info!("Filling context with data to process...");
    for i in 0..ENTITY_COUNT {
        let entity = context.spawn();

        let position = Position {
            data: Vec3 {
                x: i.to_f32().unwrap(),
                y: -(i.to_f32().unwrap()),
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

fn update_positions(positions: BundlesMut<(Position,)>) {
    // log::info!("Hello from the CPU system working with positions!");

    // let mut positions_count = 0;
    let start = Instant::now();
    for (entity, (position,)) in positions {
        assert!(matches!(entity.index() % 3, 0 | 2));
        // assert_eq!(position.data.x, entity.index() as f32);
        // assert_eq!(position.data.y, -(entity.index() as f32));
        // assert_eq!(position.data.z, 0.0);

        // log::debug!("{entity} has position of {}", position.data);
        position.data = Vec3 {
            x: entity.index().to_f32().unwrap(),
            y: entity.index().to_f32().unwrap() / 2.0,
            z: -entity.index().to_f32().unwrap() / 2.0,
        };
        log::debug!("{entity} position have been updated to {}", position.data);

        // positions_count += 1;
    }
    // assert_eq!(positions_count, ENTITY_COUNT / 3 * 2);
    let duration = start.elapsed();
    log::info!("CPU system working with positions took {duration:?}");
}

fn update_masses(context: &mut Context) {
    // log::info!("Hello from the CPU system working with masses!");

    // let mut masses_count = 0;
    let start = Instant::now();
    let masses = context
        .bundles_mut::<(Mass,)>()
        .expect("archetype of `Mass` should exist");
    for (entity, (mass,)) in masses {
        assert!(matches!(entity.index() % 3, 1 | 2));
        // assert_eq!(mass.value, entity.index());

        // log::debug!("{entity} has mass of {}", mass.value);
        mass.value = entity.index();
        log::debug!("{entity} mass have been updated to {}", mass.value);

        // masses_count += 1;
    }
    // assert_eq!(masses_count, ENTITY_COUNT / 3 * 2);
    let duration = start.elapsed();
    log::info!("CPU system working with masses took {duration:?}");
}

fn _check_tags(tags: Bundles<(Tag,)>) {
    // log::info!("Hello from the CPU system working with tags!");

    // let mut tags_count = 0;
    let start = Instant::now();
    for (entity, (tag,)) in tags {
        assert!(matches!(entity.index() % 3, 0 | 1));
        // assert_eq!(tag, &Tag);

        log::debug!("{entity} has {tag:?}");
        // tags_count += 1;
    }
    // assert_eq!(tags_count, ENTITY_COUNT / 3 * 2);
    let duration = start.elapsed();
    log::info!("CPU system working with tags took {duration:?}");
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

    let features = adapter.features();
    assert!(
        features.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES),
        "adapter does not support timestamp queries inside passes, which are required",
    );

    let device_desc = wgpu::DeviceDescriptor {
        label: Some("`gpecs` integration test device"),
        required_features: wgpu::Features::TIMESTAMP_QUERY
            | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES,
        required_limits: adapter.limits(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&device_desc))
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

fn _init_wgpu_position_tag_download_buffer(
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
    log::debug!(
        "{position_tag_gpu_archetype_id:?} buffer bindings:\n{position_tag_storage_buffer_bindings:#?}"
    );

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

fn init_wgpu_timestamp_query_download_buffer(executor: &GpuExecutor) -> Option<wgpu::Buffer> {
    let timestamp_query_resources = executor.timestamp_query_resources()?;
    let timestamp_query_download_buffer_desc = wgpu::BufferDescriptor {
        label: Some("`gpecs` timestamp query download buffer"),
        size: unsafe { timestamp_query_resources.resolve_buffer() }.size(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    };
    executor
        .device()
        .create_buffer(&timestamp_query_download_buffer_desc)
        .into()
}

fn _wgpu_copy_into_position_tag_download_buffer(
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
            position_tag_download_buffer,
            0,
            position_tag_positions_binding.size.unwrap().get(),
        );
    }
}

fn wgpu_copy_into_timestamp_query_download_buffer(
    executor: &GpuExecutor,
    timestamp_query_download_buffer: Option<&wgpu::Buffer>,
    command_encoder: &mut wgpu::CommandEncoder,
) {
    let Some((timestamp_query_resources, timestamp_query_download_buffer)) = executor
        .timestamp_query_resources()
        .zip(timestamp_query_download_buffer)
    else {
        return;
    };

    command_encoder.copy_buffer_to_buffer(
        unsafe { timestamp_query_resources.resolve_buffer() },
        0,
        timestamp_query_download_buffer,
        0,
        timestamp_query_download_buffer.size(),
    );
}

fn wgpu_map_whole_buffer(buffer: Option<&wgpu::Buffer>) -> Option<wgpu::BufferSlice<'_>> {
    let slice = buffer?.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    slice.into()
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
    let device_hal = unsafe { device.as_hal::<wgpu::hal::api::Vulkan>() };
    let device_raw = device_hal.map_or(null::<c_void>(), |device| unsafe {
        transmute(device.raw_device().handle())
    });
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
