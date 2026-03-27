#[cfg(test)]
mod tests {
    use std::fs;

    fn init_wgpu() -> (wgpu::Device, wgpu::Queue) {
        let instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
        let instance = wgpu::Instance::new(instance_desc);

        let adapter_options = wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        };
        let adapter = pollster::block_on(instance.request_adapter(&adapter_options))
            .expect("failed to create adapter");

        let adapter_limits = adapter.limits();
        let adapter_downlevel_capabilities = adapter.get_downlevel_capabilities();

        assert!(
            adapter_downlevel_capabilities
                .flags
                .contains(wgpu::DownlevelFlags::COMPUTE_SHADERS),
            "adapter does not support compute shaders, which are required",
        );

        let device_desc = wgpu::DeviceDescriptor {
            label: Some("`gpecs` integration test device"),
            required_features: wgpu::Features::default(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            required_limits: adapter_limits,
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        };
        let (device, queue) = pollster::block_on(adapter.request_device(&device_desc))
            .expect("failed to create device & queue");

        (device, queue)
    }

    #[test]
    #[ignore] // TODO: fix "Type [14] '&[gpecs_soa_erased::gpecs_soa::field::FieldDescriptor]' is invalid; Expected data type, found [12]"
    fn spirv_to_shader_module() {
        const SHADER_PATH: &str = env!("gpecs_shaders.spv");

        let (device, _) = init_wgpu();
        let data = fs::read(SHADER_PATH).expect("SPIR-V shader file should exist");

        let shader_desc = wgpu::ShaderModuleDescriptor {
            label: Some("`gpecs` shader"),
            source: wgpu::util::make_spirv(&data),
        };
        let shader_module = device.create_shader_module(shader_desc);

        let shader_compilation_info = pollster::block_on(shader_module.get_compilation_info());
        println!("Shader compilation info: {shader_compilation_info:#?}");
    }
}
