use indexmap::IndexSet;

use crate::context::Context;

use self::system::{
    registry::{SystemId, SystemRegistry},
    IntoSystem,
};

use super::Executor;

pub mod system;

#[derive(Debug)]
pub struct CpuExecutor<'c> {
    context: &'c mut Context,
    systems: SystemRegistry,
    schedule: IndexSet<SystemId>,
}

impl<'c> CpuExecutor<'c> {
    #[inline]
    pub fn new(context: &'c mut Context) -> Self {
        Self {
            context,
            systems: SystemRegistry::new(),
            schedule: IndexSet::new(),
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
    pub fn into_context(self) -> &'c mut Context {
        let Self { context, .. } = self;
        context
    }

    #[inline]
    pub fn register_system<I, S>(&mut self, system: S) -> SystemId
    where
        S: IntoSystem<I>,
    {
        let Self { systems, .. } = self;
        systems.register_system(system)
    }

    #[inline]
    pub fn add_system(&mut self, system: SystemId) -> bool {
        let Self { schedule, .. } = self;
        schedule.insert(system)
    }

    #[inline]
    pub fn remove_system(&mut self, system: SystemId) -> bool {
        let Self { schedule, .. } = self;
        schedule.shift_remove(&system)
    }
}

impl Executor for CpuExecutor<'_> {
    fn execute(&mut self) {
        let Self {
            context,
            systems,
            ref schedule,
        } = self;

        for &system_id in schedule {
            let Some(info) = systems.get_system_info_mut(system_id) else {
                unreachable!("system {system_id:?} should be present");
            };
            info.system_mut().run(context)
        }
    }
}
