use crate::{archetype::error::DuplicateComponentError, context::Context};

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
    pub fn context_mut(&mut self) -> &mut Context {
        let Self { context, .. } = self;
        context
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
    pub fn map_async<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Context),
    {
        eprintln!("map buffers from GPU to the CPU");

        let Self { context, .. } = self;
        f(context);
    }

    #[inline]
    pub fn unmap(&mut self) {
        eprintln!("unmap buffers from CPU to the GPU");
    }
}

impl Executor for GpuExecutor<'_> {
    #[inline]
    fn execute(&mut self) {
        println!("Hello from the GPU executor!")
    }
}
