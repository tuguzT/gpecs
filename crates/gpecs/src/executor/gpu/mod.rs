use std::any::TypeId;

use crate::{
    archetype::{
        error::{DuplicateComponentError, GetComponentsError},
        registry::ArchetypeInfo,
    },
    component::registry::ComponentInfo,
    context::Context,
};

use self::{
    archetype::registry::{GpuArchetypeId, GpuArchetypeRegistry},
    bundle::GpuBundle,
    component::{
        registry::{GpuComponentId, GpuComponentRegistry},
        GpuComponent,
    },
};

use super::Executor;

pub mod archetype;
pub mod bundle;
pub mod component;

#[derive(Debug)]
pub struct GpuExecutor<'context> {
    context: &'context mut Context,
    components: GpuComponentRegistry,
    archetypes: GpuArchetypeRegistry,
    // TODO: add some struct with GPU shaders and their schedule
}

impl<'context> GpuExecutor<'context> {
    #[inline]
    pub fn new(context: &'context mut Context) -> Self {
        Self {
            context,
            components: GpuComponentRegistry::new(),
            archetypes: GpuArchetypeRegistry::new(),
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
    pub fn context_mut(&mut self) -> &mut Context {
        let Self { context, .. } = self;
        context
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
            components: gpu_components,
            archetypes: gpu_archetypes,
            ..
        } = self;
        #[allow(unsafe_code)]
        let (_, _, components, archetypes) = unsafe { context.as_parts_mut() };

        gpu_archetypes.register_archetype::<B>(archetypes, components, gpu_components)
    }

    #[inline]
    pub fn get_archetype_info(&self, archetype_id: GpuArchetypeId) -> Option<&ArchetypeInfo> {
        let Self { context, .. } = self;
        context.get_archetype_info(archetype_id.into())
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

    // TODO: methods to copy data from CPU to GPU and vice versa
    //       and decide what to do with mutable access to the context
}

impl Executor for GpuExecutor<'_> {
    #[inline]
    fn execute(&mut self) {
        println!("Hello from the GPU executor!")
    }
}
