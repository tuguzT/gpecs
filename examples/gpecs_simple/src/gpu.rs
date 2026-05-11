use std::{fs, time::Instant};

use glam::Vec3;
use gpecs::{archetype::storage, prelude::*};
use gpecs_itertools::Itertools as _;
use gpecs_simple_types::{Mass, Position, Tag};
use num_traits::ToPrimitive;
use rayon::prelude::*;

use crate::{
    setup,
    statistics::{StatisticsRecord, log_statistics},
};

pub fn run(context: &mut Context, entity_count: u32, repeat_count: Option<usize>) -> &mut Context {
    setup::setup(context, entity_count);

    let (device, queue) = init_wgpu();
    let mut executor = GpuExecutor::new(context, device.clone());

    // Move all the archetypes to GPU-accessible memory
    executor
        .register_archetype_of::<(Position, Mass, Tag)>()
        .expect("archetype of `Position`, `Mass` & `Tag` should contain unique component ids");

    // let position_tag_gpu_archetype_id = executor
    //     .register_archetype_of::<(Position, Tag)>()
    //     .expect("archetype of `Position` and `Tag` should contain unique component ids");

    register_gpu_systems(&mut executor);

    log::info!("Starting to execute systems on GPU...");
    for i in (0_u128..).maybe_take(repeat_count) {
        #[cfg(debug_assertions)]
        unsafe {
            device.start_graphics_debugger_capture();
        }

        let timestamp = Instant::now();

        let mut command_encoder = init_wgpu_command_encoder(&device);
        executor.execute(&mut command_encoder);

        // let mut context_mapper = executor.context_mapper();
        // context_mapper.map_archetype(position_tag_gpu_archetype_id, &mut command_encoder);

        let command_buffer = command_encoder.finish();
        let submission_index = queue.submit([command_buffer]);

        let poll_type = wgpu::PollType::Wait {
            submission_index: Some(submission_index),
            timeout: None,
        };
        device
            .poll(poll_type)
            .expect("device should be polled successfully");

        // let (position_tag_archetype_storage, components) = context_mapper
        //     .get_mut_archetype_with_components(position_tag_gpu_archetype_id)
        //     .expect("archetype of `Position` and `Tag` should already be mapped");

        let elapsed = timestamp.elapsed();

        #[cfg(debug_assertions)]
        unsafe {
            device.stop_graphics_debugger_capture();
        }

        // let positions = position_tag_archetype_storage
        //     .as_bundles::<(Position,)>(&components.as_view())
        //     .expect("archetype should contain `Position` components");
        // check_positions(positions);

        let statistics = collect_statistics(&executor, &queue);
        log_statistics("GPU", statistics, i, elapsed);
    }

    // Return context from the executor to the caller
    executor.into_context(&queue)
}

fn register_gpu_systems(executor: &mut GpuExecutor) {
    let shader_module = init_wgpu_shader(executor.device());

    let position_gpu_id = executor.register_component::<Position>();
    let position_gpu_system_descriptor = GpuSystemDescriptor {
        label: Some("update entity position"),
        shader_module: shader_module.clone(),
        entry_point: Some("update_entity_position"),
        dispatch_strategy: DispatchStrategy::default(),
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
        dispatch_strategy: DispatchStrategy::default(),
        bind_entities: true,
        bind_components: [(mass_gpu_id, GpuComponentAccess::ReadWrite)],
        additional_bindings: [],
    };
    let mass_gpu_system_id = executor
        .register_system(mass_gpu_system_descriptor)
        .expect("GPU system by shader module should be registered");

    executor.add_system(positions_gpu_system_id);
    executor.add_system(mass_gpu_system_id);
}

fn collect_statistics(executor: &GpuExecutor, queue: &wgpu::Queue) -> Vec<StatisticsRecord> {
    let statistics = executor
        .timestamp_query_statistics(queue)
        .expect("timestamp queries should be enabled")
        .expect("timestamp query statistics should be ready");

    statistics
        .iter()
        .flat_map(|(system, statistics)| {
            let Some(system_shader) = executor.systems().get_system_shader(system) else {
                unreachable!("{system} should exist")
            };

            let label = system_shader.label().expect("GPU system should be labeled");
            let mut statistics: Vec<_> = statistics
                .iter()
                .map(|(archetype, statistics)| StatisticsRecord {
                    system: system.into(),
                    name: label.to_owned().into(),
                    archetype: archetype.into(),
                    elapsed: statistics.duration,
                })
                .collect();
            statistics.sort();
            statistics
        })
        .collect()
}

fn _check_positions(positions: storage::Bundles<(Position,)>) {
    let positions = positions.into_par_iter().enumerate();
    positions.for_each(|(index, item)| {
        let (entity, (position,)) = item;
        let expected_position = Position {
            data: Vec3 {
                x: entity.index().to_f32().unwrap(),
                y: entity.index().to_f32().unwrap() / 2.0,
                z: -entity.index().to_f32().unwrap() / 2.0,
            },
            padding: Default::default(),
        };

        assert_eq!(
            position, &expected_position,
            "position does not match expected at {index}",
        );
    });
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
    assert!(
        adapter
            .features()
            .contains(wgpu::Features::MAPPABLE_PRIMARY_BUFFERS),
        "adapter does not support mappable primary buffers, whic are required",
    );

    let device_desc = wgpu::DeviceDescriptor {
        label: Some("`gpecs` integration test device"),
        required_features: wgpu::Features::TIMESTAMP_QUERY
            | wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES
            | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
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
