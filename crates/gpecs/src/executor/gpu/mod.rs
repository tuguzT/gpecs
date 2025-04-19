use std::any::TypeId;

use system::schedule::GpuSystemSchedule;
use wgpu::{CommandEncoder, Device, ShaderModule};

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
        entry_point: Option<&str>,
        bind_entities: bool,
        component_ids: I,
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
            entry_point,
            bind_entities,
            component_ids,
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
    pub fn execute(&mut self, command_encoder: &mut CommandEncoder) {
        let Self {
            context: _,
            device: _,
            components: _,
            archetypes: _,
            systems,
            schedule,
        } = self;

        let _ = command_encoder;
        for system_id in schedule.iter() {
            let Some(system_info) = systems.get_system_info(system_id) else {
                unreachable!("system {system_id:?} should exist");
            };
            // TODO: collect all compatible archetypes
            println!("Executing system {:?}...", system_info.id());
        }
    }

    // TODO: methods to copy data from CPU to GPU and vice versa
    //       and decide what to do with mutable access to the context
}
