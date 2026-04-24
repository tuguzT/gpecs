use std::{
    collections::HashMap,
    fs,
    time::{Duration, Instant},
};

use gpecs::prelude::*;
use gpecs_simple_types::{Mass, Position, Tag};

use crate::{ITER_COUNT, setup::setup};

pub fn run(context: &mut Context) {
    setup(context);

    let (device, queue) = init_wgpu();
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
        #[cfg(debug_assertions)]
        unsafe {
            device.start_graphics_debugger_capture();
        }

        let timestamp = Instant::now();

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

        // Map download buffer to CPU memory
        // let position_tag_download_slice =
        //     wgpu_map_whole_buffer(position_tag_download_buffer.as_ref());

        let poll_type = wgpu::PollType::Wait {
            submission_index: Some(submission_index),
            timeout: None,
        };
        device.poll(poll_type).expect("device should poll");

        let mut mapped_context = executor.map_context(&queue);
        let _context = mapped_context
            .context(PollType::wait_indefinitely())
            .expect("waiting poll should be successful");
        let _context = mapped_context
            .context(PollType::wait_indefinitely())
            .expect("should be already at ready state");

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

        let elapsed = timestamp.elapsed();

        #[cfg(debug_assertions)]
        unsafe {
            device.stop_graphics_debugger_capture();
        }

        // Check data inside of the timestamp query download buffer
        if let Some(statistics) = executor.timestamp_query_statistics(&queue) {
            let statistics = statistics.expect("timestamp query statistics should be ready");
            for system_statistics in &statistics {
                let system_id = system_statistics.system_id();
                let Some(system_shader) = executor.systems().get_system_info(system_id) else {
                    unreachable!("{system_id} should exist")
                };

                let total_duration: Duration = system_statistics
                    .iter()
                    .map(|archetype_stats| archetype_stats.duration)
                    .sum();
                let name = system_shader.label().unwrap_or("<unknown>");
                log::info!(">>>> `{name}` system took {total_duration:?}");
            }
        }
        log::info!("Execution of GPU systems {i} took {elapsed:?}");
    }

    let context = executor.into_context(&queue);
    context.destroy_all();
}

fn init_wgpu() -> (wgpu::Device, wgpu::Queue) {
    let instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
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
