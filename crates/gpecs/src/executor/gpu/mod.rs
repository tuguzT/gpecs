use std::any::TypeId;

use system::schedule::GpuSystemSchedule;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, BindingResource, CommandEncoder, ComputePassDescriptor,
    Device, ShaderModule,
};

use crate::{
    archetype::error::{DuplicateComponentError, GetComponentsError},
    component::registry::ComponentInfo,
    context::Context,
};

use self::{
    archetype::registry::{GpuArchetypeId, GpuArchetypeInfo, GpuArchetypeRegistry},
    bundle::GpuBundle,
    component::{
        registry::{GpuComponentId, GpuComponentRegistry},
        GpuComponent,
    },
    system::registry::{GpuSystemId, GpuSystemInfo, GpuSystemRegistry},
};

pub mod archetype;
pub mod bundle;
pub mod component;
pub mod system;

#[derive(Debug)]
pub struct GpuExecutor<'context> {
    context: &'context mut Context,
    device: Device,
    components: GpuComponentRegistry,
    archetypes: GpuArchetypeRegistry,
    systems: GpuSystemRegistry,
    schedule: GpuSystemSchedule,
}

impl<'context> GpuExecutor<'context> {
    #[inline]
    pub fn new(context: &'context mut Context, device: Device) -> Self {
        Self {
            context,
            device,
            components: GpuComponentRegistry::new(),
            archetypes: GpuArchetypeRegistry::new(),
            systems: GpuSystemRegistry::new(),
            schedule: GpuSystemSchedule::new(),
        }
    }

    #[inline]
    pub fn context(&self) -> &Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub fn components(&self) -> &GpuComponentRegistry {
        let Self { components, .. } = self;
        components
    }

    #[inline]
    pub fn archetypes(&self) -> &GpuArchetypeRegistry {
        let Self { archetypes, .. } = self;
        archetypes
    }

    #[inline]
    pub fn systems(&self) -> &GpuSystemRegistry {
        let Self { systems, .. } = self;
        systems
    }

    #[inline]
    pub fn components_mut(&mut self) -> &mut GpuComponentRegistry {
        let Self { components, .. } = self;
        components
    }

    #[inline]
    pub fn into_context(self) -> &'context mut Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub fn register_component<C>(&mut self) -> GpuComponentId
    where
        C: GpuComponent,
    {
        let Self {
            context,
            components,
            ..
        } = self;

        components.register_component::<C>(context.components_mut())
    }

    #[inline]
    pub fn get_component_info(&self, component_id: GpuComponentId) -> Option<&ComponentInfo> {
        let Self { context, .. } = self;
        context.get_component_info(component_id.into())
    }

    #[inline]
    pub fn component_id_from(&self, type_id: TypeId) -> Option<GpuComponentId> {
        let Self {
            context,
            components,
            ..
        } = self;

        let component_id = context.component_id_from(type_id)?;
        components.map_component_id(component_id)
    }

    #[inline]
    pub fn component_id<C>(&self) -> Option<GpuComponentId>
    where
        C: GpuComponent,
    {
        let Self {
            context,
            components,
            ..
        } = self;

        let component_id = context.component_id::<C>()?;
        components.map_component_id(component_id)
    }

    #[inline]
    pub fn register_archetype<B>(&mut self) -> Result<GpuArchetypeId, DuplicateComponentError>
    where
        B: GpuBundle,
    {
        let Self {
            context,
            ref device,
            components: gpu_components,
            archetypes: gpu_archetypes,
            ..
        } = self;
        #[allow(unsafe_code)]
        let (_, _, components, archetypes) = unsafe { context.as_parts_mut() };

        gpu_archetypes.register_archetype::<B>(components, archetypes, gpu_components, device)
    }

    #[inline]
    pub fn get_archetype_info(&self, archetype_id: GpuArchetypeId) -> Option<&GpuArchetypeInfo> {
        let Self { archetypes, .. } = self;
        archetypes.get_archetype_info(archetype_id)
    }

    #[inline]
    pub fn archetype_id<B>(&self) -> Result<Option<GpuArchetypeId>, GetComponentsError>
    where
        B: GpuBundle,
    {
        let Self {
            context,
            archetypes,
            ..
        } = self;

        let Some(archetype_id) = context.archetype_id::<B>()? else {
            return Ok(None);
        };
        let archetype_id = archetypes.map_archetype_id(archetype_id);
        Ok(archetype_id)
    }

    #[inline]
    pub fn register_system<I>(
        &mut self,
        shader_module: ShaderModule,
        workgroup_count: Option<u32>,
        entry_point: Option<&str>,
        bind_entities: bool,
        bind_components: I,
    ) -> Result<GpuSystemId, DuplicateComponentError>
    where
        I: IntoIterator<Item = GpuComponentId>,
    {
        let Self {
            ref context,
            ref device,
            systems,
            ..
        } = self;
        let components = context.components();

        systems.register_system(
            components,
            device,
            shader_module,
            workgroup_count,
            entry_point,
            bind_entities,
            bind_components,
        )
    }

    #[inline]
    pub fn get_system_info(&self, system_id: GpuSystemId) -> Option<&GpuSystemInfo> {
        let Self { systems, .. } = self;
        systems.get_system_info(system_id)
    }

    #[inline]
    pub fn add_system(&mut self, system_id: GpuSystemId) -> bool {
        let Self { schedule, .. } = self;
        schedule.add_system(system_id)
    }

    #[inline]
    pub fn remove_system(&mut self, system_id: GpuSystemId) -> bool {
        let Self { schedule, .. } = self;
        schedule.remove_system(system_id)
    }

    #[inline]
    #[allow(unsafe_code)]
    pub fn execute(&mut self, command_encoder: &mut CommandEncoder) {
        let Self {
            ref context,
            ref device,
            ref archetypes,
            systems,
            schedule,
            ..
        } = self;

        let compute_pass_desc = ComputePassDescriptor {
            label: Some("`gpecs` executor compute pass"),
            timestamp_writes: None,
        };
        let mut compute_pass = command_encoder.begin_compute_pass(&compute_pass_desc);

        for system_id in schedule.iter() {
            let Some(system_info) = systems.get_system_info(system_id) else {
                unreachable!("system {system_id:?} should exist");
            };

            let shader = system_info.shader();
            let component_ids = shader
                .components_bind_group_layout_entries()
                .map(|(component_id, _)| component_id.into());
            let Ok(compatible_archetypes) =
                context.archetypes().compatible_archetypes(component_ids)
            else {
                unreachable!("system {system_id:?} should have compatible archetypes");
            };
            for archetype_info in compatible_archetypes {
                let Some(archetype_id) = archetypes.map_archetype_id(archetype_info.id()) else {
                    continue;
                };
                let Some(archetype_info) = archetypes.get_archetype_info(archetype_id) else {
                    unreachable!("archetype {archetype_id:?} should exist");
                };

                let mut storage_buffer_bindings =
                    unsafe { archetype_info.storage().storage_buffer_bindings() };
                let mut bind_group_entries = Vec::new();

                if let Some(entities_bind_group_layout_entry) =
                    shader.entities_bind_group_layout_entry()
                {
                    let Some(entities_buffer_binding) = storage_buffer_bindings.entities else {
                        continue;
                    };
                    let entities_bind_group_entry = BindGroupEntry {
                        binding: entities_bind_group_layout_entry.binding,
                        resource: BindingResource::Buffer(entities_buffer_binding),
                    };
                    bind_group_entries.push(entities_bind_group_entry);
                }
                let components_bind_group_layout_entries =
                    shader.components_bind_group_layout_entries();
                for (component_id, component_bind_group_layout_entry) in
                    components_bind_group_layout_entries
                {
                    let Some(component_bind_group_layout_entry) = component_bind_group_layout_entry
                    else {
                        continue;
                    };
                    let Some(component_buffer_binding) = storage_buffer_bindings
                        .components
                        .swap_remove(&component_id.into_id())
                    else {
                        unreachable!("archetype {archetype_id:?} should have {component_id:?}");
                    };
                    let Some(component_buffer_binding) = component_buffer_binding else {
                        break;
                    };

                    let component_bind_group_entry = BindGroupEntry {
                        binding: component_bind_group_layout_entry.binding,
                        resource: BindingResource::Buffer(component_buffer_binding),
                    };
                    bind_group_entries.push(component_bind_group_entry);
                }
                if bind_group_entries.is_empty() {
                    continue;
                }

                let bind_group_label =
                    format!("`gpecs` {system_id:?} bind group for {archetype_id:?}");
                let bind_group_desc = BindGroupDescriptor {
                    label: Some(&bind_group_label),
                    layout: shader.bind_group_layout(),
                    entries: &bind_group_entries,
                };
                let bind_group = device.create_bind_group(&bind_group_desc);

                compute_pass.set_pipeline(shader.compute_pipeline());
                compute_pass.set_bind_group(0, &bind_group, &[]);

                let storage_len = u32::try_from(archetype_info.storage().len())
                    .expect("storage length should fit into `u32`");
                let workgroup_count = storage_len.div_ceil(shader.workgroup_count().unwrap_or(64));
                compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
            }
        }
    }

    // TODO: methods to copy data from CPU to GPU and vice versa
    //       do not grant mutable access to the context
}
