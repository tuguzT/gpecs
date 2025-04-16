use crate::context::Context;

use super::Executor;

#[derive(Debug)]
pub struct GpuExecutor<'context> {
    context: &'context mut Context,
    // then add some struct with data on GPU
}

impl<'context> GpuExecutor<'context> {
    #[inline]
    pub fn new(context: &'context mut Context) -> Self {
        Self { context }
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
