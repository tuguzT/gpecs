use crate::context::Context;

use self::component::{
    registry::{GpuComponentId, GpuComponentRegistry},
    GpuComponent,
};

use super::Executor;

pub mod bundle;
pub mod component;

#[derive(Debug)]
pub struct GpuExecutor<'context> {
    context: &'context mut Context,
    components: GpuComponentRegistry,
    // then add some struct with data on GPU
}

impl<'context> GpuExecutor<'context> {
    #[inline]
    pub fn new(context: &'context mut Context) -> Self {
        Self {
            context,
            components: GpuComponentRegistry::new(),
        }
    }

    #[inline]
    pub fn context(&self) -> &Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub async fn context_mut(&mut self) -> &mut Context {
        self.sync().await;

        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub async fn into_context(mut self) -> &'context mut Context {
        self.sync().await;

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
    pub async fn sync(&mut self) {
        eprintln!("map buffers from GPU to the CPU")
    }
}

impl Executor for GpuExecutor<'_> {
    #[inline]
    fn execute(&mut self) {
        println!("Hello from the GPU executor!")
    }
}
