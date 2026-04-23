use std::{
    collections::HashMap,
    ffi::c_void,
    fs,
    mem::transmute,
    ptr::null,
    time::{Duration, Instant},
};

use gpecs::prelude::*;
use gpecs_simple_types::{Mass, Position, Tag};
use itertools::Itertools;
use num_traits::ToPrimitive;
use renderdoc::{RenderDoc, V141};

use crate::{ITER_COUNT, setup::setup};

pub fn run(context: &mut Context) {
    let (device, queue) = init_wgpu();

    let mut renderdoc = init_renderdoc();

    setup(context);

    let mut executor = GpuExecutor::new(context, device.clone());

    executor
        .register_archetype_of::<(Position, Mass)>()
        .expect("archetype of `Position` and `Mass` should contain unique component ids");
    let _position_tag_gpu_archetype_id = executor
        .register_archetype_of::<(Position, Tag)>()
        .expect("archetype of `Position` and `Tag` should contain unique component ids");

    let shader_module = init_wgpu_shader(&device);

    let position_gpu_id = executor.register_component::<Position>();
    let position_gpu_system_descriptor = GpuSystemDescriptor {
        label: Some("update entity position"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_entity_position"),
        workgroup_size: 64.try_into().ok(),
        bind_entities: true,
        bind_components: [(position_gpu_id, GpuComponentAccess::ReadWrite)],
        additional_bindings: [],
    };
    let positions_gpu_system_id = executor
        .register_system(position_gpu_system_descriptor)
        .expect("GPU system by shader module should be registered");

    let mass_gpu_id = executor.register_component::<Mass>();
    let mass_gpu_system_descriptor = GpuSystemDescriptor {
        label: Some("update entity mass"),
        shader_module,
        entry_point: Some("update_entity_mass"),
        workgroup_size: 64.try_into().ok(),
        bind_entities: true,
        bind_components: [(mass_gpu_id, GpuComponentAccess::ReadWrite)],
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

        let command_buffer = command_encoder.finish();
        let submission_index = queue.submit([command_buffer]);

        let timestamp_query_download_buffer = executor
            .timestamp_query_resources()
            .map(|resources| unsafe { resources.download_buffer() });
        let timestamp_query_download_slice = timestamp_query_download_buffer
            .map(|buffer| buffer.slice(..))
            .inspect(|slice| slice.map_async(wgpu::MapMode::Read, |_| {}));

        // Map download buffer to CPU memory
        // let position_tag_download_slice =
        //     wgpu_map_whole_buffer(position_tag_download_buffer.as_ref());

        let poll_type = wgpu::PollType::Wait {
            submission_index: Some(submission_index),
            timeout: None,
        };
        device.poll(poll_type).expect("device should poll");

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
        if let Some(timestamp_query_download_slice) = timestamp_query_download_slice {
            let timestamp_query_view = timestamp_query_download_slice.get_mapped_range();
            let timestamp_query_raw: &[u64] = bytemuck::cast_slice(&timestamp_query_view);
            log::info!("Timestamp query raw data: {timestamp_query_raw:#?}");

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

    let context = executor.into_context(&queue);
    context.destroy_all();
}

fn init_wgpu() -> (wgpu::Device, wgpu::Queue) {
    let instance_desc = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    };
    let instance = wgpu::Instance::new(instance_desc);

    let adapter_options = wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    };
    let adapter = pollster::block_on(instance.request_adapter(&adapter_options))
        .expect("failed to create adapter");

    let adapter_info = adapter.get_info();
    log::info!("Running on:\n{adapter_info:#?}");

    let adapter_features = adapter.features();
    log::info!("Adapter features:\n{adapter_features:#?}");

    let adapter_limits = adapter.limits();
    log::info!("Adapter limits:\n{adapter_limits:#?}");

    let adapter_downlevel_capabilities = adapter.get_downlevel_capabilities();
    log::info!("Adapter downlevel capabilities:\n{adapter_downlevel_capabilities:#?}");

    assert!(
        adapter_downlevel_capabilities
            .flags
            .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS),
        "adapter does not support compute shaders, which are required",
    );
    assert!(
        adapter_features.contains(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES),
        "adapter does not support timestamp queries inside passes, which are required",
    );

    let device_desc = wgpu::DeviceDescriptor {
        label: Some("`gpecs` integration test device"),
        required_features: wgpu::Features::TIMESTAMP_QUERY
            | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        required_limits: adapter_limits,
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&device_desc))
        .expect("failed to create device & queue");

    let device_limits = device.limits();
    log::info!("Limits of the current device:\n{device_limits:#?}");

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
    let position_tag_storage_buffer_slices = position_tag_gpu_archetype_info.slices();
    log::debug!(
        "{position_tag_gpu_archetype_id:?} buffer slices:\n{position_tag_storage_buffer_slices:#?}"
    );

    let position_tag_storage_buffer_component_slices: HashMap<_, _> =
        position_tag_storage_buffer_slices.components.collect();
    let position_tag_positions_binding = position_tag_storage_buffer_component_slices
        .get(&position_gpu_id)
        .copied()
        .flatten()?;
    let position_tag_tags_binding = position_tag_storage_buffer_component_slices
        .get(&tag_gpu_id)
        .copied()
        .flatten();
    assert!(position_tag_tags_binding.is_none());

    let position_tag_download_buffer_label =
        format!("`gpecs` {position_tag_gpu_archetype_id:?} download buffer");
    let position_tag_download_buffer_desc = wgpu::BufferDescriptor {
        label: Some(&position_tag_download_buffer_label),
        size: position_tag_positions_binding.size().get(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    };
    executor
        .device()
        .create_buffer(&position_tag_download_buffer_desc)
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
    let position_tag_storage_buffer_slices = position_tag_gpu_archetype_info.slices();

    let position_tag_storage_buffer_component_slices: HashMap<_, _> =
        position_tag_storage_buffer_slices.components.collect();
    let position_tag_positions_binding = position_tag_storage_buffer_component_slices
        .get(&position_gpu_id)
        .copied()
        .flatten();
    let position_tag_tags_binding = position_tag_storage_buffer_component_slices
        .get(&tag_gpu_id)
        .copied()
        .flatten();
    assert!(position_tag_tags_binding.is_none());

    if let Some((position_tag_download_buffer, position_tag_positions_binding)) =
        position_tag_download_buffer.zip(position_tag_positions_binding)
    {
        let position_tag_positions_slice = unsafe { position_tag_positions_binding.as_slice() };
        command_encoder.copy_buffer_to_buffer(
            position_tag_positions_slice.buffer(),
            position_tag_positions_slice.offset(),
            position_tag_download_buffer,
            0,
            position_tag_positions_slice.size().get(),
        );
    }
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
