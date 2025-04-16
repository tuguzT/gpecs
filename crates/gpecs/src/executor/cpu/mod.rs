use crate::context::Context;

use super::Executor;

pub struct CpuExecutor<'c> {
    context: &'c mut Context,
}

impl<'c> CpuExecutor<'c> {
    #[inline]
    pub fn new(context: &'c mut Context) -> Self {
        Self { context }
    }

    #[inline]
    pub fn context(&self) -> &Context {
        let Self { context } = self;
        context
    }

    #[inline]
    pub fn context_mut(&mut self) -> &mut Context {
        let Self { context } = self;
        context
    }

    #[inline]
    pub fn into_context(self) -> &'c mut Context {
        let Self { context } = self;
        context
    }
}

impl Executor for CpuExecutor<'_> {
    fn execute(&mut self) {
        println!("Hello from CPU executor!")
    }
}
